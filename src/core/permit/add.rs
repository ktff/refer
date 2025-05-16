use crate::core::{container::RegionContainer, container::TypeContainer, permit::Permit, *};
use std::ops::{Deref, RangeBounds};

//? NOTE: This Permit must not allow for remove operations to be called during it's lifetime,
//?       as this would allow for a Key<Ref> to live across a remove operation, so it must not
//?       expose &mut C outside self.
/// Permit for adding items to a container.
pub struct AddAccess<'a, C: AnyContainer + ?Sized> {
    permit: permit::Add,
    container: &'a mut C,
}

impl<'a, C: AnyContainer + ?Sized> AddAccess<'a, C> {
    pub fn new(container: &'a mut C) -> Self {
        Self {
            // SAFETY: Mut access is proof of exclusive access.
            permit: unsafe { permit::Add::new().into() },
            container,
        }
    }

    pub fn extend<D: DynItem + ?Sized>(&self, key: Key<Ref<'_>, D>) -> Key<Ref<'a>, D> {
        // SAFETY: We have 'a lifetime guarantee no item will be removed so key is valid for 'a.
        unsafe { key.extend() }
    }

    pub fn borrow_mut(&mut self) -> AddAccess<'_, C> {
        AddAccess {
            // SAFETY: We are borrowing exclusive access to self.
            permit: unsafe { self.permit.copy() },
            container: self.container,
        }
    }

    pub fn as_mut(&mut self) -> MutAccess<C> {
        Access::new(self.container)
    }

    pub fn as_ref(&self) -> Access<C> {
        // SAFETY: We have at least read access for whole C.
        unsafe { Access::unsafe_new(self.permit.borrow(), &self) }
    }

    pub fn step<T: Item>(self) -> Option<AddAccess<'a, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { container, permit } = self;
        container
            .step_down_mut()
            .map(|container| AddAccess { container, permit })
    }

    pub fn step_into(self, index: usize) -> Option<AddAccess<'a, C::Sub>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        container
            .get_mut(index)
            .map(|container| AddAccess { container, permit })
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = AddAccess<'a, C::Sub>>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        Some(container.iter_mut(range)?.map(move |container| AddAccess {
            container,
            // SAFETY: Access is split into disjoint containers.
            permit: unsafe { permit.copy() },
        }))
    }

    // TODO: Add without locality?, callers can use step_* functions to select the container.

    /// Adds an item to the container & connects existing source edges from it.
    /// Assumes Grc was used for building those edges.
    pub fn add<T: Item>(&mut self, locality: &impl LocalityPath, item: T) -> Key<Ref<'a>, T>
    where
        C: Container<T>,
    {
        self.try_add(locality, item).ok().expect("No more space")
    }

    /// Adds an item to the container & connects existing source edges from it.
    /// Assumes Grc was used for building those edges.
    pub fn try_add<T: Item>(
        &mut self,
        locality: &impl LocalityPath,
        item: T,
    ) -> Result<Key<Ref<'a>, T>, T>
    where
        C: Container<T>,
    {
        let key = self.container.fill_slot(locality, item)?;

        // SAFETY: This permit has exclusive access to the container for 'a
        //         and it doesn't allow for removal of items. Hence, all references
        //         are valid for at least 'a.
        let key = unsafe { Key::new_ref(key.index()) };

        // We just added this item to the container
        self.connect_source_edges(key);

        Ok(key)
    }

    /// Connects subjects source side edges to drain Items.
    /// Caller must ensure that this is called only once, when subject was put into the slot.
    fn connect_source_edges<T: Item>(&mut self, subject: Key<Ref, T>)
    where
        C: Container<T>,
    {
        let (subject, mut others) = self.as_mut().key_split(subject);
        let subject = subject.fetch();
        // We should be only iterating over drains since there is no way for other items to add edges to
        // this item before it was added to the container.
        for object in subject.iter_edges() {
            if let Some(drain) = others.borrow_mut().key_try(object) {
                // SAFETY: Subject,source,this exists at least for the duration of this function.
                //         By adding it(Key) to the drain, anyone dropping the drain will know that
                //         this subject needs to be notified. This ensures that edge in subject is
                //         valid for it's lifetime.
                let source = unsafe { Key::<_, T>::new_owned(subject.key().index()) };
                let mut drain = drain.fetch();
                let excess_key = match drain.any_localized(|item, locality| {
                    item.any_add_drain_edge(locality, source.any())
                }){
                    // SAFETY: This is the other part of edge we just added.
                    Ok (()) => unsafe{Grc::new(drain.locality().owned_key())},
                    Err(_) => panic!(
                        "Invalid item edge: subject {} -> object {}, object not drain, but owned reference of him exists.",
                        subject.key(), drain.key(),
                    )
                };
                drain.release_dyn(excess_key);
            } else if object == subject.key() {
                // We skip self references
            } else {
                // We should have caught this earlier or handle it in some other way.
                unimplemented!(
                    "Drain not found for edge object: {:?}. It's probably in some other container",
                    object
                );
            }
        }
    }
}

impl<'a, C: AnyContainer + ?Sized> Deref for AddAccess<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         container::{all::AllContainer, vec::VecContainerFamily},
//         item::{edge::Edge, vertice::Vertice},
//     };

//     #[test]
//     fn reference_add() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add_with(i, ()).unwrap())
//             .collect::<Vec<_>>();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         for node in nodes {
//             assert_eq!(
//                 collection
//                     .get(node)
//                     .unwrap()
//                     .1
//                     .from::<Vertice<usize>>()
//                     .collect::<Vec<_>>(),
//                 vec![center]
//             );
//         }
//     }

//     #[test]
//     fn reference_add_abort() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add_with(i, ()).unwrap())
//             .collect::<Vec<_>>();

//         collection.remove(nodes[n - 1]).unwrap();

//         assert!(collection
//             .add_with(
//                 Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
//                 ()
//             )
//             .is_err());

//         for &node in nodes.iter().take(n - 1) {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }
//     }

//     #[test]
//     fn reference_set() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add(i).unwrap())
//             .collect::<Vec<_>>();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         collection
//             .set(
//                 center,
//                 Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
//             )
//             .ok()
//             .unwrap();

//         for &node in nodes.iter().take(5) {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }

//         for &node in nodes.iter().skip(5) {
//             assert_eq!(
//                 collection
//                     .get(node)
//                     .unwrap()
//                     .1
//                     .from::<Vertice<usize>>()
//                     .collect::<Vec<_>>(),
//                 vec![center]
//             );
//         }
//     }

//     #[test]
//     fn reference_set_abort() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add(i).unwrap())
//             .collect::<Vec<_>>();

//         collection.remove(nodes[n - 1]).unwrap();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         assert!(collection
//             .add_with(
//                 Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
//                 ()
//             )
//             .is_err());

//         for &node in nodes.iter().take(5) {
//             assert_eq!(
//                 collection
//                     .get(node)
//                     .unwrap()
//                     .1
//                     .from::<Vertice<usize>>()
//                     .collect::<Vec<_>>(),
//                 vec![center]
//             );
//         }

//         for &node in nodes.iter().skip(5).take(4) {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }
//     }

//     #[test]
//     fn reference_remove() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add_with(i, ()).unwrap())
//             .collect::<Vec<_>>();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         let _ = collection.remove(center).unwrap();

//         for node in nodes {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }
//     }

//     #[test]
//     fn cascading_remove() {
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let a = collection.add_with(0, ()).unwrap();
//         let b = collection.add_with(1, ()).unwrap();
//         let edge = collection
//             .add_with(Edge::new([Ref::new(a), Ref::new(b)]), ())
//             .unwrap();

//         assert_eq!(collection.get(a).unwrap().1.from_count(), 1);
//         assert_eq!(collection.get(b).unwrap().1.from_count(), 1);

//         let _ = collection.remove(a).unwrap();
//         assert!(collection.get(edge).is_none());
//         assert!(collection.get(a).is_none());
//         assert!(collection.get(b).unwrap().0 == (&1, &()));
//         assert_eq!(collection.get(b).unwrap().1.from_count(), 0);
//     }
// }
