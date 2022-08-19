use crate::core::*;

/// T --> T
pub struct Vertice<T: ?Sized, D: Directioned = Bi>(Vec<Ref<T, Global, D>>);

impl<T: ?Sized, D: Directioned> Vertice<T, D> {
    pub fn new() -> Self {
        Vertice(Vec::new())
    }

    pub fn add(to: Key<T>) {
        unimplemented!()
    }
}
