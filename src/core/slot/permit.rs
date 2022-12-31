use crate::core::{self, collection::Result, AnyContainer, AnyKey, Container, Key, KeyPath};
use log::*;
use std::{any::TypeId, collections::HashSet, marker::PhantomData, ops::Deref};

pub struct Mut;

pub struct Ref;
impl From<Mut> for Ref {
    fn from(_: Mut) -> Self {
        Ref
    }
}

pub struct Slot;

pub struct Item;
impl From<Slot> for Item {
    fn from(_: Slot) -> Self {
        Item
    }
}

pub struct Shell;
impl From<Slot> for Shell {
    fn from(_: Slot) -> Self {
        Shell
    }
}

pub struct Permit<R, A> {
    _marker: PhantomData<(R, A)>,
}

impl<R, A> Permit<R, A> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn access(&self) -> Self {
        Self::new()
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
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<A: Into<B>, B> From<Permit<Mut, A>> for Permit<Ref, B> {
    fn from(_: Permit<Mut, A>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<R> From<Permit<R, Slot>> for Permit<R, Item> {
    fn from(_: Permit<R, Slot>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<R> From<Permit<R, Slot>> for Permit<R, Shell> {
    fn from(_: Permit<R, Slot>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

pub struct SlotPermit<'a, T: core::AnyItem + ?Sized, R, A, C: ?Sized> {
    permit: TypePermit<'a, T, R, A, C>,
    key: Key<T>,
}

impl<'a, R, T: core::Item, A, C: Container<T>> SlotPermit<'a, T, R, A, C> {
    pub fn get(self) -> Result<core::Slot<'a, T, C::Shell, R, A>> {
        let Self { permit, key } = self;
        permit
            .get_slot(key)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::Slot::new(key, slot, permit.permit.permit) })
            .ok_or_else(|| key.into())
    }
}

impl<'a, R, T: core::DynItem + ?Sized, A, C: AnyContainer + ?Sized> SlotPermit<'a, T, R, A, C> {
    pub fn get_dyn(self) -> Result<core::DynSlot<'a, T, R, A>> {
        let Self { permit, key } = self;
        permit
            .get_slot_any(key.upcast())
            // SAFETY: Type level logic of AnyPermit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::DynSlot::new(key, slot, permit.permit.permit) })
            .ok_or_else(|| key.into())
    }
}

impl<'a, T: core::AnyItem + ?Sized, A, C: ?Sized> SlotPermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> SlotPermit<T, Ref, A, C> {
        SlotPermit {
            permit: self.permit.borrow(),
            key: self.key,
        }
    }

    pub fn borrow_mut(&mut self) -> SlotPermit<T, Mut, A, C> {
        SlotPermit {
            permit: self.permit.borrow_mut(),
            key: self.key,
        }
    }
}

impl<'a, T: core::AnyItem + ?Sized, A, C: ?Sized> Copy for SlotPermit<'a, T, Ref, A, C> {}

impl<'a, T: core::AnyItem + ?Sized, A, C: ?Sized> Clone for SlotPermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

pub struct PathPermit<'a, T: core::Item, R, A, C> {
    permit: TypePermit<'a, T, R, A, C>,
    path: KeyPath<T>,
}

impl<'a, R, T: core::Item, A, C: Container<T>> PathPermit<'a, T, R, A, C> {
    pub fn path(&self) -> KeyPath<T> {
        self.path
    }

    pub fn iter(self) -> impl Iterator<Item = core::Slot<'a, T, C::Shell, R, A>> {
        let Self { permit, path } = self;
        permit
            .iter_slot(path)
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T under path.
            .map(move |(key, slot)| unsafe {
                core::Slot::new(key, slot, permit.permit.permit.access())
            })
    }

    /// Splits on lower level, or returns self if level is higher.
    pub fn split_level(
        self,
        level: u32,
    ) -> Box<dyn ExactSizeIterator<Item = PathPermit<'a, T, R, A, C>> + 'a>
    where
        R: 'static,
        A: 'static,
    {
        if let Some(iter) = self.path.iter_level(level) {
            Box::new(iter.map(move |path| Self {
                permit: TypePermit {
                    permit: AnyPermit {
                        permit: Permit {
                            ..self.permit.permit.permit
                        },
                        ..self.permit.permit
                    },
                    ..self.permit
                },
                path,
            }))
        } else {
            Box::new(std::iter::once(self))
        }
    }
}

impl<'a, T: core::Item, A, C: Container<T>> PathPermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> PathPermit<T, Ref, A, C> {
        PathPermit {
            permit: self.permit.borrow(),
            path: self.path,
        }
    }

    pub fn borrow_mut(&mut self) -> PathPermit<T, Mut, A, C> {
        PathPermit {
            permit: self.permit.borrow_mut(),
            path: self.path,
        }
    }
}

impl<'a, T: core::Item, R, A, C: Container<T>> Deref for PathPermit<'a, T, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: core::Item, A, C> Copy for PathPermit<'a, T, Ref, A, C> {}

impl<'a, T: core::Item, A, C> Clone for PathPermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            path: self.path,
        }
    }
}

pub struct TypePermit<'a, T: ?Sized, R, A, C: ?Sized> {
    permit: AnyPermit<'a, R, A, C>,
    _marker: PhantomData<&'a T>,
}

impl<'a, R, T: core::Item, A, C: Container<T>> TypePermit<'a, T, R, A, C> {
    pub fn slot(self, key: Key<T>) -> SlotPermit<'a, T, R, A, C> {
        SlotPermit { permit: self, key }
    }

    pub fn path(self, path: KeyPath<T>) -> PathPermit<'a, T, R, A, C> {
        PathPermit { permit: self, path }
    }

    // Sub over all T in the container.
    pub fn all(self) -> PathPermit<'a, T, R, A, C> {
        // Compute common prefix of all keys in the iterator.
        let first = self.first(TypeId::of::<T>());
        let last = self.last(TypeId::of::<T>());
        let (first, last) = match (first, last) {
            (Some(first), Some(last)) => (first, last),
            _ => return self.path(KeyPath::default()),
        };
        let common = first.path().intersect(last.path()).of();

        self.path(common)
    }

    // None if a == b
    pub fn split_pair(
        self,
        a: Key<T>,
        b: Key<T>,
    ) -> Option<(SlotPermit<'a, T, R, A, C>, SlotPermit<'a, T, R, A, C>)> {
        if a == b {
            None
        } else {
            Some((
                Self {
                    permit: AnyPermit {
                        permit: Permit {
                            ..self.permit.permit
                        },
                        ..self.permit
                    },
                    ..self
                }
                .slot(a),
                self.slot(b),
            ))
        }
    }
}

impl<'a, T: core::Item, A: Into<Shell>, C: Container<T>> TypePermit<'a, T, Mut, A, C> {
    pub fn connect(&mut self, from: AnyKey, to: Key<T>) -> core::Ref<T> {
        self.borrow_mut()
            .slot(to)
            .get()
            .map_err(|error| {
                error!("Failed to connect {:?} -> {:?}, error: {}", from, to, error);
                error
            })
            .expect("Failed to connect")
            .shell_add(from);

        core::Ref::new(to)
    }

    pub fn disconnect(&mut self, from: AnyKey, to: core::Ref<T>) {
        self.borrow_mut()
            .slot(to.key())
            .get()
            .map_err(|error| {
                error!(
                    "Failed to disconnect {:?} -> {:?}, error: {}",
                    from,
                    to.key(),
                    error
                );
                error
            })
            .expect("Failed to disconnect")
            .shell_remove(from)
    }
}

impl<'a, T: ?Sized, A, C: ?Sized> TypePermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> TypePermit<T, Ref, A, C> {
        TypePermit {
            permit: (&self.permit).into(),
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> TypePermit<T, Mut, A, C> {
        TypePermit {
            permit: (&mut self.permit).into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized, R, A, C: ?Sized> Deref for TypePermit<'a, T, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: ?Sized, A, C: ?Sized> Copy for TypePermit<'a, T, Ref, A, C> {}

impl<'a, T: ?Sized, A, C: ?Sized> Clone for TypePermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, A: Into<B>, B, C: ?Sized> From<TypePermit<'a, T, Mut, A, C>>
    for TypePermit<'a, T, Ref, B, C>
{
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, Mut, A, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R, C: ?Sized> From<TypePermit<'a, T, R, Slot, C>> for TypePermit<'a, T, R, Item, C> {
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R, C: ?Sized> From<TypePermit<'a, T, R, Slot, C>> for TypePermit<'a, T, R, Shell, C> {
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a: 'b, 'b, T, R, A, B, C: ?Sized> From<&'b TypePermit<'a, T, R, A, C>>
    for TypePermit<'b, T, Ref, B, C>
where
    Permit<R, A>: Into<Permit<Ref, B>>,
{
    fn from(permit: &'b TypePermit<'a, T, R, A, C>) -> Self {
        Self {
            permit: (&permit.permit).into(),
            _marker: PhantomData,
        }
    }
}

impl<'a: 'b, 'b, T, A, B, C: ?Sized> From<&'b mut TypePermit<'a, T, Mut, A, C>>
    for TypePermit<'b, T, Mut, B, C>
where
    Permit<Mut, A>: Into<Permit<Mut, B>>,
{
    fn from(permit: &'b mut TypePermit<'a, T, Mut, A, C>) -> Self {
        Self {
            permit: (&mut permit.permit).into(),
            _marker: PhantomData,
        }
    }
}

pub struct TypeSplitPermit<'a, A, C> {
    permit: AnyPermit<'a, Mut, A, C>,
    splitted: Vec<TypeId>,
}

impl<'a, A, C> TypeSplitPermit<'a, A, C> {
    pub fn ty<T: core::Item>(&mut self) -> Option<TypePermit<'a, T, Mut, A, C>>
    where
        C: Container<T>,
    {
        if self.splitted.contains(&TypeId::of::<T>()) {
            None
        } else {
            self.splitted.push(TypeId::of::<T>());
            Some(
                AnyPermit {
                    permit: Permit {
                        ..self.permit.permit
                    },
                    ..self.permit
                }
                .ty(),
            )
        }
    }
}

pub struct SlotSplitPermit<'a, A, C: ?Sized> {
    permit: AnyPermit<'a, Mut, A, C>,
    splitted: HashSet<AnyKey>,
}

impl<'a, A, C: ?Sized> SlotSplitPermit<'a, A, C> {
    pub fn slot<T: core::DynItem + ?Sized>(
        &mut self,
        key: Key<T>,
    ) -> Option<SlotPermit<'a, T, Mut, A, C>>
    where
        C: AnyContainer,
    {
        if self.splitted.insert(key.upcast()) {
            Some(
                AnyPermit {
                    permit: Permit {
                        ..self.permit.permit
                    },
                    ..self.permit
                }
                .slot(key),
            )
        } else {
            None
        }
    }
}

pub struct AnyPermit<'a, R, A, C: ?Sized> {
    permit: Permit<R, A>,
    container: &'a C,
}

impl<'a, R, A, C: AnyContainer + ?Sized> AnyPermit<'a, R, A, C> {
    /// SAFETY: Caller must ensure that it has the correct R & S access to C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            permit: Permit::new(),
        }
    }

    pub fn slot<T: core::DynItem + ?Sized>(self, key: Key<T>) -> SlotPermit<'a, T, R, A, C> {
        SlotPermit {
            permit: TypePermit {
                permit: self,
                _marker: PhantomData,
            },
            key,
        }
    }

    pub fn ty<T: core::Item>(self) -> TypePermit<'a, T, R, A, C>
    where
        C: Container<T> + Sized,
    {
        TypePermit {
            permit: self,
            _marker: PhantomData,
        }
    }

    pub fn iter(self, key: TypeId) -> impl Iterator<Item = core::AnySlot<'a, R, A>> {
        let Self { container, permit } = self;
        std::iter::successors(container.first(key), move |&key| container.next(key)).map(
            move |key| {
                container
                    .get_slot_any(key)
                    // SAFETY: Type level logic of AnyPermit ensures that it has sufficient access for 'a to all slots.
                    //         Furthermore first-next iteration ensures that we don't access the same slot twice.
                    .map(|slot| unsafe { core::AnySlot::new(key, slot, permit.access()) })
                    .expect("Should be valid key")
            },
        )
    }

    // None if a == b
    pub fn split_pair(
        self,
        a: AnyKey,
        b: AnyKey,
    ) -> Option<(
        SlotPermit<'a, dyn core::AnyItem, R, A, C>,
        SlotPermit<'a, dyn core::AnyItem, R, A, C>,
    )> {
        if a == b {
            None
        } else {
            Some((
                Self {
                    permit: Permit { ..self.permit },
                    container: self.container,
                }
                .slot(a),
                self.slot(b),
            ))
        }
    }
}

impl<'a, A: Into<Shell>, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn connect_dyn<T: core::DynItem + ?Sized>(
        &mut self,
        from: AnyKey,
        to: Key<T>,
    ) -> core::DynRef<T> {
        self.borrow_mut()
            .slot(to)
            .get_dyn()
            .map_err(|error| {
                error!("Failed to connect {:?} -> {:?}, error: {}", from, to, error);
                error
            })
            .expect("Failed to connect")
            .shell_add(from);

        core::DynRef::new(to)
    }

    pub fn disconnect_dyn<T: core::DynItem + ?Sized>(&mut self, from: AnyKey, to: core::DynRef<T>) {
        self.borrow_mut()
            .slot(to.key())
            .get_dyn()
            .map_err(|error| {
                error!(
                    "Failed to disconnect {:?} -> {:?}, error: {}",
                    from,
                    to.key(),
                    error
                );
                error
            })
            .expect("Failed to disconnect")
            .shell_remove(from)
    }
}

impl<'a, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, Slot, C> {
    pub fn split_parts(self) -> (AnyPermit<'a, Mut, Item, C>, AnyPermit<'a, Mut, Shell, C>) {
        let (item, shell) = self.permit.split();
        (
            AnyPermit {
                permit: item,
                container: self.container,
            },
            AnyPermit {
                permit: shell,
                container: self.container,
            },
        )
    }
}

impl<'a, A, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn split_types(self) -> TypeSplitPermit<'a, A, C>
    where
        C: Sized,
    {
        TypeSplitPermit {
            permit: self,
            splitted: Vec::new(),
        }
    }

    pub fn split_slots(self) -> SlotSplitPermit<'a, A, C> {
        SlotSplitPermit {
            permit: self,
            splitted: HashSet::new(),
        }
    }
}

impl<'a, A, C: ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn borrow(&self) -> AnyPermit<Ref, A, C> {
        self.into()
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<Mut, A, C> {
        self.into()
    }
}

impl<'a, R, A, C: ?Sized> Deref for AnyPermit<'a, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a, A, C: ?Sized> Copy for AnyPermit<'a, Ref, A, C> {}

impl<'a, A, C: ?Sized> Clone for AnyPermit<'a, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: Permit { ..self.permit },
            ..*self
        }
    }
}

impl<'a, A: Into<B>, B, C: ?Sized> From<AnyPermit<'a, Mut, A, C>> for AnyPermit<'a, Ref, B, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, Mut, A, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a, R, C: ?Sized> From<AnyPermit<'a, R, Slot, C>> for AnyPermit<'a, R, Item, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a, R, C: ?Sized> From<AnyPermit<'a, R, Slot, C>> for AnyPermit<'a, R, Shell, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a: 'b, 'b, R, A, B, C: ?Sized> From<&'b AnyPermit<'a, R, A, C>> for AnyPermit<'b, Ref, B, C>
where
    Permit<R, A>: Into<Permit<Ref, B>>,
{
    fn from(permit: &'b AnyPermit<'a, R, A, C>) -> Self {
        Self {
            permit: permit.permit.access().into(),
            container: permit.container,
        }
    }
}

impl<'a: 'b, 'b, A, B, C: ?Sized> From<&'b mut AnyPermit<'a, Mut, A, C>>
    for AnyPermit<'b, Mut, B, C>
where
    Permit<Mut, A>: Into<Permit<Mut, B>>,
{
    fn from(permit: &'b mut AnyPermit<'a, Mut, A, C>) -> Self {
        Self {
            permit: permit.permit.access().into(),
            container: permit.container,
        }
    }
}
