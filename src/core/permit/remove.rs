use crate::core::{container::RegionContainer, container::TypeContainer, *};
use std::ops::{Deref, DerefMut, RangeBounds};

pub struct RemovePermit<'a, C: AnyContainer + ?Sized> {
    permit: Permit<permit::Mut>,
    container: &'a mut C,
}

impl<'a, C: AnyContainer + ?Sized> RemovePermit<'a, C> {
    pub fn new(container: &'a mut C) -> Self {
        Self {
            // SAFETY: Mut access is proof of exclusive access.
            permit: unsafe { Permit::<permit::Mut>::new() },
            container,
        }
    }

    pub fn borrow_mut(&mut self) -> RemovePermit<'_, C> {
        RemovePermit {
            permit: self.permit.access(),
            container: self.container,
        }
    }

    pub fn access(&self) -> AnyPermit<permit::Ref, C> {
        // SAFETY: We have at least read access for whole C.
        unsafe { AnyPermit::unsafe_new(self.permit.borrow(), &self) }
    }

    pub fn access_mut(&mut self) -> AnyPermit<permit::Mut, C> {
        AnyPermit::new(self.container)
    }

    pub fn step<T: Item>(self) -> Option<RemovePermit<'a, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { container, permit } = self;
        container
            .get_mut()
            .map(|container| RemovePermit { container, permit })
    }

    pub fn step_into(self, index: usize) -> Option<RemovePermit<'a, C::Sub>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        container
            .get_mut(index)
            .map(|container| RemovePermit { container, permit })
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = RemovePermit<'a, C::Sub>>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        Some(
            container
                .iter_mut(range)?
                .map(move |container| RemovePermit {
                    container,
                    permit: permit.access(),
                }),
        )
    }

    /// Expects that slot exists
    fn drop_slot<T: Item>(&mut self, key: Key<Refer, T>)
    where
        C: Container<T>,
    {
        let (item, locality) = self.unfill_slot(key.into()).expect("Should be present");
        item.localized_drop(locality);
    }

    fn resolve_remove(&mut self, mut remove: Vec<Key>) {
        // Recursive remove
        while let Some(other) = remove.pop() {
            // Detach
            if self.detach(other, &mut remove).is_ok() {
                // Drop
                self.unfill_slot_any(other);
            }
        }
    }
}

impl<'a, C: AnyContainer + ?Sized> RemovePermit<'a, C> {
    // TODO: Incorporate Ref

    /// May have side effects that invalidate other Keys.
    pub fn remove<T: Item>(&mut self, key: Key<T>) -> Result<()>
    where
        C: Container<T>,
    {
        // Detach

        // item <--> others
        let mut remove = Vec::new();
        self.detach(key.any(), &mut remove)?;

        // Drop
        let (item, locality) = self.unfill_slot(key).expect("Should be present");
        item.localized_drop(locality);

        // Propagate change
        self.resolve_remove(remove);

        Ok(())
    }

    /// Detaches item from others.
    ///
    /// Err if key doesn't exist.
    fn detach(&mut self, subject: Key, remove: &mut Vec<Key>) -> Result<()> {
        // Access
        let (item, mut others) = self.access_mut().split_of(subject);

        // Disconnect from others
        for edge in item.get_dyn()?.edges(None) {
            others
                .slot(edge.key())
                .and_then(|slot| slot.get_dyn().ok())
                .map(|mut other| {
                    if !other.remove_edge(edge.reverse(subject)) {
                        remove.push(edge.key());
                    }
                });
        }

        Ok(())
    }
}

impl<'a, C: AnyContainer + ?Sized> Deref for RemovePermit<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.container
    }
}

impl<'a, C: AnyContainer + ?Sized> DerefMut for RemovePermit<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.container
    }
}
