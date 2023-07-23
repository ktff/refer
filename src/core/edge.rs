use std::ops::Not;

use super::{DynItem, Key, Ptr};

/// Sides of edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum Side {
    /// Edge source where edge data can be inlined.
    Source,
    /// Edge drain
    Drain,
    /// Bi directional.
    Bi,
}

impl Side {
    pub fn object<T>(self, object: T) -> PartialEdge<T> {
        PartialEdge {
            subject: self,
            object,
        }
    }
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::Source => Side::Drain,
            Side::Drain => Side::Source,
            Side::Bi => Side::Bi,
        }
    }
}

/// Source & Drain model for reference.
/// Edge = source[data] -> drain
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Edge<S, D> {
    pub source: S,
    pub drain: D,
}

/// Edge where one side is described with T while other by it's side type.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PartialEdge<T> {
    pub subject: Side,
    pub object: T,
}

impl<T> PartialEdge<T> {
    pub fn map<F>(self, map: impl FnOnce(T) -> F) -> PartialEdge<F> {
        PartialEdge {
            subject: self.subject,
            object: map(self.object),
        }
    }

    /// Current subject becomes new object.
    pub fn reverse<F>(self, object: F) -> (T, PartialEdge<F>) {
        (
            self.object,
            PartialEdge {
                subject: !self.subject,
                object,
            },
        )
    }
}

impl<P, T: DynItem + ?Sized> PartialEdge<Key<P, T>> {
    pub fn ptr(&self) -> PartialEdge<Key<Ptr, T>> {
        PartialEdge {
            subject: self.subject,
            object: self.object.ptr(),
        }
    }
}
