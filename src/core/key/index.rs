use std::{fmt, hash::Hash, num::NonZeroU64};

const INDEX_BITS: u32 = std::mem::size_of::<Index>() as u32 * 8;
pub const MAX_KEY_LEN: u32 = INDEX_BITS;

// NOTE: Index could be larger than u64 so the possibility of changing that to u128 is left as an option.

/// Index shouldn't be zero. Instead impl can use this for optimizations and to check for invalid composite keys.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Index(pub NonZeroU64);

impl Index {
    /// Length of low bits
    pub const fn len_low_bits(self) -> u32 {
        INDEX_BITS - self.0.get().leading_zeros()
    }

    /// Length of high bits
    pub const fn len_high_bits(self) -> u32 {
        INDEX_BITS - self.0.get().trailing_zeros()
    }

    pub fn as_usize(self) -> usize {
        self.0.get() as usize
    }

    /// Pushes prefix on suffix/self from top.
    pub fn with_prefix(self, prefix_len: u32, prefix: usize) -> Self {
        debug_assert!(
            std::mem::size_of::<usize>() as u32 * 8 - prefix.leading_zeros() <= prefix_len,
            "Invalid prefix"
        );

        let prefix = (prefix as u64) << (INDEX_BITS - prefix_len);
        let suffix = NonZeroU64::new(self.0.get() >> prefix_len).expect("Invalid suffix");

        Index(prefix | suffix)
    }

    /// Splits of prefix from top of self.
    /// This is the inverse of with_prefix.
    pub fn split_prefix(self, prefix_len: u32) -> (usize, Self) {
        let prefix = (self.0.get() >> (INDEX_BITS - prefix_len)) as usize;
        let suffix = NonZeroU64::new(self.0.get() << prefix_len).expect("Invalid suffix");

        (prefix, Index(suffix))
    }

    /// Tries to split of prefix from top of self.
    /// Can fail if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> Result<(usize, Self), Self> {
        let prefix = self.0.get() >> (INDEX_BITS - prefix_len);
        let suffix = NonZeroU64::new(self.0.get() << prefix_len);

        if let Some(suffix) = suffix {
            Ok(((prefix as usize), Index(suffix)))
        } else {
            Err(Index(NonZeroU64::new(prefix).expect("Invalid prefix")))
        }
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_low() {
        assert_eq!(Index(NonZeroU64::new(1).unwrap()).len_low_bits(), 1);
        assert_eq!(Index(NonZeroU64::new(2).unwrap()).len_low_bits(), 2);
        assert_eq!(Index(NonZeroU64::new(3).unwrap()).len_low_bits(), 2);
        assert_eq!(Index(NonZeroU64::new(4).unwrap()).len_low_bits(), 3);
    }

    #[test]
    fn len_high() {
        assert_eq!(Index(NonZeroU64::new(1).unwrap()).len_high_bits(), 64);
        assert_eq!(Index(NonZeroU64::new(2).unwrap()).len_high_bits(), 63);
        assert_eq!(Index(NonZeroU64::new(3).unwrap()).len_high_bits(), 64);
        assert_eq!(Index(NonZeroU64::new(4).unwrap()).len_high_bits(), 62);
    }

    #[test]
    fn with_prefix() {
        assert_eq!(
            Index(NonZeroU64::new(0x1000_0000_0000_0000).unwrap()).with_prefix(16, 0x1234),
            Index(NonZeroU64::new(0x1234_1000_0000_0000).unwrap())
        );

        assert_eq!(
            Index(NonZeroU64::new(0x1000_0000_0000_0000).unwrap())
                .with_prefix(16, 0x1234)
                .with_prefix(8, 0x56),
            Index(NonZeroU64::new(0x5612_3410_0000_0000).unwrap())
        );
    }

    #[test]
    fn split_prefix() {
        assert_eq!(
            Index(NonZeroU64::new(0x1234_1000_0000_0000).unwrap()).split_prefix(16),
            (
                0x1234,
                Index(NonZeroU64::new(0x1000_0000_0000_0000).unwrap())
            )
        );

        assert_eq!(
            Index(NonZeroU64::new(0x5612_3410_0000_0000).unwrap())
                .split_prefix(16)
                .1
                .split_prefix(8),
            (0x34, Index(NonZeroU64::new(0x1000_0000_0000_0000).unwrap()))
        );
    }

    #[test]
    fn split_prefix_try() {
        assert_eq!(
            Index(NonZeroU64::new(0x1234_1000_0000_0000).unwrap()).split_prefix_try(16),
            Ok((
                0x1234,
                Index(NonZeroU64::new(0x1000_0000_0000_0000).unwrap())
            ))
        );

        assert_eq!(
            Index(NonZeroU64::new(0x1234_0000_0000_0000).unwrap()).split_prefix_try(16),
            Err(Index(NonZeroU64::new(0x1234).unwrap()))
        );

        assert_eq!(
            Index(NonZeroU64::new(0x5612_3410_0000_0000).unwrap())
                .split_prefix_try(16)
                .unwrap()
                .1
                .split_prefix_try(8),
            Ok((0x34, Index(NonZeroU64::new(0x1000_0000_0000_0000).unwrap())))
        );

        assert_eq!(
            Index(NonZeroU64::new(0x5612_3400_0000_0000).unwrap())
                .split_prefix_try(16)
                .unwrap()
                .1
                .split_prefix_try(8),
            Err(Index(NonZeroU64::new(0x34).unwrap()))
        );
    }


    
}
