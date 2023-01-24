use super::RegionPath;
use std::{
    any::{Any, TypeId},
    fmt::Debug,
    ops::RangeInclusive,
};

pub trait LocalityPath: Debug + Any {
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
