use std::{cell::UnsafeCell, num::NonZeroU64};

use crate::core::*;

// Chunking can be done according to one of two points:
// a) Connection
// b) Data

pub type ChunkedCellIter<'a, C: Container<T>, T: AnyItem + ?Sized> =
    impl Iterator<Item = (SubKey<T>, &'a UnsafeCell<T>, &'a UnsafeCell<C::Shell>)> + 'a;

/// Container which groups items into chunks.
pub struct ChunkedContainer<C: AnyContainer + KeyContainer> {
    chunks: Vec<Chunk<C>>,
    key_len: u32,
}

impl<C: AnyContainer + KeyContainer> ChunkedContainer<C> {
    fn get_chunk(&self, prefix: Index) -> Option<&Chunk<C>> {
        // Can't underflow since prefix is > 0
        let i = prefix.0.get() as usize - 1;
        self.chunks.get(i)
    }

    fn get_mut_chunk(&mut self, prefix: Index) -> Option<&mut Chunk<C>> {
        // Can't underflow since prefix is > 0
        let i = prefix.0.get() as usize - 1;
        self.chunks.get_mut(i)
    }

    fn first_since<I: AnyItem>(&self, chunk_i: usize) -> Option<SubKey<I>> {
        self.chunks
            .iter()
            .enumerate()
            .skip(chunk_i)
            .filter_map(|(i, chunk)| chunk.container.first::<I>().map(|key| (i, key)))
            .next()
            .map(|(chunk_i, suffix)| {
                suffix.with_prefix(
                    self.key_len,
                    Index(NonZeroU64::new(chunk_i as u64 + 1).expect("Shouldn't be zero")),
                )
            })
    }
}

// NOTE: Prefix should be specified at build.
// NOTE: Selection logic should be passed in for each type.

impl<T: AnyItem + ?Sized, C: AnyContainer + KeyContainer> Allocator<T> for ChunkedContainer<C>
where
    C: Allocator<T>,
{
    /// Reserves slot for item.
    /// None if collection is out of keys or memory.
    fn reserve(&mut self, item: &T) -> Option<ReservedKey<T>> {
        unimplemented!()
    }

    /// Cancels reservation for item.
    /// Panics if there is no reservation.
    fn cancel(&mut self, key: ReservedKey<T>) {
        let (prefix, suffix) = key.split_prefix(self.key_len);
        self.get_mut_chunk(prefix)
            .expect("Invalid key")
            .container
            .cancel(suffix);
    }

    /// Fulfills reservation.
    /// Panics if there is no reservation.
    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        let (prefix, suffix) = key.split_prefix(self.key_len);
        self.get_mut_chunk(prefix)
            .expect("Invalid key")
            .container
            .fulfill(suffix, item)
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        let (prefix, suffix) = key.split_prefix(self.key_len);
        self.get_mut_chunk(prefix)
            .and_then(|chunk| chunk.container.unfill(suffix))
    }
}

impl<T: AnyItem + ?Sized, C: AnyContainer + KeyContainer> Container<T> for ChunkedContainer<C>
where
    C: Container<T>,
{
    type Shell = C::Shell;

    type CellIter<'a> = ChunkedCellIter<'a,C,T>
    where
        Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        let (prefix, suffix) = key.split_prefix(self.key_len);
        self.get_chunk(prefix)
            .and_then(|chunk| chunk.container.get_slot(suffix))
    }

    /// Even if some it may be empty.
    fn iter_slot(&self) -> Option<Self::CellIter<'_>> {
        let key_len = self.key_len;
        Some(
            self.chunks
                .iter()
                .enumerate()
                .filter_map(move |(i, chunk)| {
                    let prefix = Index(NonZeroU64::new(i as u64 + 1).expect("Shouldn't be zero"));
                    chunk.container.iter_slot().map(|iter| {
                        iter.map(move |(suffix, item, shell)| {
                            (suffix.with_prefix(key_len, prefix), item, shell)
                        })
                    })
                })
                .flat_map(|iter| iter),
        )
    }
}

impl<C: AnyContainer + KeyContainer> AnyContainer for ChunkedContainer<C> {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        let (prefix, suffix) = key.split_prefix(self.key_len);
        self.get_chunk(prefix)?.container.any_get_slot(suffix)
    }

    /// Frees if it exists.
    fn any_unfill(&mut self, key: AnySubKey) -> bool {
        let (prefix, suffix) = key.split_prefix(self.key_len);
        self.get_mut_chunk(prefix)
            .map_or(false, |chunk| chunk.container.any_unfill(suffix))
    }
}

impl<C: AnyContainer + KeyContainer> KeyContainer for ChunkedContainer<C> {
    fn first<I: AnyItem>(&self) -> Option<SubKey<I>> {
        self.first_since(0)
    }

    fn next<I: AnyItem>(&self, key: SubKey<I>) -> Option<SubKey<I>> {
        let (prefix, suffix) = key.split_prefix(self.key_len);

        if let Some(suffix) = self.get_chunk(prefix)?.container.next(suffix) {
            Some(suffix.with_prefix(self.key_len, prefix))
        } else {
            // Shortened from prefix.0.get() = (prefix.0.get() -1) + 1
            self.first_since(prefix.0.get() as usize)
        }
    }
}

struct Chunk<C: AnyContainer> {
    container: C,
    // items: usize,
}
