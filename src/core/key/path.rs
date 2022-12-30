use crate::core::{AnyItem, Index};

use super::{IndexBase, KeyPath, INDEX_BASE_BITS};
use std::ptr::Pointee;

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

    pub fn level(&self) -> u32 {
        (INDEX_BASE_BITS.get() - 1) - self.0.trailing_zeros()
    }

    fn top(&self) -> IndexBase {
        self.0.get() ^ self.bit()
    }

    #[cfg(test)]
    fn bottom(&self) -> IndexBase {
        self.top().rotate_left(self.level())
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

    pub fn start_of_index(self, index: Index) -> bool {
        let diff = self.0.get() ^ index.get();
        let same_bits = diff.leading_zeros();
        self.level() <= same_bits
    }

    pub fn start_of_path(self, other: Self) -> bool {
        if self.level() <= other.level() {
            self.start_of_index(other.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bottom() {
        let path = Path::new_bottom(0b1010, 4);
        assert_eq!(path.level(), 4);
        assert_eq!(path.bottom(), 0b1010);
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
        assert!(path1.start_of_path(path2));
    }

    #[test]
    fn start_of_path_eq() {
        let path1 = Path::new_bottom(0b101, 3);
        assert!(path1.start_of_path(path1));
    }

    #[test]
    fn start_of_path_false() {
        let path1 = Path::new_bottom(0b101, 3);
        let path2 = Path::new_bottom(0b1010, 4);
        let path3 = Path::new_bottom(0b1011, 4);
        assert!(!path2.start_of_path(path1));
        assert!(!path3.start_of_path(path1));
        assert!(!path3.start_of_path(path2));
        assert!(!path2.start_of_path(path3));
        assert!(path1.start_of_path(path2));
        assert!(path1.start_of_path(path3));
    }
}
