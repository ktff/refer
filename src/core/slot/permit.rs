use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use crate::core::{self, AnyContainer, AnyKey, CollectionError, Container, Key, Model};

// TODO: Make this types as only acceptable for R and S in Permit.

pub struct Ref;
pub struct Mut;
pub struct Slot;
pub struct Item;
pub struct Shell;

pub struct Permit<R, S> {
    _marker: PhantomData<(R, S)>,
}

impl<R, S> Permit<R, S> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<R> Permit<R, Slot> {
    pub fn split(self) -> Split<Permit<R, Item>, Permit<R, Shell>> {
        Split {
            items: Permit {
                _marker: PhantomData,
            },
            shells: Permit {
                _marker: PhantomData,
            },
        }
    }
}

impl<S> Permit<Mut, S> {
    pub fn borrow(&self) -> Permit<Ref, S> {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<S> Copy for Permit<Ref, S> {}

impl<S> Clone for Permit<Ref, S> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct TypePermit<'a, T, R, S, C> {
    container: &'a C,
    _marker: PhantomData<(R, T, S)>,
}

impl<'a, R, T: core::Item, S, C: Model<T>> TypePermit<'a, T, R, S, C> {
    /// SAFETY: Caller must ensure that it has the correct R & S access to all T in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    fn access(&self) -> Permit<R, S> {
        Permit::new()
    }

    pub fn get(self, key: Key<T>) -> Result<core::Slot<'a, T, C::Shell, R, S>, CollectionError> {
        self.container
            .get_slot(key.into())
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::Slot::new(key, slot, self.access()) })
            .ok_or_else(|| key.into())
    }

    pub fn iter(self) -> impl Iterator<Item = core::Slot<'a, T, C::Shell, R, S>> {
        self.container
            .iter_slot()
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T.
            .map(move |(key, slot)| unsafe { core::Slot::new(key.into_key(), slot, self.access()) })
    }

    // pub fn iter_grouped(self)->

    // None if a == b
    pub fn split_pair(
        self,
        a: Key<T>,
        b: Key<T>,
    ) -> Result<
        Option<(
            core::Slot<'a, T, C::Shell, R, S>,
            core::Slot<'a, T, C::Shell, R, S>,
        )>,
        CollectionError,
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

impl<'a, T: core::Item, S, C: Container<T>> TypePermit<'a, T, Mut, S, C> {
    pub fn borrow(&self) -> TypePermit<T, Ref, S, C> {
        TypePermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> TypePermit<T, Mut, S, C> {
        TypePermit {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

impl<'a, R, T: core::Item, C: Container<T>> TypePermit<'a, T, R, Slot, C> {
    pub fn split(
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

impl<'a, T, S, C> Copy for TypePermit<'a, T, Ref, S, C> {}

impl<'a, T, S, C> Clone for TypePermit<'a, T, Ref, S, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

pub struct AnyPermit<'a, R, S, C> {
    container: &'a C,
    _marker: PhantomData<(R, S)>,
}

impl<'a, R, S, C: AnyContainer> AnyPermit<'a, R, S, C> {
    /// SAFETY: Caller must ensure that it has the correct R & S access to C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    fn access(&self) -> Permit<R, S> {
        Permit::new()
    }

    pub fn get(self, key: AnyKey) -> Result<core::AnySlot<'a, R, S>, CollectionError> {
        self.container
            .get_slot_any(key.into())
            // SAFETY: Type level logic of AnyPermit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::AnySlot::new(key, slot, self.access()) })
            .ok_or_else(|| key.into())
    }

    /// Returns first key for given type
    pub fn first(&self, key: TypeId) -> Option<AnyKey> {
        self.container.first(key).map(|key| key.into_key())
    }

    /// Returns following key after given in ascending order
    /// for the same type.
    pub fn next(&self, key: AnyKey) -> Option<AnyKey> {
        self.container.next(key.into()).map(|key| key.into_key())
    }

    /// All types in the container.
    pub fn types(&self) -> HashSet<TypeId> {
        self.container.types()
    }

    // None if a == b
    pub fn split_pair(
        self,
        a: AnyKey,
        b: AnyKey,
    ) -> Result<Option<(core::AnySlot<'a, R, S>, core::AnySlot<'a, R, S>)>, CollectionError> {
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

impl<'a, R, S, C: AnyContainer> AnyPermit<'a, R, S, C> {
    pub fn of<T: core::Item>(self) -> TypePermit<'a, T, R, S, C>
    where
        C: Container<T>,
    {
        TypePermit {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

impl<'a, S, C: AnyContainer> AnyPermit<'a, Mut, S, C> {
    pub fn borrow(&self) -> AnyPermit<Ref, S, C> {
        AnyPermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<Mut, S, C> {
        AnyPermit {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

impl<'a, R, C: AnyContainer> AnyPermit<'a, R, Slot, C> {
    pub fn split(self) -> (AnyPermit<'a, R, Item, C>, AnyPermit<'a, R, Shell, C>) {
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

impl<'a, S, C> Copy for AnyPermit<'a, Ref, S, C> {}

impl<'a, S, C> Clone for AnyPermit<'a, Ref, S, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

pub struct SplitOwnership<'a, S, C> {
    container: &'a C,
    _marker: PhantomData<S>,
}

impl<'a, S, C: AnyContainer> SplitOwnership<'a, S, C> {
    /// SAFETY: Caller must ensure that it has exclusive access to S in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> AnyPermit<Ref, S, C> {
        AnyPermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn get_mut(&mut self) -> AnyPermit<Mut, S, C> {
        AnyPermit {
            container: self.container,
            _marker: PhantomData,
        }
    }

    pub fn split_types(self) -> TypeOwnership<'a, S, C> {
        TypeOwnership {
            container: self.container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn split_keys(self) -> KeyOwnership<'a, S, C> {
        KeyOwnership {
            container: self.container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }
}

pub struct TypeOwnership<'a, S, C> {
    container: &'a C,
    splitted: Vec<TypeId>,
    _marker: PhantomData<S>,
}

impl<'a, S, C> TypeOwnership<'a, S, C> {
    /// SAFETY: Caller must ensure that it has exclusive access to S in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn split_off<T: core::Item>(&mut self) -> Option<TypePermit<'a, T, Mut, S, C>>
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

pub struct KeyOwnership<'a, S, C> {
    container: &'a C,
    // TODO: On more keys change to HashSet
    splitted: Vec<AnyKey>,
    _marker: PhantomData<S>,
}

impl<'a, S, C> KeyOwnership<'a, S, C> {
    /// SAFETY: Caller must ensure that it has exclusive access to S in C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Each key can only be get once.
    pub fn get<T: core::Item>(
        &mut self,
        key: Key<T>,
    ) -> Result<Option<core::Slot<'a, T, C::Shell, Mut, S>>, CollectionError>
    where
        C: Model<T>,
    {
        if self.splitted.contains(&key.into()) {
            Ok(None)
        } else {
            self.splitted.push(key.into());

            TypePermit {
                container: self.container,
                _marker: PhantomData,
            }
            .get(key)
            .map(Some)
        }
    }

    // TODO: fn get_any
}

pub struct Split<I, S> {
    pub items: I,
    pub shells: S,
}

impl<I, S> Split<I, S> {
    pub fn new(item: I, shell: S) -> Self {
        Self {
            items: item,
            shells: shell,
        }
    }

    pub fn map<I2, S2, F, G>(self, f: F, g: G) -> Split<I2, S2>
    where
        F: FnOnce(I) -> I2,
        G: FnOnce(S) -> S2,
    {
        Split {
            items: f(self.items),
            shells: g(self.shells),
        }
    }
}
