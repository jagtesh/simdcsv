//! Portability utilities for bit operations and platform-specific intrinsics

/// Count trailing zeros in a 64-bit integer
#[inline(always)]
pub fn trailing_zeros(x: u64) -> u32 {
    x.trailing_zeros()
}

/// Count leading zeros in a 64-bit integer
#[inline(always)]
pub fn leading_zeros(x: u64) -> u32 {
    x.leading_zeros()
}

/// Count number of set bits (Hamming weight/popcount)
#[inline(always)]
pub fn hamming(x: u64) -> u32 {
    x.count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_zeros() {
        assert_eq!(trailing_zeros(0b1000), 3);
        assert_eq!(trailing_zeros(0b0001), 0);
        assert_eq!(trailing_zeros(0b1010_0000), 5);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(leading_zeros(0b0001), 63);
        assert_eq!(
            leading_zeros(
                0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            ),
            0
        );
    }

    #[test]
    fn test_hamming() {
        assert_eq!(hamming(0b1111), 4);
        assert_eq!(hamming(0b1010_1010), 4);
        assert_eq!(hamming(0), 0);
    }
}
