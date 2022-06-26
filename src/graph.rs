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
pub trait Edge<'a>: Container + Eq + PartialEq + Clone + 'a {
    type V: Vertice<'a>;

    fn from(&self) -> Self::V;

    fn to(&self) -> Self::V;

    /// Edge going from `to` to `from`.
    fn opposite(&self) -> Self;
}

pub trait Memory {
    fn iter<'a, S: Structure<'a>>(&'a self) -> Box<dyn Iterator<Item = S>>;
}

// ! Owner je jedini koji može imati uni directional reference, drugi moraju imati bi directional reference.
// ! Owner može imati i bi directional reference.
// ! Samo jedna struktura može biti vlasnik druge. Memory je vlasnik root struktura.

pub trait Structure<'a>: Container + Clone + 'a {
    // type Iter: Iterator<Item = Self::R> + 'a;
    // type R: Reference<'a>;

    // /// Edges from this vertice.
    // fn edges(&self) -> Self::Iter;

    /// Name is an structure level unique property.
    fn field<R: Reference<'a>>(&self, name: &str) -> Option<R>;

    fn iter<R: Reference<'a>>(&self) -> dyn Iterator<Item = R>;

    // fn field<R: Reference<'a>>(&self, name: &str) -> Option<R>;
}

// EDGE
pub trait Reference<'a>: Container + Clone + 'a {
    type T: Structure<'a>;

    fn to(&self) -> Self::T;
}
