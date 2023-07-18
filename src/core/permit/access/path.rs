use super::*;

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, T: TypePermit, P: PathPermit<C>>
    AccessPermit<'a, C, R, T, P>
{
    pub fn path(&self) -> Path {
        P::path(&self.key_state, self.container)
    }

    /// None if the path is not a subpath of current path.
    pub fn sub_path(self, path: impl Into<Path>) -> Option<AccessPermit<'a, C, R, T, Path>> {
        let path = path.into();
        if self.path().contains(path) {
            Some(self.key_transition(|_| path))
        } else {
            None
        }
    }

    /// Constrains the permit to the given path.
    /// None if they don't overlap.
    pub fn and(self, path: impl Into<Path>) -> Option<AccessPermit<'a, C, R, T, Path>> {
        let path = self.path().and(path)?;
        Some(self.key_transition(|_| path))
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, P: PathPermit<C>>
    AccessPermit<'a, C, R, All, P>
{
    // pub fn iter_dyn(self) -> impl Iterator<Item = Slot<'a, R, T>> {
    //     let path = self.path();
    //     let Self {
    //         permit, container, ..
    //     } = self;
    //     assert!(container.container_path().contains(path));
    //     container
    //         .iter_slot(path.of())
    //         .into_iter()
    //         .flat_map(|iter| iter)
    //         // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T under path.
    //         .map(move |slot| unsafe { Slot::new(slot, permit.access()) })
    // }

    // /// Iterates over valid slot permit of type in ascending order.
    // pub fn iter_type(
    //     self,
    //     ty: TypeId,
    // ) -> impl Iterator<Item = SlotPermit<'a, R, core::Ref<'a>, dyn AnyItem, C>> {
    //     let container = self.container;
    //     std::iter::successors(container.first_key(ty), move |&key| {
    //         container.next_key(ty, key.ptr())
    //     })
    //     .map(move |key| {
    //         // SAFETY: First-next iteration ensures that we don't access the same slot twice.
    //         unsafe { self.unsafe_split(|permit| permit.slot(key)) }
    //     })
    // }

    // /// Iterates over valid keys of type in ascending order.
    // pub fn keys(&self, ty: TypeId) -> impl Iterator<Item = Key> + 'a {
    //     let container = self.container;
    //     std::iter::successors(container.first_key(ty).map(|key| key.ptr()), move |&key| {
    //         container.next_key(ty, key).map(|key| key.ptr())
    //     })
    // }
}

impl<'a, C: Container<T> + ?Sized, R: Into<permit::Ref>, T: Item, P: PathPermit<C>>
    AccessPermit<'a, C, R, T, P>
{
    /// Splits on lower level, or returns self if level is higher.
    pub fn level_split(
        self,
        level: u32,
    ) -> Box<dyn ExactSizeIterator<Item = AccessPermit<'a, C, R, T, Path>> + 'a>
    where
        R: 'static,
    {
        // Compute common path of all keys in the iterator.
        let first = self.first_key(TypeId::of::<T>());
        let last = self.last_key(TypeId::of::<T>());
        let path = match (first, last) {
            (Some(first), Some(last)) => first.path().or(last.path()),
            (Some(_), None) | (None, Some(_)) => unreachable!(),
            // There is no slots to iterate so we can return self.
            (None, None) => {
                let path = self.path();
                return Box::new(std::iter::once(self.key_transition(|_| path)));
            }
        };

        if let Some(iter) = path
            .and(self.path())
            .expect("Path out of Container path")
            .iter_level(level)
        {
            Box::new(
                // SAFETY: We depend on iter_level returning disjoint paths.
                iter.map(move |path| unsafe {
                    self.unsafe_split(|this| this.key_transition(|_| path))
                }),
            )
        } else {
            let path = self.path();
            Box::new(std::iter::once(self.key_transition(|_| path)))
        }
    }
}

// impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, P: PathPermit<C>>
//     AccessPermit<'a, C, R, All, P> {
//     type Item = core::DynSlot<'a, R, T>;
//     type IntoIter = impl Iterator<Item = core::Slot<'a, R, T>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

impl<'a, C: Container<T> + ?Sized, R: Into<permit::Ref>, T: Item, P: PathPermit<C>> IntoIterator
    for AccessPermit<'a, C, R, T, P>
{
    type Item = Slot<'a, R, T>;
    type IntoIter = impl Iterator<Item = Slot<'a, R, T>>;

    fn into_iter(self) -> Self::IntoIter {
        let path = self.path();
        let Self {
            permit, container, ..
        } = self;
        assert!(container.container_path().contains(path));
        container
            .iter_slot(path.of())
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T under path.
            .map(move |slot| unsafe { Slot::new(slot, permit.access()) })
    }
}
