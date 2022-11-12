pub mod permit;
mod slot;

pub use slot::Slot;

use crate::{AnyItem, Container, Key};
use std::{any::TypeId, marker::PhantomData};

pub struct Owned<C>(C);

impl<C> Owned<C> {
    /// UNSAFE: Caller must ensure that complete ownership is transferred.
    pub unsafe fn new(c: C) -> Self {
        Self(c)
    }

    pub fn slot(&self) -> AnyPermit<permit::Ref, permit::Slot, C> {
        AnyPermit::new(&self.0)
    }

    pub fn slot_mut(&mut self) -> AnyPermit<permit::Mut, permit::Slot, C> {
        AnyPermit::new(&self.0)
    }

    pub fn complex(&mut self) -> ComplexOwnership<permit::Slot, C> {
        ComplexOwnership::new(&self.0)
    }

    pub fn split(
        &mut self,
    ) -> Split<SplitOwnership<permit::Item, C>, SplitOwnership<permit::Shell, C>> {
        Split {
            item: SplitOwnership::new(&self.0),
            shell: SplitOwnership::new(&self.0),
        }
    }
}

pub struct SplitOwnership<'a, S, C> {
    container: &'a C,
    _marker: PhantomData<S>,
}

impl<'a, S, C> SplitOwnership<'a, S, C> {
    fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> AnyPermit<permit::Ref, S, C> {
        AnyPermit::new(self.container)
    }

    pub fn get_mut(&mut self) -> AnyPermit<permit::Mut, S, C> {
        AnyPermit::new(self.container)
    }

    pub fn complex(self) -> ComplexOwnership<'a, S, C> {
        ComplexOwnership::new(self.container)
    }
}

pub struct ComplexOwnership<'a, S, C> {
    container: &'a C,
    splitted: Vec<TypeId>,
    _marker: PhantomData<S>,
}

impl<'a, S, C> ComplexOwnership<'a, S, C> {
    fn new(container: &'a C) -> Self {
        Self {
            container,
            splitted: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn split_off<T: AnyItem>(&mut self) -> Option<Permit<'a, permit::Mut, T, S, C>>
    where
        C: Container<T>,
    {
        if self.splitted.contains(&TypeId::of::<T>()) {
            None
        } else {
            self.splitted.push(TypeId::of::<T>());
            Some(Permit::new(self.container))
        }
    }
}

pub struct Permit<'a, R, T, S, C> {
    container: &'a C,
    _marker: PhantomData<(R, T, S)>,
}

impl<'a, R, T: AnyItem, S, C: Container<T>> Permit<'a, R, T, S, C> {
    fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }

    fn access(&self) -> Access<R, S> {
        Access {
            _marker: PhantomData,
        }
    }

    pub fn get(self, key: Key<T>) -> Option<Slot<'a, T, C::GroupItem, C::Shell, C::Alloc, R, S>> {
        self.container.get_slot(key.into()).map(|slot| Slot {
            key,
            slot,
            access: self.access(),
        })
    }

    pub fn iter(self) -> impl Iterator<Item = Slot<'a, T, C::GroupItem, C::Shell, C::Alloc, R, S>> {
        self.container
            .iter_slot()
            .into_iter()
            .flat_map(|iter| iter)
            .map(move |(key, slot)| Slot {
                key: key.into_key(),
                slot,
                access: self.access(),
            })
    }
}

impl<'a, T: AnyItem, S, C: Container<T>> Permit<'a, permit::Mut, T, S, C> {
    pub fn borrow(&self) -> Permit<permit::Ref, T, S, C> {
        Permit::new(self.container)
    }

    pub fn borrow_mut(&mut self) -> Permit<permit::Mut, T, S, C> {
        Permit::new(self.container)
    }
}

impl<'a, R, T: AnyItem, C: Container<T>> Permit<'a, R, T, permit::Slot, C> {
    pub fn split(
        self,
    ) -> Split<Permit<'a, R, T, permit::Item, C>, Permit<'a, R, T, permit::Shell, C>> {
        Split {
            item: Permit::new(self.container),
            shell: Permit::new(self.container),
        }
    }
}

impl<'a, T, S, C> Copy for Permit<'a, permit::Ref, T, S, C> {}

impl<'a, T, S, C> Clone for Permit<'a, permit::Ref, T, S, C> {
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

impl<'a, R, S, C> AnyPermit<'a, R, S, C> {
    fn new(container: &'a C) -> Self {
        Self {
            container,
            _marker: PhantomData,
        }
    }
}

impl<'a, R, S, C> AnyPermit<'a, R, S, C> {
    pub fn of<T: AnyItem>(self) -> Permit<'a, R, T, S, C>
    where
        C: Container<T>,
    {
        Permit::new(self.container)
    }
}

impl<'a, S, C> AnyPermit<'a, permit::Mut, S, C> {
    pub fn borrow(&self) -> AnyPermit<permit::Ref, S, C> {
        AnyPermit::new(self.container)
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<permit::Mut, S, C> {
        AnyPermit::new(self.container)
    }
}

impl<'a, R, C> AnyPermit<'a, R, permit::Slot, C> {
    pub fn split(
        self,
    ) -> Split<AnyPermit<'a, R, permit::Item, C>, AnyPermit<'a, R, permit::Shell, C>> {
        Split {
            item: AnyPermit::new(self.container),
            shell: AnyPermit::new(self.container),
        }
    }
}

impl<'a, S, C> Copy for AnyPermit<'a, permit::Ref, S, C> {}

impl<'a, S, C> Clone for AnyPermit<'a, permit::Ref, S, C> {
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            _marker: PhantomData,
        }
    }
}

pub struct Access<R, S> {
    _marker: PhantomData<(R, S)>,
}

impl<R> Access<R, permit::Slot> {
    pub fn split(self) -> Split<Access<R, permit::Item>, Access<R, permit::Shell>> {
        Split {
            item: Access {
                _marker: PhantomData,
            },
            shell: Access {
                _marker: PhantomData,
            },
        }
    }
}

pub struct Split<I, S> {
    pub item: I,
    pub shell: S,
}

impl<I, S> Split<I, S> {
    pub fn new(item: I, shell: S) -> Self {
        Self { item, shell }
    }

    pub fn map<I2, S2, F, G>(self, f: F, g: G) -> Split<I2, S2>
    where
        F: FnOnce(I) -> I2,
        G: FnOnce(S) -> S2,
    {
        Split {
            item: f(self.item),
            shell: g(self.shell),
        }
    }
}
