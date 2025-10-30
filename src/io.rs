//! I/O utilities for loading CSV files with padding

use crate::memory::{aligned_free, allocate_padded_buffer};
use std::fs::File;
use std::io::Read;
use std::ptr::NonNull;

/// A buffer containing file data with padding for safe SIMD operations
pub struct PaddedBuffer {
    ptr: NonNull<u8>,
    length: usize,
    padding: usize,
}

impl PaddedBuffer {
    /// Get a slice view of the data (excluding padding)
    #[inline(always)]
    pub fn data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.length) }
    }

    /// Get the length of the data (excluding padding)
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the buffer is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Get raw pointer to the data
    #[inline(always)]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for PaddedBuffer {
    fn drop(&mut self) {
        unsafe {
            aligned_free(self.ptr, self.length, self.padding);
        }
    }
}

/// Load a file into memory with padding for safe SIMD operations
///
/// # Arguments
/// * `filename` - Path to the file to load
/// * `padding` - Number of bytes to pad at the end for safe SIMD reads
///
/// # Returns
/// A `PaddedBuffer` containing the file data with padding
pub fn get_corpus(filename: &str, padding: usize) -> Result<PaddedBuffer, String> {
    let mut file =
        File::open(filename).map_err(|e| format!("Could not open file '{}': {}", filename, e))?;

    let metadata = file
        .metadata()
        .map_err(|e| format!("Could not read file metadata: {}", e))?;

    let length = metadata.len() as usize;

    let ptr = allocate_padded_buffer(length, padding)?;

    // SAFETY: We just allocated this buffer with the correct size
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr(), length) };

    file.read_exact(slice)
        .map_err(|e| format!("Could not read file data: {}", e))?;

    Ok(PaddedBuffer {
        ptr,
        length,
        padding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_get_corpus() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_simdcsv.csv");

        {
            let mut file = File::create(&test_file).unwrap();
            file.write_all(b"a,b,c\n1,2,3\n4,5,6\n").unwrap();
        }

        let buffer = get_corpus(test_file.to_str().unwrap(), 64).unwrap();

        assert_eq!(buffer.len(), 18);
        assert_eq!(buffer.data(), b"a,b,c\n1,2,3\n4,5,6\n");

        // Cleanup
        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_padded_buffer_alignment() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_alignment.csv");

        {
            let mut file = File::create(&test_file).unwrap();
            file.write_all(b"test").unwrap();
        }

        let buffer = get_corpus(test_file.to_str().unwrap(), 64).unwrap();

        // Check alignment
        assert_eq!(
            buffer.as_ptr() as usize % 64,
            0,
            "Buffer should be 64-byte aligned"
        );

        std::fs::remove_file(test_file).ok();
    }
}
