use std::{any::TypeId, cell::UnsafeCell, collections::HashSet};

use crate::core::*;

pub type SlotIter<'a, L: ChunkingLogic, T: AnyItem>
where
    L::C: Container<T>,
= impl Iterator<
    Item = (
        SubKey<T>,
        &'a UnsafeCell<T>,
        &'a UnsafeCell<<<L as ChunkingLogic>::C as Container<T>>::Shell>,
    ),
>;

pub trait ChunkingLogic: 'static {
    type C: 'static;

    /// Assign item to some chunk.
    fn assign<T: AnyItem>(&mut self, chunks: &mut Vec<Self::C>, item: &T) -> Option<usize>
    where
        Self::C: Allocator<T>;

    fn key_len(&self) -> u32;
}

/// A chunked container.
pub struct Chunked<L: ChunkingLogic> {
    logic: L,
    chunks: Vec<L::C>,
}

impl<L: ChunkingLogic> Chunked<L> {
    pub fn new(logic: L) -> Self {
        Self {
            logic,
            chunks: Vec::new(),
        }
    }
}

impl<L: ChunkingLogic, T: AnyItem> Allocator<T> for Chunked<L>
where
    L::C: Allocator<T>,
{
    fn reserve(&mut self, item: &T) -> Option<ReservedKey<T>> {
        let index = self.logic.assign(&mut self.chunks, item)?;
        let sub_key = self.chunks[index].reserve(item)?;
        Some(sub_key.with_prefix(self.logic.key_len(), index))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let (index, sub_key) = key.split_prefix(self.logic.key_len());
        self.chunks[index].cancel(sub_key);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        let (index, sub_key) = key.split_prefix(self.logic.key_len());
        self.chunks[index].fulfill(sub_key, item)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        let (index, sub_key) = key.split_prefix(self.logic.key_len());
        self.chunks[index].unfill(sub_key)
    }
}

impl<L: ChunkingLogic> !Sync for Chunked<L> {}

impl<L: ChunkingLogic, T: AnyItem> Container<T> for Chunked<L>
where
    L::C: Container<T>,
{
    type Shell = <L::C as Container<T>>::Shell;

    type SlotIter<'a> = SlotIter<'a, L, T>
    where
        Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        let (prefix, suffix) = key.split_prefix(self.logic.key_len());
        self.chunks.get(prefix)?.get_slot(suffix)
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        let key_len = self.logic.key_len();
        Some(
            self.chunks
                .iter()
                .enumerate()
                .flat_map(move |(prefix, chunk)| {
                    chunk.iter_slot().map(|iter| {
                        iter.map(move |(suffix, v, s)| (suffix.with_prefix(key_len, prefix), v, s))
                    })
                })
                .flat_map(|iter| iter),
        )
    }
}

impl<L: ChunkingLogic> AnyContainer for Chunked<L>
where
    L::C: AnyContainer,
{
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        let (prefix, suffix) = key.split_prefix(self.logic.key_len());
        self.chunks.get(prefix)?.any_get_slot(suffix)
    }

    fn any_unfill(&mut self, key: AnySubKey) -> bool {
        let (prefix, suffix) = key.split_prefix(self.logic.key_len());
        self.chunks
            .get_mut(prefix)
            .map_or(false, |chunk| chunk.any_unfill(suffix))
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        self.chunks.iter().enumerate().find_map(|(prefix, chunk)| {
            chunk
                .first(key)
                .map(|suffix| suffix.with_prefix(self.logic.key_len(), prefix))
        })
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        let (prefix, suffix) = key.split_prefix(self.logic.key_len());
        let chunk = self.chunks.get(prefix)?;
        if let Some(suffix) = chunk.next(suffix) {
            Some(suffix.with_prefix(self.logic.key_len(), prefix))
        } else {
            self.chunks
                .iter()
                .enumerate()
                .skip(prefix + 1)
                .find_map(|(prefix, chunk)| {
                    chunk
                        .first(key.type_id())
                        .map(|suffix| suffix.with_prefix(self.logic.key_len(), prefix))
                })
        }
    }

    fn types(&self) -> HashSet<TypeId> {
        self.chunks.iter().flat_map(|chunk| chunk.types()).collect()
    }
}
