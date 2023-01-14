use bitvec::macros::internal::funty::Integral;

use super::*;
use crate::core::{AnyItem, Item};
use std::{
    cmp::Ordering,
    num::{NonZeroU16, NonZeroU32, NonZeroUsize},
    ops::RangeInclusive,
    ptr::Pointee,
};

/// Path in container
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Path(Index);

impl Path {
    /// Constructs from level top bits.
    /// Will increase the level of the path if it's key.
    pub fn new_top(path: IndexBase, level: u32) -> Self {
        let offset = (INDEX_BASE_BITS.get() - 1).saturating_sub(level);
        let path = ((path >> offset) | 1) << offset;
        Path(Index::new(path).expect("Shouldn't be zero"))
    }

    /// Constructs from level bottom bits.
    /// Will increase the level of the path if it's key.
    pub fn new_bottom(path: IndexBase, level: u32) -> Self {
        Self::new_top(path.rotate_right(level), level)
    }

    pub fn level(self) -> u32 {
        (INDEX_BASE_BITS.get() - 1) - self.0.trailing_zeros()
    }

    pub fn top(self) -> IndexBase {
        self.0.get() ^ self.bit()
    }

    #[cfg(test)]
    fn bottom(self) -> IndexBase {
        self.top().rotate_left(self.level())
    }

    fn bit(self) -> IndexBase {
        1 << self.0.trailing_zeros()
    }

    /// Remaining bits for index
    pub fn remaining_len(self) -> NonZeroU32 {
        NonZeroU32::new(INDEX_BASE_BITS.get() - self.level())
            .expect("Should have at least one bit left")
    }

    pub fn of<T: Pointee<Metadata = ()> + AnyItem + ?Sized>(self) -> KeyPath<T> {
        KeyPath::new(self)
    }

    /// None if len is larger than size of usize.
    pub fn region(self, len: NonZeroU32) -> Option<RegionPath> {
        RegionPath::new(self, len)
    }

    /// None if remaining len is larger than size of usize.
    pub fn leaf(self) -> Option<LeafPath> {
        LeafPath::new(self)
    }

    /// Returns longest path that covers both paths.
    pub fn or(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        // Bits: 0 - same, 1 - diff
        let diff = self.0.get() ^ other.0.get();
        let same_bits = diff.leading_zeros();
        let level = same_bits.min(self.level()).min(other.level());
        Self::new_top(self.0.get(), level)
    }

    /// Returns shortest path covered by both paths.
    pub fn and(self, other: impl Into<Self>) -> Option<Self> {
        let other = other.into();
        match self.level().cmp(&other.level()) {
            Ordering::Less if self.contains_index(other.0) => Some(other),
            Ordering::Equal if self == other => Some(self),
            Ordering::Greater if other.contains_index(self.0) => Some(self),
            _ => None,
        }
    }

    /// Iterates over children of given level, None if level is too high.
    pub fn iter_level(self, level: u32) -> Option<impl ExactSizeIterator<Item = Self>> {
        let level = level.min(INDEX_BASE_BITS.get() - 1);

        // Range of bottom bits
        let diff = level.checked_sub(self.level())?;
        let range = 0..(1usize.checked_shl(diff).expect("Level to low"));

        let top = self.top();
        let offset = INDEX_BASE_BITS.get() - 1 - level;
        Some(range.map(move |bottom| {
            let path = top | ((((bottom as IndexBase) << 1) | 1) << offset);
            Path(Index::new(path).expect("Shouldn't be zero"))
        }))
    }

    pub fn contains_index(self, index: Index) -> bool {
        let diff = self.0.get() ^ index.get();
        let same_bits = diff.leading_zeros();
        self.level() <= same_bits
    }

    pub fn contains(self, other: impl Into<Self>) -> bool {
        let other = other.into();
        if self.level() <= other.level() {
            self.contains_index(other.0)
        } else {
            false
        }
    }

    /// Remaining range of indices under this path.
    /// Err if range is too large for usize.
    pub fn remaining_range(self) -> Result<RangeInclusive<usize>, ()> {
        let level = self.level();
        match level.cmp(&(std::mem::size_of::<usize>() as u32 * 8)) {
            Ordering::Less => {
                let remaining_len = INDEX_BASE_BITS.get() - level;
                let end = (1 << remaining_len) - 1;
                Ok(0..=end)
            }
            Ordering::Equal => Ok(0..=usize::MAX),
            Ordering::Greater => Err(()),
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = self.level();
        let path = self.0.get().rotate_left(level + 1) >> 1;
        write!(
            f,
            "{:0width$x}:{}",
            path,
            level,
            width = ((level + 3) / 4) as usize
        )
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new_bottom(0, 0)
    }
}

impl<T: Pointee + AnyItem + ?Sized> From<KeyPath<T>> for Path {
    fn from(path: KeyPath<T>) -> Self {
        path.path()
    }
}

/// Path end optimized for leaf containers.
/// Has path and non zero region extending from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LeafPath {
    /// Path of indices
    path: IndexBase,
    /// Level of path
    level: u32,

    remaining_len: NonZeroU32,
}

impl LeafPath {
    /// None if remaining_len is larger than size of usize.
    pub fn new(path: Path) -> Option<Self> {
        if path.remaining_len().get() > std::mem::size_of::<usize>() as u32 * 8 {
            return None;
        }

        Some(Self {
            path: path.top(),
            level: path.level(),
            remaining_len: path.remaining_len(),
        })
    }

    pub fn path(&self) -> Path {
        Path::new_top(self.path, self.level)
    }

    pub fn level(self) -> u32 {
        self.level
    }

    pub fn top(self) -> IndexBase {
        self.path
    }

    /// Remaining bits for index
    pub fn remaining_len(&self) -> NonZeroU32 {
        self.remaining_len
    }

    pub fn contains(&self, index: NonZeroUsize) -> bool {
        index
            .get()
            .checked_shr(self.remaining_len.get())
            .unwrap_or(0)
            == 0
    }

    /// Panics if index is out of range.
    #[inline(always)]
    pub fn key_of<T: Item>(&self, index: NonZeroUsize) -> Key<T> {
        assert!(self.contains(index), "Index has too many bits");

        let index = self.path | (index.get() as IndexBase);
        // SAFETY: This is safe since argument `index` is NonZero and
        //         applying `or` operation with path will not result in zero.
        let index = unsafe { Index::new_unchecked(index) };
        Key::new(index)
    }

    /// May panic/return out of path index if key isn't of this path.
    #[inline(always)]
    pub fn index_of<T: Pointee + AnyItem + ?Sized>(&self, key: Key<T>) -> usize {
        // Xor is intentional so to catch keys that don't correspond to this path
        (key.index().get() ^ self.path) as usize
    }

    /// Returns range of indices that are included in this path.
    pub fn range_of(&self, path: impl Into<Path>) -> Option<RangeInclusive<usize>> {
        let path = self.path().and(path)?;
        let start = path.top() as usize & usize_mask(self.remaining_len.get());
        let range = path
            .remaining_range()
            .expect("Should be less or equal to size of usize");
        Some((start + *range.start())..=(start + *range.end()))
    }
}

/// Region of paths that starts at some path and has non zero length.
#[derive(Debug, Clone)]
pub struct RegionPath {
    /// Path of indices
    path: IndexBase,
    /// Path level
    level: u16,
    /// Region length
    len: NonZeroU16,
    /// Remaining sub region length
    remaining_len: NonZeroU16,
}

impl RegionPath {
    /// None if len is too large
    pub fn new(path: Path, len: NonZeroU32) -> Option<Self> {
        if len.get() > std::mem::size_of::<usize>() as u32 * 8 {
            return None;
        }

        let level = path.level();
        Some(RegionPath {
            path: path.top(),
            level: level as u16,
            len: NonZeroU16::new(len.get() as u16)?,
            remaining_len: NonZeroU16::new(
                INDEX_BASE_BITS.get().checked_sub(level + len.get())? as u16
            )?,
        })
    }

    pub fn path(&self) -> Path {
        Path::new_top(self.path, self.level as u32)
    }

    /// Panics if index is out of range.
    pub fn path_of(&self, index: usize) -> Path {
        // Constructed by adding index at the end of the path
        assert!(
            index
                .checked_shr(self.len.get() as u32)
                .map(|r| r == 0)
                .unwrap_or(true),
            "Index has too many bits"
        );

        Path::new_top(
            self.path | ((index as IndexBase) << self.remaining_len.get()),
            (self.level + self.len.get()) as u32,
        )
    }

    /// May panic/return invalid index if key isn't of this path.
    #[inline(always)]
    pub fn index_of<T: Pointee + AnyItem + ?Sized>(&self, key: Key<T>) -> usize {
        ((key.index().get() ^ self.path) >> self.remaining_len.get()) as usize
    }

    /// Returns range of indices that are included in this path.
    pub fn range_of(&self, path: impl Into<Path>) -> Option<RangeInclusive<usize>> {
        let path = self.path().and(path)?;
        let start =
            (path.top() >> self.remaining_len.get()) as usize & usize_mask(self.len.get() as u32);
        let bottom_len = path
            .remaining_len()
            .get()
            .saturating_sub(self.remaining_len.get() as u32);
        let end = start + usize_mask(bottom_len as u32);
        Some(start..=end)
    }
}

fn usize_mask(len: u32) -> usize {
    1.checked_shl(len)
        .map(|mask| mask - 1)
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bottom() {
        let path = Path::new_bottom(0b1010, 4);
        assert_eq!(path.level(), 4);
        assert_eq!(path.bottom(), 0b1010);
        assert_eq!(path.remaining_len().get(), INDEX_BASE_BITS.get() - 4);
        assert_eq!(format!("{}", path), "a:4");
    }

    #[test]
    fn new_top() {
        let path = Path::new_top((0b1010usize as IndexBase).rotate_right(4), 4);
        assert_eq!(path.level(), 4);
        assert_eq!(path.top().rotate_left(4), 0b1010);
        assert_eq!(path.bottom(), 0b1010);
        assert_eq!(format!("{}", path), "a:4");
    }

    #[test]
    fn new_full() {
        let path = Path::new_bottom(0b1010, INDEX_BASE_BITS.get());
        assert_eq!(path.level(), INDEX_BASE_BITS.get() - 1);
        assert_eq!(path.bottom(), 0b101);
    }

    #[test]
    fn or() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b1100, 4);
        let path3 = Path::new_bottom(0b1, 1);
        assert_eq!(path1.or(path2), path3);
    }

    #[test]
    fn or_eq() {
        let path1 = Path::new_bottom(0b1010, 4);
        assert_eq!(path1.or(path1), path1);
    }

    #[test]
    fn or_empty() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b0, 1);
        assert_eq!(path1.or(path2), Path::default());
    }

    #[test]
    fn and_equal() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b1010, 4);
        assert_eq!(path1.and(path2), Some(path1));
    }

    #[test]
    fn and_non_equal() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b101, 3);
        assert_eq!(path1.and(path2), Some(path1));
        assert_eq!(path2.and(path1), Some(path1));
    }

    #[test]
    fn and_empty() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b0, 1);
        assert_eq!(path1.and(path2), None);
        assert_eq!(path2.and(path1), None);
    }

    #[test]
    fn iter_level() {
        let path = Path::new_bottom(0b1010, 4);
        let mut iter = path.iter_level(6).unwrap();
        assert_eq!(iter.next(), Some(Path::new_bottom(0b101000, 6)));
        assert_eq!(iter.next(), Some(Path::new_bottom(0b101001, 6)));
        assert_eq!(iter.next(), Some(Path::new_bottom(0b101010, 6)));
        assert_eq!(iter.next(), Some(Path::new_bottom(0b101011, 6)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_level_empty() {
        let path = Path::new_bottom(0b1010, 4);
        assert!(path.iter_level(3).is_none());
    }

    #[test]
    fn start_of_path() {
        let path1 = Path::new_bottom(0b101, 3);
        let path2 = Path::new_bottom(0b1010, 4);
        assert!(path1.contains(path2));
    }

    #[test]
    fn start_of_path_eq() {
        let path1 = Path::new_bottom(0b101, 3);
        assert!(path1.contains(path1));
    }

    #[test]
    fn start_of_path_false() {
        let path1 = Path::new_bottom(0b101, 3);
        let path2 = Path::new_bottom(0b1010, 4);
        let path3 = Path::new_bottom(0b1011, 4);
        assert!(!path2.contains(path1));
        assert!(!path3.contains(path1));
        assert!(!path3.contains(path2));
        assert!(!path2.contains(path3));
        assert!(path1.contains(path2));
        assert!(path1.contains(path3));
    }

    #[test]
    fn path_region() {
        let path = Path::new_bottom(0b1010, 4);
        let region = RegionPath::new(path, NonZeroU32::new(2).unwrap()).unwrap();
        assert_eq!(region.path_of(0b00), Path::new_bottom(0b101000, 6));
        assert_eq!(region.path_of(0b01), Path::new_bottom(0b101001, 6));
        assert_eq!(region.path_of(0b10), Path::new_bottom(0b101010, 6));
        assert_eq!(region.path_of(0b11), Path::new_bottom(0b101011, 6));
    }

    #[test]
    #[should_panic]
    fn index_to_large() {
        Path::new_bottom(0b1010, 4)
            .region(NonZeroU32::new(3).unwrap())
            .unwrap()
            .path_of(0b1000);
    }

    #[test]
    fn test_leaf() {
        let path = Path::new_bottom(0b100110, 6);
        let leaf = path.leaf().unwrap();
        assert_eq!(leaf.path().bottom(), 0b100110);
        assert_eq!(leaf.remaining_len(), path.remaining_len());
    }

    #[test]
    fn key() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 4);
        let leaf = path.leaf().unwrap();
        let key = leaf.key_of::<()>(NonZeroUsize::new(1).unwrap());
        assert!(path.contains_index(key.index()));
        assert_eq!(leaf.index_of(key), 1);
    }

    #[test]
    #[should_panic]
    fn key_index_to_large() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 4);
        let leaf = path.leaf().unwrap();
        leaf.key_of::<()>(NonZeroUsize::new(0b10000).unwrap());
    }

    #[test]
    fn one_container_level() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(4).unwrap()).unwrap();
        let container_path = region.path_of(0b1000);
        let leaf = container_path.leaf().unwrap();
        let key = leaf.key_of::<()>(NonZeroUsize::new(1).unwrap());

        assert!(container_path.contains_index(key.index()));
        assert_eq!(leaf.index_of(key), 1);
        assert!(path.contains_index(key.index()));
        assert_eq!(region.index_of(key), 0b1000);
    }

    #[test]
    fn leaf_range_of_default() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let leaf = path.leaf().unwrap();

        assert_eq!(leaf.range_of(Path::default()), Some(0..=255));
    }

    #[test]
    fn leaf_range_of_self() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let leaf = path.leaf().unwrap();

        assert_eq!(leaf.range_of(path), Some(0..=255));
    }

    #[test]
    fn leaf_range_of() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let leaf = path.leaf().unwrap();
        let sub = path
            .region(NonZeroU32::new(2).unwrap())
            .unwrap()
            .path_of(0b10);

        assert_eq!(leaf.range_of(sub), Some(128..=(128 + 64 - 1)));
    }

    #[test]
    fn leaf_range_of_empty() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let leaf = path.leaf().unwrap();
        let other = Path::new_bottom(0b100111, INDEX_BASE_BITS.get() - 8);

        assert_eq!(leaf.range_of(other), None);
    }

    #[test]
    fn region_range_of_default() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(2).unwrap()).unwrap();

        assert_eq!(region.range_of(Path::default()), Some(0..=3));
    }

    #[test]
    fn region_range_of_self() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(2).unwrap()).unwrap();

        assert_eq!(region.range_of(path), Some(0..=3));
    }

    #[test]
    fn region_range_of() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(2).unwrap()).unwrap();
        let sub = path
            .region(NonZeroU32::new(2).unwrap())
            .unwrap()
            .path_of(0b10);

        assert_eq!(region.range_of(sub), Some(2..=2));
    }

    #[test]
    fn region_range_of_sub() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(2).unwrap()).unwrap();
        let sub = path
            .region(NonZeroU32::new(3).unwrap())
            .unwrap()
            .path_of(0b101);

        assert_eq!(region.range_of(sub), Some(2..=2));
    }

    #[test]
    fn region_range_of_empty() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(2).unwrap()).unwrap();
        let other = Path::new_bottom(0b100111, INDEX_BASE_BITS.get() - 8);

        assert_eq!(region.range_of(other), None);
    }
}
