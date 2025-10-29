//! CSV parser with SIMD acceleration

use crate::portability::{hamming, trailing_zeros};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Parsed CSV structure containing field separator indexes
pub struct ParsedCsv {
    pub indexes: Vec<u32>,
}

impl ParsedCsv {
    /// Create a new ParsedCsv with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indexes: Vec::with_capacity(capacity),
        }
    }
}

/// SIMD input structure for processing 64 bytes at a time
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
struct SimdInput {
    lo: __m256i,
    hi: __m256i,
}

#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
struct SimdInput {
    i0: uint8x16_t,
    i1: uint8x16_t,
    i2: uint8x16_t,
    i3: uint8x16_t,
}

/// Fill SIMD input from buffer
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn fill_input(ptr: *const u8) -> SimdInput {
    SimdInput {
        lo: _mm256_loadu_si256(ptr as *const __m256i),
        hi: _mm256_loadu_si256(ptr.add(32) as *const __m256i),
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn fill_input(ptr: *const u8) -> SimdInput {
    SimdInput {
        i0: vld1q_u8(ptr),
        i1: vld1q_u8(ptr.add(16)),
        i2: vld1q_u8(ptr.add(32)),
        i3: vld1q_u8(ptr.add(48)),
    }
}

/// Compare all bytes in SIMD input against a mask value
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn cmp_mask_against_input(input: SimdInput, mask: u8) -> u64 {
    let mask_vec = _mm256_set1_epi8(mask as i8);
    let cmp_res_0 = _mm256_cmpeq_epi8(input.lo, mask_vec);
    let res_0 = _mm256_movemask_epi8(cmp_res_0) as u32 as u64;
    let cmp_res_1 = _mm256_cmpeq_epi8(input.hi, mask_vec);
    let res_1 = _mm256_movemask_epi8(cmp_res_1) as u64;
    res_0 | (res_1 << 32)
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn cmp_mask_against_input(input: SimdInput, mask: u8) -> u64 {
    let mask_vec = vdupq_n_u8(mask);
    let cmp_res_0 = vceqq_u8(input.i0, mask_vec);
    let cmp_res_1 = vceqq_u8(input.i1, mask_vec);
    let cmp_res_2 = vceqq_u8(input.i2, mask_vec);
    let cmp_res_3 = vceqq_u8(input.i3, mask_vec);
    neon_movemask_bulk(cmp_res_0, cmp_res_1, cmp_res_2, cmp_res_3)
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn neon_movemask_bulk(
    i0: uint8x16_t,
    i1: uint8x16_t,
    i2: uint8x16_t,
    i3: uint8x16_t,
) -> u64 {
    // Simplified NEON movemask implementation
    // Extract high bit from each byte and pack into u64
    let mask0 = neon_movemask(i0);
    let mask1 = neon_movemask(i1);
    let mask2 = neon_movemask(i2);
    let mask3 = neon_movemask(i3);
    
    (mask0 as u64) 
        | ((mask1 as u64) << 16) 
        | ((mask2 as u64) << 32) 
        | ((mask3 as u64) << 48)
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn neon_movemask(input: uint8x16_t) -> u16 {
    // Extract high bit from each byte
    let bit_mask = vdupq_n_u8(0x80);
    let masked = vandq_u8(input, bit_mask);
    
    // Use a lookup-based approach to pack bits
    let low = vget_low_u8(masked);
    let high = vget_high_u8(masked);
    
    let mut result = 0u16;
    for i in 0..8 {
        if vget_lane_u8(low, i) != 0 {
            result |= 1 << i;
        }
        if vget_lane_u8(high, i) != 0 {
            result |= 1 << (i + 8);
        }
    }
    result
}

/// Find quote mask using carryless multiplication
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn find_quote_mask(input: SimdInput, prev_iter_inside_quote: &mut u64) -> u64 {
    let quote_bits = cmp_mask_against_input(input, b'"');
    
    // Use carryless multiply to find quote regions
    let quote_mask = _mm_cvtsi128_si64(_mm_clmulepi64_si128(
        _mm_set_epi64x(0, quote_bits as i64),
        _mm_set1_epi8(-1),
        0,
    )) as u64;
    
    let quote_mask = quote_mask ^ *prev_iter_inside_quote;
    
    // Update for next iteration
    *prev_iter_inside_quote = ((quote_mask as i64) >> 63) as u64;
    
    quote_mask
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn find_quote_mask(input: SimdInput, prev_iter_inside_quote: &mut u64) -> u64 {
    let quote_bits = cmp_mask_against_input(input, b'"');
    
    // Use polynomial multiplication for ARM
    let quote_mask = vmull_p64(!0u64, quote_bits);
    let quote_mask = quote_mask ^ *prev_iter_inside_quote;
    
    *prev_iter_inside_quote = ((quote_mask as i64) >> 63) as u64;
    
    quote_mask
}

/// Flatten bits into indexes
#[inline(always)]
fn flatten_bits(base_ptr: &mut Vec<u32>, idx: u32, mut bits: u64) {
    if bits == 0 {
        return;
    }

    let cnt = hamming(bits);
    
    // Unrolled loop for first 8 bits
    if cnt > 0 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 1 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 2 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 3 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 4 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 5 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 6 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    if cnt > 7 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
    
    // Continue for 9-16 bits
    if cnt > 8 {
        for _ in 8..cnt.min(16) {
            base_ptr.push(idx + trailing_zeros(bits));
            bits &= bits - 1;
        }
    }
    
    // Handle remaining bits
    while bits != 0 && cnt > 16 {
        base_ptr.push(idx + trailing_zeros(bits));
        bits &= bits - 1;
    }
}

/// Parse CSV buffer and find field separator indexes
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[target_feature(enable = "pclmulqdq")]
pub unsafe fn find_indexes_avx2(buf: &[u8], pcsv: &mut ParsedCsv) -> bool {
    let len = buf.len();
    let mut prev_iter_inside_quote = 0u64;
    
    if len < 64 {
        return true;
    }
    
    let lenminus64 = len - 64;
    let mut idx = 0;

    // Buffered processing for better pipelining
    const BUFFER_SIZE: usize = 4;
    
    if lenminus64 > 64 * BUFFER_SIZE {
        let mut fields = [0u64; BUFFER_SIZE];
        
        while idx < lenminus64.saturating_sub(64 * BUFFER_SIZE - 1) {
            // Process BUFFER_SIZE chunks and store results
            for b in 0..BUFFER_SIZE {
                let internal_idx = 64 * b + idx;
                
                // Prefetch for next iteration
                #[cfg(target_arch = "x86_64")]
                {
                    let prefetch_ptr = buf.as_ptr().add(internal_idx + 128);
                    _mm_prefetch(prefetch_ptr as *const i8, _MM_HINT_T0);
                }
                
                let input = fill_input(buf.as_ptr().add(internal_idx));
                let quote_mask = find_quote_mask(input, &mut prev_iter_inside_quote);
                let sep = cmp_mask_against_input(input, b',');
                let end = cmp_mask_against_input(input, b'\n');
                
                fields[b] = (end | sep) & !quote_mask;
            }
            
            // Flatten all buffered results
            for b in 0..BUFFER_SIZE {
                let internal_idx = 64 * b + idx;
                flatten_bits(&mut pcsv.indexes, internal_idx as u32, fields[b]);
            }
            
            idx += 64 * BUFFER_SIZE;
        }
    }
    
    // Process remaining chunks
    while idx < lenminus64 {
        let input = fill_input(buf.as_ptr().add(idx));
        let quote_mask = find_quote_mask(input, &mut prev_iter_inside_quote);
        let sep = cmp_mask_against_input(input, b',');
        let end = cmp_mask_against_input(input, b'\n');
        
        let field_sep = (end | sep) & !quote_mask;
        flatten_bits(&mut pcsv.indexes, idx as u32, field_sep);
        
        idx += 64;
    }
    
    // Process remaining bytes with scalar fallback
    let in_quote_start = prev_iter_inside_quote != 0;
    process_tail_scalar(&buf[idx..], idx, pcsv, in_quote_start);

    true
}

/// Parse CSV buffer (x86_64 with runtime feature detection)
#[cfg(target_arch = "x86_64")]
pub fn find_indexes(buf: &[u8], pcsv: &mut ParsedCsv) -> bool {
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("pclmulqdq") {
        unsafe { find_indexes_avx2(buf, pcsv) }
    } else {
        find_indexes_fallback(buf, pcsv)
    }
}

/// Parse CSV buffer (ARM NEON)
#[cfg(target_arch = "aarch64")]
pub fn find_indexes(buf: &[u8], pcsv: &mut ParsedCsv) -> bool {
    let len = buf.len();
    let mut prev_iter_inside_quote = 0u64;
    
    if len < 64 {
        process_tail_scalar(buf, 0, pcsv, false);
        return true;
    }
    
    let lenminus64 = len - 64;
    let mut idx = 0;

    // Main processing loop
    unsafe {
        while idx < lenminus64 {
            let input = fill_input(buf.as_ptr().add(idx));
            let quote_mask = find_quote_mask(input, &mut prev_iter_inside_quote);
            let sep = cmp_mask_against_input(input, b',');
            let end = cmp_mask_against_input(input, b'\n');
            
            let field_sep = (end | sep) & !quote_mask;
            flatten_bits(&mut pcsv.indexes, idx as u32, field_sep);
            
            idx += 64;
        }
    }
    
    // Process remaining bytes with scalar fallback
    let in_quote_start = prev_iter_inside_quote != 0;
    process_tail_scalar(&buf[idx..], idx, pcsv, in_quote_start);

    true
}

/// Parse CSV buffer (fallback for unsupported architectures)
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn find_indexes(buf: &[u8], pcsv: &mut ParsedCsv) -> bool {
    find_indexes_fallback(buf, pcsv)
}

/// Scalar fallback implementation
fn find_indexes_fallback(buf: &[u8], pcsv: &mut ParsedCsv) -> bool {
    process_tail_scalar(buf, 0, pcsv, false);
    true
}

/// Process remaining bytes with scalar code
#[inline(always)]
fn process_tail_scalar(buf: &[u8], offset: usize, pcsv: &mut ParsedCsv, mut in_quote: bool) {
    for (i, &byte) in buf.iter().enumerate() {
        match byte {
            b'"' => in_quote = !in_quote,
            b',' | b'\n' if !in_quote => pcsv.indexes.push((offset + i) as u32),
            _ => {}
        }
    }
}

/// Parse CSV file
pub fn parse_csv(buf: &[u8]) -> ParsedCsv {
    let mut pcsv = ParsedCsv::with_capacity(buf.len() / 10); // Estimate
    find_indexes(buf, &mut pcsv);
    pcsv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_csv() {
        // Create data larger than 64 bytes to trigger SIMD path
        let mut data = Vec::new();
        for i in 0..20 {
            data.extend_from_slice(format!("field{},value{}\n", i, i).as_bytes());
        }
        
        let pcsv = parse_csv(&data);
        
        // Should find commas and newlines
        assert!(!pcsv.indexes.is_empty());
        
        // Count commas and newlines in data
        let comma_count = data.iter().filter(|&&b| b == b',').count();
        let newline_count = data.iter().filter(|&&b| b == b'\n').count();
        
        // Should find all separators
        assert_eq!(pcsv.indexes.len(), comma_count + newline_count);
    }

    #[test]
    fn test_parse_quoted_csv() {
        // Create data with quotes that's larger than 64 bytes
        let mut data = Vec::new();
        for i in 0..10 {
            data.extend_from_slice(format!("\"field,{}\",value{}\n", i, i).as_bytes());
        }
        
        let pcsv = parse_csv(&data);
        
        // Should find separators but not commas inside quotes
        assert!(!pcsv.indexes.is_empty());
        
        // There should be fewer indexes than total commas+newlines
        // because commas inside quotes don't count
        let comma_count = data.iter().filter(|&&b| b == b',').count();
        let newline_count = data.iter().filter(|&&b| b == b'\n').count();
        
        assert!(pcsv.indexes.len() < comma_count + newline_count);
    }

    #[test]
    fn test_parse_empty() {
        let data = b"";
        let pcsv = parse_csv(data);
        assert!(pcsv.indexes.is_empty());
    }

    #[test]
    fn test_parse_no_separators() {
        let mut data = vec![b'a'; 100];
        let pcsv = parse_csv(&data);
        assert!(pcsv.indexes.is_empty());
    }
}
