use super::*;
use crate::core::{AnyContainer, AnyItem, AnyPermit, AnySlot, IndexBase, Item};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    ptr::Pointee,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerPath {
    /// Path of indices
    path: IndexBase,
    /// Number of top bits used by the path
    bit_len: u16,
}

impl ContainerPath {
    pub fn new() -> Self {
        Self {
            path: 0,
            bit_len: 0,
        }
    }

    pub fn path(&self) -> Path {
        Path::new(self.path, self.bit_len as u32)
    }

    pub fn region(self, bit_len: NonZeroU16) -> ContainerRegion {
        assert!(
            bit_len.get() <= std::mem::size_of::<usize>() as u16 * 8,
            "Index must fit in usize"
        );

        ContainerRegion {
            path: self.path,
            bit_len: self.bit_len,
            index_bit_len: bit_len,
            index_off: NonZeroU16::new(
                (INDEX_BASE_BITS.get() as u16)
                    .checked_sub(self.bit_len + bit_len.get())
                    .expect("Should have at least one bit left"),
            )
            .expect("Should have at least one bit left"),
        }
    }

    /// Remaining bits for index
    pub fn index_bit_len(&self) -> NonZeroU16 {
        NonZeroU16::new((INDEX_BASE_BITS.get() as u16) - self.bit_len)
            .expect("Should have at least one bit left")
    }

    pub fn with_index<T: Item>(&self, index: NonZeroUsize) -> Key<T> {
        debug_assert!(
            index.get() >> self.index_bit_len().get() == 0,
            "Index has too many bits"
        );

        let index = self.path | (index.get() as IndexBase);
        // SAFETY: This is safe since argument `index` is NonZero and
        //         applying `or` operation with path will not result in zero.
        let index = unsafe { Index::new_unchecked(index) };
        Key::new(index)
    }

    pub fn index<T: Pointee + AnyItem + ?Sized>(&self, key: Key<T>) -> NonZeroUsize {
        let key: Index = key.index();
        let index = key.get() ^ self.path;
        NonZeroUsize::new(index as usize).expect("Index must be non zero")
    }
}

impl Default for ContainerPath {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ContainerRegion {
    /// Path of indices
    path: IndexBase,
    /// Number of top bits used by the path
    bit_len: u16,
    /// Bits used by this container
    index_bit_len: NonZeroU16,
    index_off: NonZeroU16,
}

impl ContainerRegion {
    pub fn with_path(&self, index: usize) -> ContainerPath {
        // Constructed by adding index at the end of the path
        assert!(
            index
                .checked_shr(self.index_bit_len.get() as u32)
                .map(|r| r == 0)
                .unwrap_or(true),
            "Index has too many bits"
        );

        ContainerPath {
            path: self.path | ((index as IndexBase) << self.index_off.get()),
            bit_len: self.bit_len + self.index_bit_len.get(),
        }
    }

    pub fn path<T: Pointee + AnyItem + ?Sized>(&self, key: Key<T>) -> usize {
        ((key.index().get() ^ self.path) >> self.index_off.get()) as usize
    }
}
