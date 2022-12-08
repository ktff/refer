use crate::core::{Allocator, AnyItem, Container, ReservedKey, Shell, SubKey, UnsafeSlot};
use std::{any::Any, ops::Range};

// TODO: Revisit this

pub trait DisperseLayer<T: AnyItem>: Send + Sync {
    type R: Copy;

    type Inner: Container<T>;

    type IterInner<'a>: Iterator<Item = (usize, &'a Self::Inner)>
    where
        Self: 'a;

    type IterInnerMut<'a>: Iterator<Item = (usize, &'a mut Self::Inner)>
    where
        Self: 'a;

    fn get_inner(&self, index: usize) -> Option<&Self::Inner>;

    fn get_inner_mut(&mut self, index: usize) -> Option<&mut Self::Inner>;

    fn iter_inner(&self) -> Self::IterInner<'_>;

    fn range_inner(&self) -> Range<usize>;

    fn key_len(&self) -> u32;

    fn assign(
        &mut self,
        item: Option<&T>,
        r: Self::R,
    ) -> Option<(usize, <Self::Inner as Allocator<T>>::R)>;
}

pub struct Layer<L>(L);

impl<T, L: DisperseLayer<T>> Container<T> for Layer<L> {}
