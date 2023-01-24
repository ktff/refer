use super::{LeafPath, LocalityPath, LocalityRegion, RegionPath};
use std::{fmt, hash::Hash, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalityKey(LeafPath);

impl LocalityKey {
    /// Constructors of LocalityKey should guarantee that all keys under LeafPath have the same Context.
    pub(crate) fn new(path: LeafPath) -> Self {
        Self(path)
    }
}

impl LocalityPath for LocalityKey {
    fn map(&self, region: RegionPath) -> Option<LocalityRegion> {
        self.path().map(region)
    }

    fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}

impl Deref for LocalityKey {
    type Target = LeafPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for LocalityKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:Context", self.path())
    }
}
