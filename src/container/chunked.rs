use std::{any::TypeId, collections::HashSet};

use crate::core::*;

pub type SlotIter<'a, L: ChunkingLogic<T>, T: AnyItem>
where
    L::C: Container<T>,
= impl Iterator<
    Item = (
        SubKey<T>,
        UnsafeSlot<
            'a,
            T,
            <<L as Chunk>::C as Container<T>>::GroupItem,
            <<L as Chunk>::C as Container<T>>::Shell,
            <<L as Chunk>::C as Allocator<T>>::Alloc,
        >,
    ),
>;

pub trait Chunk: Sync + Send + 'static {
    /// Chunk container type
    type C: 'static;

    /// Key len of chunking layer.
    fn key_len(&self) -> u32;
}

pub trait ChunkingLogic<T: AnyItem>: Chunk
where
    Self::C: Allocator<T>,
{
    // TODO: Some descriptive name
    type R: Copy;

    /// Assign item to some chunk.
    fn assign(
        &mut self,
        chunks: &mut Vec<Self::C>,
        item: Option<&T>,
        r: Self::R,
    ) -> Option<(usize, <Self::C as Allocator<T>>::Locality)>;
}

/// A container that chunks items into separate containers according to ChunkingLogic.
pub struct Chunked<L: Chunk> {
    logic: L,
    chunks: Vec<L::C>,
}

impl<L: Chunk> Chunked<L> {
    pub fn new(logic: L) -> Self {
        Self {
            logic,
            chunks: Vec::new(),
        }
    }

    // TODO: Think through correctness of such inner methods and can they be replaced.
    pub fn inner(&self) -> &[L::C] {
        &self.chunks
    }

    pub fn inner_mut(&mut self) -> &mut [L::C] {
        &mut self.chunks
    }

    pub fn inner_logic(&self) -> &L {
        &self.logic
    }
}

impl<L: ChunkingLogic<T>, T: AnyItem> Allocator<T> for Chunked<L>
where
    L::C: Allocator<T>,
{
    type Alloc = <L::C as Allocator<T>>::Alloc;

    type Locality = L::R;

    fn reserve(
        &mut self,
        item: Option<&T>,
        r: Self::Locality,
    ) -> Option<(ReservedKey<T>, &Self::Alloc)> {
        let (index, r) = self.logic.assign(&mut self.chunks, item, r)?;
        let (sub_key, alloc) = self.chunks[index].reserve(item, r)?;
        Some((sub_key.push(self.logic.key_len(), index), alloc))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let (index, sub_key) = key.pop(self.logic.key_len());
        self.chunks[index].cancel(sub_key);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        let (index, sub_key) = key.pop(self.logic.key_len());
        self.chunks[index]
            .fulfill(sub_key, item)
            .push(self.logic.key_len(), index)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized,
    {
        let (index, sub_key) = key.pop(self.logic.key_len());
        self.chunks[index].unfill(sub_key)
    }
}

impl<L: ChunkingLogic<T>, T: AnyItem> Container<T> for Chunked<L>
where
    L::C: Container<T>,
{
    type GroupItem = <L::C as Container<T>>::GroupItem;

    type Shell = <L::C as Container<T>>::Shell;

    type SlotIter<'a> = SlotIter<'a, L, T>
    where
        Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<UnsafeSlot<T, Self::GroupItem, Self::Shell, Self::Alloc>> {
        let (prefix, suffix) = key.pop(self.logic.key_len());
        self.chunks.get(prefix)?.get_slot(suffix)
    }

    fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        let key_len = self.logic.key_len();
        Some(
            self.chunks
                .iter()
                .enumerate()
                .flat_map(move |(prefix, chunk)| {
                    chunk.iter_slot().map(|iter| {
                        iter.map(move |(suffix, slot)| (suffix.push(key_len, prefix), slot))
                    })
                })
                .flat_map(|iter| iter),
        )
    }
}

impl<L: Chunk> AnyContainer for Chunked<L>
where
    L::C: AnyContainer,
{
    fn get_any_slot(&self, key: AnySubKey) -> Option<AnyUnsafeSlot> {
        let (prefix, suffix) = key.pop(self.logic.key_len());
        self.chunks.get(prefix)?.get_any_slot(suffix)
    }

    fn unfill_any_slot(&mut self, key: AnySubKey) {
        let (prefix, suffix) = key.pop(self.logic.key_len());
        self.chunks
            .get_mut(prefix)
            .map(|chunk| chunk.unfill_any_slot(suffix));
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        self.chunks.iter().enumerate().find_map(|(prefix, chunk)| {
            chunk
                .first(key)
                .map(|suffix| suffix.push(self.logic.key_len(), prefix))
        })
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        let (prefix, suffix) = key.pop(self.logic.key_len());
        let chunk = self.chunks.get(prefix)?;
        if let Some(suffix) = chunk.next(suffix) {
            Some(suffix.push(self.logic.key_len(), prefix))
        } else {
            self.chunks
                .iter()
                .enumerate()
                .skip(prefix + 1)
                .find_map(|(prefix, chunk)| {
                    chunk
                        .first(key.type_id())
                        .map(|suffix| suffix.push(self.logic.key_len(), prefix))
                })
        }
    }

    fn types(&self) -> HashSet<TypeId> {
        self.chunks.iter().flat_map(|chunk| chunk.types()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collection::owned::Owned, container::vec::VecContainer};
    use std::any::Any;

    struct Uniform;

    impl Chunk for Uniform {
        type C = VecContainer<usize>;

        fn key_len(&self) -> u32 {
            2
        }
    }

    impl ChunkingLogic<usize> for Uniform {
        type R = ();

        fn assign(
            &mut self,
            chunks: &mut Vec<Self::C>,
            item: Option<&usize>,
            _: (),
        ) -> Option<(usize, ())> {
            while chunks.len() < 1 << self.key_len() {
                chunks.push(VecContainer::new(MAX_KEY_LEN - self.key_len()));
            }
            Some(((*item.unwrap()) % (1 << self.key_len()), ()))
        }
    }

    #[test]
    fn add_items() {
        let n = 20;
        let mut container = Owned::new(Chunked::new(Uniform));

        let keys = (0..n)
            .map(|i| container.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        for (i, key) in keys.iter().enumerate() {
            assert_eq!(container.get(*key).unwrap().0, (&i, &()));
        }
    }

    #[should_panic]
    #[test]
    fn reserve_cancel() {
        let mut container = Owned::new(Chunked::new(Uniform));

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        let copy = ReservedKey::new(key.key());

        container.cancel(key);
        container.fulfill(copy, item);
    }

    #[test]
    fn take() {
        let mut container = Owned::new(Chunked::new(Uniform));

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        assert_eq!(container.remove(key).unwrap(), item);
        assert!(container.get(key).is_none());
    }

    #[test]
    fn iter() {
        let n = 20;
        let mut container = Owned::new(Chunked::new(Uniform));

        let mut keys = (0..n)
            .map(|i| (container.add_with(i, ()).unwrap(), i))
            .collect::<Vec<_>>();

        keys.sort();

        assert_eq!(
            keys,
            container
                .items()
                .iter()
                .map(|(key, (&item, _))| (key, item))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn get_any() {
        let mut container = Owned::new(Chunked::new(Uniform));

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        assert_eq!(
            (container.items_mut().get_any(key.into()).unwrap().0 as &dyn Any)
                .downcast_ref::<usize>(),
            Some(&item)
        );
    }

    #[test]
    fn unfill_any() {
        let mut container = Chunked::new(Uniform);

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        let key = container.fulfill(key, item);

        container.unfill_any_slot(key.into());
        assert!(container.get_slot(key.into()).is_none());
    }

    #[test]
    fn iter_keys() {
        let n = 20;
        let mut container = Owned::new(Chunked::new(Uniform));

        let mut keys = (0..n)
            .map(|i| container.add_with(i, ()).unwrap().into())
            .collect::<Vec<AnyKey>>();

        keys.sort();

        let any_keys = std::iter::successors(container.first(keys[0].type_id()), |key| {
            container.next(*key)
        })
        .take(30)
        .collect::<Vec<_>>();

        assert_eq!(keys, any_keys);
    }
}
