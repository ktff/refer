mod key;
mod key_path;
mod path;

pub use key::{AnyKey, Key};
pub use key_path::*;
pub use path::{Path, PathLeaf, PathRegion};

// NOTE: Base must be greater than usize
pub type IndexBase = u64;
pub type Index = std::num::NonZeroU64;
pub const INDEX_BASE_BITS: std::num::NonZeroU32 =
    std::num::NonZeroU32::new(std::mem::size_of::<IndexBase>() as u32 * 8).expect("Zero bits");

// TODO: Test this whole stack
