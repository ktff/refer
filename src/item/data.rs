use crate::core::*;
use std::ops::{Deref, DerefMut};

pub struct Data<T: Sync + Send + 'static>(T);

impl<T: Sync + Send + 'static> Item for Data<T> {
    type I<'a> = std::iter::Empty<AnyRef>;

    fn iter_references(&self, _: Index) -> Self::I<'_> {
        std::iter::empty()
    }
}

impl<T: Sync + Send + 'static> AnyItem for Data<T> {
    fn iter_references_any<'a>(
        &'a self,
        _: Index,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        None
    }

    fn remove_reference(&mut self, _: Index, _: AnyKey) -> bool {
        true
    }

    fn set_reference(&mut self, _: AnyKey, _: AnyKey) {}
}

impl<T: Sync + Send + 'static> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Sync + Send + 'static> DerefMut for Data<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
