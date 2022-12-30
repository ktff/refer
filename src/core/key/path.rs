use crate::core::{AnyItem, Index, Item};

use super::{ContainerPath, IndexBase, Key, KeyPath, INDEX_BASE_BITS};
use std::{any::TypeId, num::NonZeroU64, ptr::Pointee};

/// Path in container
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Path(Index);

impl Path {
    /// Will increase the level of the path if it's key.
    pub fn new(path: IndexBase, level: u32) -> Self {
        // path & mask | bit
        let offset = (INDEX_BASE_BITS.get() - 1).saturating_sub(level);
        let path = ((path >> offset) | 1) << offset;
        Path(Index::new(path).expect("Shouldn't be zero"))
    }

    pub fn level(&self) -> u32 {
        (INDEX_BASE_BITS.get() - 1) - self.0.trailing_zeros()
    }

    fn top(&self) -> IndexBase {
        self.0.get() ^ self.bit()
    }

    fn bit(&self) -> IndexBase {
        1 << self.0.trailing_zeros()
    }

    pub fn of<T: Pointee<Metadata = ()> + AnyItem + ?Sized>(self) -> KeyPath<T> {
        KeyPath::new(self)
    }

    /// Leaves only common prefix.
    pub fn intersect(self, other: Self) -> Self {
        // Bits: 0 - same, 1 - diff
        let diff = self.0.get() ^ other.0.get();
        let same_bits = diff.leading_zeros();
        let level = same_bits.min(self.level()).min(other.level());
        Self::new(self.0.get(), level)
    }

    /// Iterates over children of given level, None if level is too high.
    pub fn iter_level(self, level: u32) -> Option<impl ExactSizeIterator<Item = Self>> {
        let level = level.min(INDEX_BASE_BITS.get() - 1);

        // Range of bottom bits
        let diff = level.checked_sub(self.level())?;
        let range = 0..(1usize.checked_shl(diff).expect("Level to low"));

        let top = self.top();
        let offset = INDEX_BASE_BITS.get() - level;
        Some(range.map(move |bottom| {
            let path = top | ((((bottom as IndexBase) << 1) | 1) << offset);
            Path(Index::new(path).expect("Shouldn't be zero"))
        }))
    }

    pub fn start_of_index(self, index: Index) -> bool {
        let diff = self.0.get() ^ index.get();
        let same_bits = diff.leading_zeros();
        self.level() <= same_bits
    }

    pub fn start_of_path(self, other: Self) -> bool {
        self.start_of_index(other.0)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = self.level();
        let path = self.0.get().rotate_left(level + 1) >> 1;
        write!(f, "{:0width$x}:{}", path, level, width = level as usize)
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
