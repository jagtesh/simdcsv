//! Memory utilities for aligned allocation

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

/// Allocate memory aligned to a specific boundary with padding
///
/// # Safety
/// The returned pointer must be deallocated with `aligned_free`
#[inline]
pub fn allocate_padded_buffer(length: usize, padding: usize) -> Result<NonNull<u8>, String> {
    let total_size = length + padding;

    // Align to 64-byte boundary (cache line size)
    let layout =
        Layout::from_size_align(total_size, 64).map_err(|e| format!("Invalid layout: {}", e))?;

    // SAFETY: We verify the layout is valid above
    let ptr = unsafe { alloc(layout) };

    NonNull::new(ptr).ok_or_else(|| "Failed to allocate memory".to_string())
}

/// Free memory allocated with `allocate_padded_buffer`
///
/// # Safety
/// - `ptr` must have been allocated with `allocate_padded_buffer`
/// - `length` and `padding` must match the original allocation
/// - `ptr` must not be used after calling this function
#[inline]
pub unsafe fn aligned_free(ptr: NonNull<u8>, length: usize, padding: usize) {
    let total_size = length + padding;
    let layout = Layout::from_size_align_unchecked(total_size, 64);
    dealloc(ptr.as_ptr(), layout);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_free() {
        let length = 1024;
        let padding = 64;

        let ptr = allocate_padded_buffer(length, padding).unwrap();

        // Verify alignment
        assert_eq!(
            ptr.as_ptr() as usize % 64,
            0,
            "Pointer should be 64-byte aligned"
        );

        // Write and read to verify it's usable
        unsafe {
            *ptr.as_ptr() = 42;
            assert_eq!(*ptr.as_ptr(), 42);

            aligned_free(ptr, length, padding);
        }
    }

    #[test]
    fn test_zero_length() {
        let result = allocate_padded_buffer(0, 64);
        assert!(result.is_ok());

        if let Ok(ptr) = result {
            unsafe {
                aligned_free(ptr, 0, 64);
            }
        }
    }
}
