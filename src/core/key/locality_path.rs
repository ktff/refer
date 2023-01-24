use super::RegionPath;
use std::{fmt::Debug, ops::RangeInclusive};

pub trait LocalityPath: Debug {
    /// Maps self to LocalityRegion for given region.
    fn map(&self, region: RegionPath) -> Option<LocalityRegion>;
}

#[derive(Debug, Clone)]
pub enum LocalityRegion {
    /// Range of possible IDs
    Ids(RangeInclusive<usize>),
    /// Range of possible path indices
    Indices(RangeInclusive<usize>),
    /// Exactly this index
    Index(usize),
    Any,
}

impl LocalityRegion {
    pub fn contains(&self, index: usize) -> bool {
        match self {
            Self::Ids(range) => range.contains(&index),
            Self::Indices(range) => range.contains(&index),
            Self::Index(i) => index == *i,
            Self::Any => true,
        }
    }
}

impl LocalityPath for () {
    fn map(&self, _: RegionPath) -> Option<LocalityRegion> {
        Some(LocalityRegion::Any)
    }
}
