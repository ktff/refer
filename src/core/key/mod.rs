mod key;
mod key_path;
mod path;

pub use key::{AnyKey, Key};
pub use key_path::*;
pub use path::{Path, PathLeaf, PathRegion};

// NOTE: Base must be greater than usize
pub use base::*;

// TODO: Test this whole stack

pub const INDEX_BASE_BITS: std::num::NonZeroU32 =
    std::num::NonZeroU32::new(std::mem::size_of::<IndexBase>() as u32 * 8).expect("Zero bits");

#[cfg(feature = "base_u64")]
mod base {
    pub type IndexBase = u64;
    pub type Index = std::num::NonZeroU64;
}

#[cfg(feature = "base_u128")]
mod base {
    pub type IndexBase = u128;
    pub type Index = std::num::NonZeroU128;
}
