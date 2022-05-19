use crate::property::Container;

pub trait Graph {
    type V<'a>: Vertice<'a, E = Self::E<'a>>;
    type E<'a>: Edge<'a, V = Self::V<'a>>;

    type VerticeIter<'a>: Iterator<Item = Self::V<'a>>
    where
        Self: 'a;

    fn vertices<'a>(&'a self) -> Self::VerticeIter<'a>;
}

pub trait Vertice<'a>: Container + Clone + 'a {
    type Iter: Iterator<Item = Self::E> + 'a;
    type E: Edge<'a, V = Self>;

    /// Edges from this vertice.
    fn edges(&self) -> Self::Iter;
}

/// Edge from vertice to vertice with properties.
/// Edge is directional, so opposite edge has different properties.
pub trait Edge<'a>: Container + Clone + 'a {
    type V: Vertice<'a>;

    fn from(&self) -> Self::V;

    fn to(&self) -> Self::V;

    /// Edge going from `to` to `from`.
    fn opposite(&self) -> Self;
}
