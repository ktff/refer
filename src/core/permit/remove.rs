use crate::core::{container::RegionContainer, container::TypeContainer, permit::Permit, *};
use std::ops::{Deref, DerefMut, RangeBounds};

/// Permit to remove items from a container.
/// No Key<Ref> can live across possible calls to remove.
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

    pub fn access(&self) -> Access<C> {
        // SAFETY: We have at least read access for whole C.
        unsafe { Access::unsafe_new(self.permit.borrow(), &self) }
    }

    pub fn access_mut(&mut self) -> MutAccess<C> {
        MutAccess::new(self.container)
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

    /// Some(true) if removed.
    /// Some(false) if not removed because there are edgeless references to it.
    /// None if key doesn't exist.
    /// Can have side effects that also invalidate other Key<Ptr>.
    pub fn remove<T: Item>(&mut self, key: Key<Ptr, T>) -> Option<bool>
    where
        C: Container<T>,
    {
        // Standalone check
        if T::IS_STANDALONE {
            if self.access().key(key).get_try()?.has_owners() {
                return Some(false);
            }
        }

        // Drop
        let (item, locality) = self.unfill_slot(key)?;
        let edges = item.localized_drop(locality);

        // Detach
        let mut remove = Vec::new();
        self.remove_edges(key.any(), edges, &mut remove);

        // Propagate change
        while let Some(other) = remove.pop() {
            // Standalone items can always remove edges so remove doesn't contain such items.
            if let Some(edges) = self.localized_drop(other) {
                self.remove_edges(other, edges, &mut remove);
            }
        }

        Some(true)
    }

    fn remove_edges(
        &mut self,
        subject: Key,
        edges: Vec<PartialEdge<Key<Owned>>>,
        remove: &mut Vec<Key>,
    ) {
        for edge in edges {
            if let Some(mut object) = self.access_mut().key(edge.object.ptr()).get_dyn_try() {
                let (object_key, rev_edge) = edge.reverse(subject);
                match object.remove_edge(object_key, rev_edge) {
                    Ok(subject) => std::mem::forget(subject),
                    Err(object_key) => {
                        remove.push(object_key.ptr());
                        std::mem::forget(object_key);
                    }
                }
            } else {
                std::mem::forget(edge.object);
            }
        }
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
