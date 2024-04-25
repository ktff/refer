use crate::core::*;

// ! No Key<Ref> can live across possible calls to remove.

pub trait ContainerExt: AnyContainer {
    fn as_add(&mut self) -> Add<Self> {
        Add::new(self)
    }

    fn as_mut(&mut self) -> MutAccess<Self> {
        MutAccess::new(self)
    }

    fn as_ref(&mut self) -> Access<Self> {
        Access::new_ref(self)
    }

    // fn step_down<T: Item>(&mut self) -> Option<&mut Self::Sub>
    // where
    //     Self: TypeContainer<T>,
    // {
    //     self.get_mut()
    // }

    // pub fn step_into(self, index: usize) -> Option<RemoveAccess<'a, C::Sub>>
    // where
    //     C: RegionContainer,
    // {
    //     let Self { container, permit } = self;
    //     container
    //         .get_mut(index)
    //         .map(|container| RemoveAccess { container, permit })
    // }

    // pub fn step_range(
    //     self,
    //     range: impl RangeBounds<usize>,
    // ) -> Option<impl Iterator<Item = RemoveAccess<'a, C::Sub>>>
    // where
    //     C: RegionContainer,
    // {
    //     let Self { container, permit } = self;
    //     Some(
    //         container
    //             .iter_mut(range)?
    //             .map(move |container| RemoveAccess {
    //                 container,
    //                 // SAFETY: Access is split into disjoint containers.
    //                 permit: unsafe { permit.copy() },
    //             }),
    //     )
    // }

    // TODO: fn Move Item

    // TODO: fn Redirect by modifying references to one to point to other Item

    /// true if removed.
    /// false if not removed because something outside this container is referencing it.
    /// None if key doesn't exist.
    /// Can have side effects that also invalidate other Key<Ptr>.
    fn remove<T: Item>(&mut self, key: Key<Ptr, T>) -> Option<bool>
    where
        Self: Container<T>,
    {
        if self.container_path().is_root() {
            // We are in root so only owned keys can prevent removal.

            // Standalone check
            if T::IS_STANDALONE {
                if self.as_ref().key(key).get_try()?.has_owners() {
                    // There are owned keys
                    return Some(false);
                }
            }

            // Drop
            let (item, locality) = self.unfill_slot(key)?;
            let edges = item.localized_drop(locality);

            // Detach
            let mut remove = Vec::new();
            remove_edges(self, key.any(), edges, &mut remove);

            // Propagate change
            while let Some(other) = remove.pop() {
                // Standalone items can always remove edges so remove doesn't contain such items.
                if let Some(edges) = self.localized_drop(other) {
                    remove_edges(self, other, edges, &mut remove);
                }
            }

            Some(true)
        } else {
            // Item
            let item = self.as_ref().key(key).get_try()?;
            if item.has_owners() {
                // There are owned keys
                return Some(false);
            }
            if item.iter_edges(None).next().is_none() {
                // There are no owned keys no any other edges, we can remove it

                // Drop
                let (item, locality) = self.unfill_slot(key)?;
                let edges = item.localized_drop(locality);
                assert!(
                    edges.is_empty(),
                    "Item has edges but didn't iterated over them."
                );

                Some(true)
            } else {
                // There are edges, possibly outside of this container.
                // We need to determine somehow if there are edges to items outside of this container
                // in the tree formed by this and transitive removals. If there are, we can't remove it.
                unimplemented!()
            }
        }
    }
}

impl<C: AnyContainer + ?Sized> ContainerExt for C {}

fn remove_edges(
    con: &mut (impl AnyContainer + ?Sized),
    subject: Key,
    edges: Vec<PartialEdge<Key<Owned>>>,
    remove: &mut Vec<Key>,
) {
    let mut extra = Vec::<(_, MultiOwned)>::new();
    for edge in edges {
        // Remove from extra
        if let Ok(i) = extra.binary_search_by_key(&edge.ptr(), |(edge, _)| *edge) {
            if let Some(key) = extra[i].1.take() {
                std::mem::forget(key);
            } else {
                let (_, rem) = extra.remove(i);
                std::mem::forget(rem);
            }
        }
        // Remove from object
        else if let Some(mut object) = con.as_mut().key(edge.object.ptr()).get_try() {
            let edge_ptr = edge.ptr();
            let (object_key, rev_edge) = edge.reverse(subject);
            match object.remove_edges(object_key, rev_edge) {
                Ok(subject) => {
                    let (subject, rem) = subject.sub();
                    // Add extra removed to extra
                    rem.map(
                        |rem| match extra.binary_search_by_key(&edge_ptr, |(edge, _)| *edge) {
                            Ok(i) => extra[i].1.append(rem),
                            Err(i) => extra.insert(i, (edge_ptr, rem)),
                        },
                    );
                    std::mem::forget(subject);
                }
                Err((present, object_key)) => {
                    assert_eq!(present, Found::Yes);
                    remove.push(object_key.ptr());
                    std::mem::forget(object_key);
                }
            }
        } else {
            std::mem::forget(edge.object);
        }
    }
}
