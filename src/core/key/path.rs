use super::*;
use crate::core::{AnyItem, Item};
use std::{
    num::{NonZeroU16, NonZeroU32, NonZeroUsize},
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

    /// None if len is too large.
    pub fn region(self, len: NonZeroU32) -> Option<PathRegion> {
        PathRegion::new(self, len)
    }

    pub fn leaf(self) -> PathLeaf {
        PathLeaf::new(self)
    }

    /// Leaves only common prefix.
    pub fn intersect(self, other: Self) -> Self {
        // Bits: 0 - same, 1 - diff
        let diff = self.0.get() ^ other.0.get();
        let same_bits = diff.leading_zeros();
        let level = same_bits.min(self.level()).min(other.level());
        Self::new_top(self.0.get(), level)
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

    pub fn includes_index(self, index: Index) -> bool {
        let diff = self.0.get() ^ index.get();
        let same_bits = diff.leading_zeros();
        self.level() <= same_bits
    }

    pub fn includes_path(self, other: Self) -> bool {
        if self.level() <= other.level() {
            self.includes_index(other.0)
        } else {
            false
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

/// Path end optimized for leaf containers.
/// Has path and non zero region extending from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathLeaf {
    /// Path of indices
    path: IndexBase,
    /// Level of path
    level: u32,

    remaining_len: NonZeroU32,
}

impl PathLeaf {
    pub fn new(path: Path) -> Self {
        Self {
            path: path.top(),
            level: path.level(),
            remaining_len: path.remaining_len(),
        }
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

    /// Panics if index is out of range.
    pub fn key_of<T: Item>(&self, index: NonZeroUsize) -> Key<T> {
        assert_eq!(
            index.get() >> self.remaining_len.get(),
            0,
            "Index has too many bits"
        );

        let index = self.path | (index.get() as IndexBase);
        // SAFETY: This is safe since argument `index` is NonZero and
        //         applying `or` operation with path will not result in zero.
        let index = unsafe { Index::new_unchecked(index) };
        Key::new(index)
    }

    /// May panic/return out of path index if key isn't of this path.
    pub fn index_of<T: Pointee + AnyItem + ?Sized>(&self, key: Key<T>) -> usize {
        // Xor is intentional so to catch keys that don't correspond to this path
        (key.index().get() ^ self.path) as usize
    }
}

/// Region of paths that starts at some path and has non zero length.
#[derive(Debug, Clone)]
pub struct PathRegion {
    /// Path of indices
    path: IndexBase,
    /// Path level
    level: u16,
    /// Region length
    len: NonZeroU16,
    /// Remaining sub region length
    remaining_len: NonZeroU16,
}

impl PathRegion {
    /// None if len is too large
    pub fn new(path: Path, len: NonZeroU32) -> Option<Self> {
        if len.get() > std::mem::size_of::<usize>() as u32 * 8 {
            return None;
        }

        let level = path.level();
        Some(PathRegion {
            path: path.top(),
            level: level as u16,
            len: NonZeroU16::new(len.get() as u16)?,
            remaining_len: NonZeroU16::new(
                INDEX_BASE_BITS.get().checked_sub(level + len.get())? as u16
            )?,
        })
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
    pub fn index_of<T: Pointee + AnyItem + ?Sized>(&self, key: Key<T>) -> usize {
        ((key.index().get() ^ self.path) >> self.remaining_len.get()) as usize
    }
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
    fn intersect() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b1100, 4);
        let path3 = Path::new_bottom(0b1, 1);
        assert_eq!(path1.intersect(path2), path3);
    }

    #[test]
    fn intersect_eq() {
        let path1 = Path::new_bottom(0b1010, 4);
        assert_eq!(path1.intersect(path1), path1);
    }

    #[test]
    fn intersect_empty() {
        let path1 = Path::new_bottom(0b1010, 4);
        let path2 = Path::new_bottom(0b0, 1);
        assert_eq!(path1.intersect(path2), Path::default());
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
        assert!(path1.includes_path(path2));
    }

    #[test]
    fn start_of_path_eq() {
        let path1 = Path::new_bottom(0b101, 3);
        assert!(path1.includes_path(path1));
    }

    #[test]
    fn start_of_path_false() {
        let path1 = Path::new_bottom(0b101, 3);
        let path2 = Path::new_bottom(0b1010, 4);
        let path3 = Path::new_bottom(0b1011, 4);
        assert!(!path2.includes_path(path1));
        assert!(!path3.includes_path(path1));
        assert!(!path3.includes_path(path2));
        assert!(!path2.includes_path(path3));
        assert!(path1.includes_path(path2));
        assert!(path1.includes_path(path3));
    }

    #[test]
    fn path_region() {
        let path = Path::new_bottom(0b1010, 4);
        let region = PathRegion::new(path, NonZeroU32::new(2).unwrap()).unwrap();
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
        let leaf = path.leaf();
        assert_eq!(leaf.path().bottom(), 0b100110);
        assert_eq!(leaf.remaining_len(), path.remaining_len());
    }

    #[test]
    fn key() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 4);
        let leaf = path.leaf();
        let key = leaf.key_of::<()>(NonZeroUsize::new(1).unwrap());
        assert!(path.includes_index(key.index()));
        assert_eq!(leaf.index_of(key), 1);
    }

    #[test]
    #[should_panic]
    fn key_index_to_large() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 4);
        let leaf = path.leaf();
        leaf.key_of::<()>(NonZeroUsize::new(0b10000).unwrap());
    }

    #[test]
    fn one_container_level() {
        let path = Path::new_bottom(0b100110, INDEX_BASE_BITS.get() - 8);
        let region = path.region(NonZeroU32::new(4).unwrap()).unwrap();
        let container_path = region.path_of(0b1000);
        let leaf = container_path.leaf();
        let key = leaf.key_of::<()>(NonZeroUsize::new(1).unwrap());

        assert!(container_path.includes_index(key.index()));
        assert_eq!(leaf.index_of(key), 1);
        assert!(path.includes_index(key.index()));
        assert_eq!(region.index_of(key), 0b1000);
    }
}
