use super::RegionPath;
use std::{any::TypeId, fmt::Debug, ops::RangeInclusive};

pub trait LocalityPath: Debug {
    /// Maps self to LocalityRegion for given region.
    fn map(&self, region: RegionPath) -> Option<LocalityRegion>;

    fn upcast(&self) -> &dyn LocalityPath;
}

#[derive(Debug, Clone)]
pub enum LocalityRegion {
    /// Id tied to given type.
    Id((TypeId, usize)),
    /// Range of possible path indices
    Indices(RangeInclusive<usize>),
    /// Exactly this index
    Index(usize),
    Any,
}

// impl LocalityRegion {
//     pub fn contains(&self, index: usize) -> bool {
//         match self {
//             Self::Id(id) => range.contains(&index),
//             Self::Indices(range) => range.contains(&index),
//             Self::Index(i) => index == *i,
//             Self::Any => true,
//         }
//     }
// }

impl LocalityPath for () {
    fn map(&self, _: RegionPath) -> Option<LocalityRegion> {
        Some(LocalityRegion::Any)
    }

    fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}

impl<L: LocalityPath> LocalityPath for [L; 2] {
    fn map(&self, region: RegionPath) -> Option<LocalityRegion> {
        self[0].map(region).or_else(|| self[1].map(region))
    }

    fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}
