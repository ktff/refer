use crate::core::*;

pub struct Owned<C>(C);

impl<C: AnyContainer> Owned<C> {
    /// UNSAFE: Caller must ensure that complete ownership is transferred.
    pub unsafe fn new(c: C) -> Self {
        Self(c)
    }

    pub fn slot(&self) -> AnyPermit<permit::Ref, permit::Slot, C> {
        // SAFETY: We have at least read access for whole C.
        unsafe { AnyPermit::new(&self.0) }
    }

    pub fn slot_mut(&mut self) -> AnyPermit<permit::Mut, permit::Slot, C> {
        // SAFETY: We have at least mut access for whole C.
        unsafe { AnyPermit::new(&self.0) }
    }

    pub fn complex(&mut self) -> ComplexOwnership<permit::Slot, C> {
        // SAFETY: We have at least mut access for whole C.
        unsafe { ComplexOwnership::new(&self.0) }
    }

    pub fn split(
        &mut self,
    ) -> (
        SplitOwnership<permit::Item, C>,
        SplitOwnership<permit::Shell, C>,
    ) {
        // SAFETY: We have at least mut access for whole C.
        unsafe { (SplitOwnership::new(&self.0), SplitOwnership::new(&self.0)) }
    }

    pub fn inner(&self) -> &C {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.0
    }
}

/// This is safe since Owned has full ownership of C.
unsafe impl<C: 'static> Sync for Owned<C> {}

impl<C: Allocator<T> + Container<T> + AnyContainer + 'static, T: Item> Collection<T> for Owned<C> {
    fn add_with(&mut self, item: T, r: Self::R) -> Result<Key<T>, T> {
        // Allocate slot
        let (key, _) = if let Some(key) = self.reserve(Some(&item), r) {
            key
        } else {
            return Err(item);
        };

        // Update connections
        if !super::util::add_references(self.slot_mut().split().1, key.key().into_key(), &item) {
            // Failed

            // Deallocate slot
            self.0.cancel(key);

            return Err(item);
        }

        // Add item & shell
        Ok(self.0.fulfill(key, item).into_key())
    }

    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T> {
        let (items, shells) = self.slot_mut().split();
        let mut slot = if let Some(item) = items.of().get(key) {
            item
        } else {
            // No item
            return Err(set);
        };

        // Update connections
        if !super::util::update_diff(shells, key, slot.item(), &set) {
            // Failed
            return Err(set);
        }

        // Replace item
        Ok(std::mem::replace(slot.item_mut(), set))
    }

    fn take(&mut self, key: Key<T>) -> Option<T> {
        let mut remove = Vec::new();

        // Update connections
        super::util::notify_item_removed(self.slot_mut().split(), key.into(), &mut remove)?;
        // Deallocate
        let (item, _) = self.0.unfill(key.into())?;

        // Recursive remove
        while let Some(rf) = remove.pop() {
            // Update connections
            if super::util::notify_item_removed(self.slot_mut().split(), rf, &mut remove).is_some()
            {
                // Deallocate
                self.0.unfill_any(rf.into());
            }
        }

        Some(item)
    }
}

impl<C: Allocator<T> + 'static, T: 'static> Allocator<T> for Owned<C> {
    type Alloc = C::Alloc;

    type R = C::R;

    fn reserve(&mut self, item: Option<&T>, r: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)> {
        self.0.reserve(item, r)
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        self.0.cancel(key)
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        self.0.fulfill(key, item)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized,
    {
        self.0.unfill(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        container::{all::AllContainer, vec::VecContainerFamily},
        item::{edge::Edge, vertice::Vertice},
    };

    #[test]
    fn reference_add() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        for node in nodes {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }
    }

    #[test]
    fn reference_add_abort() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        collection.take(nodes[n - 1]).unwrap();

        assert!(collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                ()
            )
            .is_err());

        for &node in nodes.iter().take(n - 1) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn reference_set() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add(i).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        collection
            .set(
                center,
                Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
            )
            .ok()
            .unwrap();

        for &node in nodes.iter().take(5) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }

        for &node in nodes.iter().skip(5) {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }
    }

    #[test]
    fn reference_set_abort() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add(i).unwrap())
            .collect::<Vec<_>>();

        collection.take(nodes[n - 1]).unwrap();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        assert!(collection
            .add_with(
                Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
                ()
            )
            .is_err());

        for &node in nodes.iter().take(5) {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }

        for &node in nodes.iter().skip(5).take(4) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn reference_remove() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        let _ = collection.take(center).unwrap();

        for node in nodes {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn cascading_remove() {
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let a = collection.add_with(0, ()).unwrap();
        let b = collection.add_with(1, ()).unwrap();
        let edge = collection
            .add_with(Edge::new([Ref::new(a), Ref::new(b)]), ())
            .unwrap();

        assert_eq!(collection.get(a).unwrap().1.from_count(), 1);
        assert_eq!(collection.get(b).unwrap().1.from_count(), 1);

        let _ = collection.take(a).unwrap();
        assert!(collection.get(edge).is_none());
        assert!(collection.get(a).is_none());
        assert!(collection.get(b).unwrap().0 == (&1, &()));
        assert_eq!(collection.get(b).unwrap().1.from_count(), 0);
    }
}
