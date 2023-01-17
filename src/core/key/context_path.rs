use super::{Key, Path};
use crate::core::DynItem;
use std::{
    fmt::{self},
    hash::Hash,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextPath(Path);

impl ContextPath {
    /// Constructors of ContextPath should guarantee that all keys under Path have the same Context.
    pub fn new(path: Path) -> Self {
        Self(path)
    }
}

impl ContextPath {
    #[inline(always)]
    pub fn path(&self) -> Path {
        self.0
    }

    /// Returns shortest path covered by both paths.
    pub fn and(self, other: impl Into<Path>) -> Option<Self> {
        self.0.and(other).map(|path| Self(path))
    }

    /// Iterates over children of given level, None if level is too high.
    pub fn iter_level(self, level: u32) -> Option<impl ExactSizeIterator<Item = Self>> {
        self.0
            .iter_level(level)
            .map(|iter| iter.map(move |path| Self(path)))
    }

    /// True if self start of given path.
    pub fn contains(self, path: ContextPath) -> bool {
        self.0.contains(path.0)
    }

    /// True if self start of given key.
    pub fn contains_key<T: DynItem + ?Sized>(self, key: Key<T>) -> bool {
        self.0.contains_index(key.index())
    }
}

impl fmt::Display for ContextPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:Context", self.0)
    }
}
