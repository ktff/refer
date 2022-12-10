use bitvec::access;

use crate::core::{
    self, collection::Result, container, AnyContainer, AnyKey, Container, Key, KeyPrefix,
    UnsafeSlot,
};
use std::{
    any::{Any, TypeId},
    collections::HashSet,
    marker::{PhantomData, Unsize},
    ptr::Pointee,
};

pub struct Ref;
pub struct Mut;
pub struct Slot;
pub struct Item;
pub struct Shell;

pub trait ItemAccess {}
impl ItemAccess for Item {}
impl ItemAccess for Slot {}

pub trait ShellAccess {}
impl ShellAccess for Shell {}
impl ShellAccess for Slot {}

pub trait RefAccess {}
impl RefAccess for Ref {}
impl RefAccess for Mut {}

pub struct Permit<R, A> {
    _marker: PhantomData<(R, A)>,
}

impl<R, A> Permit<R, A> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<R> Permit<R, Slot> {
    pub fn split(self) -> (Permit<R, Item>, Permit<R, Shell>) {
        (
            Permit {
                _marker: PhantomData,
            },
            Permit {
                _marker: PhantomData,
            },
        )
    }
}

impl<A> Permit<Mut, A> {
    pub fn borrow(&self) -> Permit<Ref, A> {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<A> Copy for Permit<Ref, A> {}

impl<A> Clone for Permit<Ref, A> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct TypePermit<'a, T, R, A, C> {
    container: &'a C,
    _marker: PhantomData<(R, T, A)>,
}

impl<'a, R, T: core::Item, A, C: Container<T>> TypePermit<'a, T, R, A, C> {
    /// SAFETY: Caller must ensure that it has the correct R & S access to all T in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    fn access(&self) -> Permit<R, A> {
        Permit::new()
    }

    pub fn get(self, key: Key<T>) -> Result<core::Slot<'a, T, C::Shell, R, A>> {
        self.container
            .get_slot(key)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::Slot::new(key, slot, self.access()) })
            .ok_or_else(|| key.into())
    }

    pub fn iter(self) -> impl Iterator<Item = core::Slot<'a, T, C::Shell, R, A>> {
        self.iter_sub(KeyPrefix::default())
    }

    pub fn iter_sub(
        self,
        prefix: KeyPrefix,
    ) -> impl Iterator<Item = core::Slot<'a, T, C::Shell, R, A>> {
        self.container
            .iter_slot(prefix)
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T.
            .map(move |(key, slot)| unsafe { core::Slot::new(key, slot, self.access()) })
    }

    /// Splits iterator into max 2^chunk_count_pow2 chunks.
    pub fn iter_chunked(
        self,
        chunk_count_pow2: u32,
    ) -> Vec<impl Iterator<Item = core::Slot<'a, T, C::Shell, R, A>>> {
        // Compute common prefix of all keys in the iterator.
        let first = self.container.first(TypeId::of::<T>());
        let last = self.container.last(TypeId::of::<T>());
        let (first, last) = match (first, last) {
            (Some(first), Some(last)) => (first, last),
            _ => return Vec::new(),
        };
        let common = first
            .index()
            .as_prefix()
            .intersect(last.index().as_prefix());

        // Split access into chunks under common.
        common
            .iter_sub(chunk_count_pow2)
            .map(move |prefix| {
                self.container
                    .iter_slot(prefix)
                    .into_iter()
                    .flat_map(|iter| iter)
                    // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T.
                    //         That whole access we split across mutually exclusive prefixes.
                    .map(move |(key, slot)| unsafe {
                        core::Slot::new(key, slot, Permit::<R, A>::new())
                    })
            })
            .collect()
    }

    // None if a == b
    pub fn get_pair(
        self,
        a: Key<T>,
        b: Key<T>,
    ) -> Result<
        Option<(
            core::Slot<'a, T, C::Shell, R, A>,
            core::Slot<'a, T, C::Shell, R, A>,
        )>,
    > {
        if a == b {
            Ok(None)
        } else {
            let a = Self {
                container: self.container,
                _marker: PhantomData,
            }
            .get(a)?;
            let b = Self {
                container: self.container,
                _marker: PhantomData,
            }
            .get(b)?;
            Ok(Some((a, b)))
        }
    }
}

impl<'a, T: core::Item, A, C: Container<T>> TypePermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> TypePermit<T, Ref, A, C> {
        TypePermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> TypePermit<T, Mut, A, C> {
        TypePermit {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

impl<'a, R, T: core::Item, C: Container<T>> TypePermit<'a, T, R, Slot, C> {
    pub fn split_slot(
        self,
    ) -> (
        TypePermit<'a, T, R, Item, C>,
        TypePermit<'a, T, R, Shell, C>,
    ) {
        (
            TypePermit {
                container: self.container,
                _marker: PhantomData,
            },
            TypePermit {
                container: self.container,
                _marker: PhantomData,
            },
        )
    }
}

impl<'a, T, A, C> Copy for TypePermit<'a, T, Ref, A, C> {}

impl<'a, T, A, C> Clone for TypePermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

pub struct AnyPermit<'a, R, A, C> {
    container: &'a C,
    _marker: PhantomData<(R, A)>,
}

impl<'a, R, A, C: AnyContainer> AnyPermit<'a, R, A, C> {
    /// SAFETY: Caller must ensure that it has the correct R & S access to C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    fn access(&self) -> Permit<R, A> {
        Permit::new()
    }

    pub fn of<T: core::Item>(self) -> TypePermit<'a, T, R, A, C>
    where
        C: Container<T>,
    {
        TypePermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn get<T: core::Item>(self, key: Key<T>) -> Result<core::Slot<'a, T, C::Shell, R, A>>
    where
        C: Container<T>,
    {
        self.of().get(key)
    }

    pub fn get_dyn<T: core::AnyItem + Unsize<dyn core::AnyItem> + ?Sized>(
        self,
        key: Key<T>,
    ) -> Result<core::DynSlot<'a, T, R, A>> {
        self.container
            .get_slot_any(key.upcast())
            // SAFETY: Type level logic of AnyPermit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::DynSlot::new(key, slot, self.access()) })
            .ok_or_else(|| key.into())
    }

    pub fn iter(self, key: TypeId) -> impl Iterator<Item = core::AnySlot<'a, R, A>> {
        let container = self.container;
        std::iter::successors(self.first(key), move |&key| self.next(key)).map(move |key| {
            container
                .get_slot_any(key)
                // SAFETY: Type level logic of AnyPermit ensures that it has sufficient access for 'a to all slots.
                //         Furthermore first-next iteration ensures that we don't access the same slot twice.
                .map(|slot| unsafe { core::AnySlot::new(key, slot, Permit::<R, A>::new()) })
                .expect("Should be valid key")
        })
    }

    /// Returns first key for given type
    pub fn first(&self, key: TypeId) -> Option<AnyKey> {
        self.container.first(key)
    }

    /// Returns following key after given in ascending order
    /// for the same type.
    pub fn next(&self, key: AnyKey) -> Option<AnyKey> {
        self.container.next(key)
    }

    /// Returns last key for given type
    pub fn last(&self, key: TypeId) -> Option<AnyKey> {
        self.container.last(key)
    }

    /// All types in the container.
    pub fn types(&self) -> HashSet<TypeId> {
        self.container.types()
    }

    // None if a == b
    pub fn get_pair(
        self,
        a: AnyKey,
        b: AnyKey,
    ) -> Result<Option<(core::AnySlot<'a, R, A>, core::AnySlot<'a, R, A>)>> {
        if a == b {
            Ok(None)
        } else {
            let a = Self {
                container: self.container,
                _marker: PhantomData,
            }
            .get_dyn(a)?;
            let b = Self {
                container: self.container,
                _marker: PhantomData,
            }
            .get_dyn(b)?;
            Ok(Some((a, b)))
        }
    }
}

impl<'a, C: AnyContainer> AnyPermit<'a, Mut, Slot, C> {
    pub fn split_slots(self) -> (AnyPermit<'a, Mut, Item, C>, AnyPermit<'a, Mut, Shell, C>) {
        (
            AnyPermit {
                container: self.container,
                _marker: PhantomData,
            },
            AnyPermit {
                container: self.container,
                _marker: PhantomData,
            },
        )
    }
}

impl<'a, A, C: AnyContainer> AnyPermit<'a, Mut, A, C> {
    pub fn split_types(self) -> TypeSplitPermit<'a, A, C> {
        TypeSplitPermit {
            container: self.container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn split_keys(self) -> KeySplitPermit<'a, A, C> {
        KeySplitPermit {
            container: self.container,
            splitted: HashSet::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, A, C: AnyContainer> AnyPermit<'a, Mut, A, C> {
    pub fn borrow(&self) -> AnyPermit<Ref, A, C> {
        AnyPermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<Mut, A, C> {
        AnyPermit {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

impl<'a, A, C> Copy for AnyPermit<'a, Ref, A, C> {}

impl<'a, A, C> Clone for AnyPermit<'a, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

pub struct TypeSplitPermit<'a, A, C> {
    container: &'a C,
    splitted: Vec<TypeId>,
    _marker: PhantomData<A>,
}

impl<'a, A, C> TypeSplitPermit<'a, A, C> {
    /// SAFETY: Caller must ensure that it has exclusive access to S in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn of<T: core::Item>(&mut self) -> Option<TypePermit<'a, T, Mut, A, C>>
    where
        C: Container<T>,
    {
        if self.splitted.contains(&TypeId::of::<T>()) {
            None
        } else {
            self.splitted.push(TypeId::of::<T>());
            Some(TypePermit {
                container: self.container,
                _marker: PhantomData,
            })
        }
    }
}

pub struct KeySplitPermit<'a, A, C> {
    container: &'a C,
    splitted: HashSet<AnyKey>,
    _marker: PhantomData<A>,
}

impl<'a, A, C> KeySplitPermit<'a, A, C> {
    /// SAFETY: Caller must ensure that it has exclusive access to S in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            splitted: HashSet::new(),
            _marker: PhantomData,
        }
    }

    /// Each key can only be get once.
    pub fn get<T: core::Item>(
        &mut self,
        key: Key<T>,
    ) -> Result<Option<core::Slot<'a, T, C::Shell, Mut, A>>>
    where
        C: Container<T>,
    {
        if self.splitted.insert(key.upcast()) {
            TypePermit {
                container: self.container,
                _marker: PhantomData,
            }
            .get(key)
            .map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn get_dyn<T: core::AnyItem + Unsize<dyn core::AnyItem> + ?Sized>(
        &mut self,
        key: Key<T>,
    ) -> Result<Option<core::DynSlot<'a, T, Mut, A>>>
    where
        C: AnyContainer,
    {
        if self.splitted.insert(key.upcast()) {
            AnyPermit {
                container: self.container,
                _marker: PhantomData,
            }
            .get_dyn(key)
            .map(Some)
        } else {
            Ok(None)
        }
    }
}
