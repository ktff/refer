#![allow(type_alias_bounds)]

#[macro_use]
pub mod container;
mod locality;
#[macro_use]
mod item;
#[cfg(feature = "dag")]
pub mod iter;
mod key;
pub mod permit;
mod slot;

pub use container::{AnyContainer, Container, DynContainer};
pub use item::*;
#[cfg(feature = "dag")]
pub use iter::{IterNode, VisitDAG};
pub use key::*;
pub use locality::*;
pub use permit::ContainerExt;
pub use slot::*;

use permit::{access::Permits, All, Permit};

// *************************** Useful aliases *************************** //
pub type Remove<'a, C> = &'a mut C;
pub type Add<'a, C> = permit::add::AddAccess<'a, C>;
pub type Access<'a, C, R = permit::Ref, T = permit::All, K = permit::All> =
    permit::access::Access<'a, C, R, T, K>;

pub type MutAccess<'a, C> = Access<'a, C, permit::Mut>;
pub type TypeAccess<'a, C, T> = Access<'a, C, permit::Ref, T>;
pub type MutTypeAccess<'a, C, T> = Access<'a, C, permit::Mut, T>;
pub type ObjectAccess<'a, C, T> = Access<'a, C, permit::Mut, T, permit::Not<Key<Ptr, T>>>;

pub type RefSlot<'a, T> = Slot<'a, permit::Ref, T>;
pub type MutSlot<'a, T> = Slot<'a, permit::Mut, T>;
pub type DynMutSlot<'a, T: ?Sized> = Slot<'a, permit::Mut, T>;

//************************************* CONVENIENT ACCESS ************************************//

/// Examples of what's possible with KeyAccess/DynKeyAccess traits.
#[allow(dead_code)]
fn compile_check<'a, T: Item, C: Container<T>>(key: &Grc<T>, container: &'a mut C) {
    let rf: Key<Ref, T> = key.borrow();
    let dy = rf.any();

    // Access
    let mut access = container.as_mut();
    key.from(&access);
    key.from(access.as_ref());
    key.from(&access.as_ref());
    rf.from(&access);
    rf.from(access.as_ref());
    rf.from(&access.as_ref());
    key.borrow().any().from(&access);
    key.borrow().any().from(access.as_ref());
    key.borrow().any().from(&access.as_ref());
    rf.any().from(&access);
    rf.any().from(access.as_ref());
    rf.any().from(&access.as_ref());
    dy.from(&access);
    dy.from(access.as_ref());
    dy.from(&access.as_ref());
    key.from(&mut access);
    rf.from(&mut access);
    key.borrow().any().from(&mut access);
    rf.any().from(&mut access);
    dy.from(&mut access);
    key.from(access);
    rf.from(container.as_mut());
    key.borrow().any().from(container.as_mut());
    rf.any().from(container.as_mut());
    dy.from(container.as_mut());

    // TypePermit
    let mut access = container.as_mut().ty();
    key.from(&access);
    key.from(access.as_ref());
    key.from(&access.as_ref());
    rf.from(&access);
    rf.from(access.as_ref());
    rf.from(&access.as_ref());
    key.from(&mut access);
    rf.from(&mut access);
    key.from(access);
    rf.from(container.as_mut().ty());

    // Container
    key.from(&mut *container);
    rf.from(&mut *container);
    key.borrow().any().from(&mut *container);
    rf.any().from(&mut *container);
    dy.from(&mut *container);
    key.from(container);
}

/// Enables key.get(permit) access when known Item is guaranteed to exist.
pub trait KeyAccess<'a, C, R, TP, A> {
    type T: DynItem + ?Sized;
    fn from(&self, access: A) -> Slot<'a, R, Self::T>;
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, C: AnyContainer, P: Permit, TP: Permits<T>>
    KeyAccess<'a, C, P, TP, Access<'a, C, P, TP>> for Key<Ref<'b>, T>
{
    type T = T;
    fn from(&self, access: Access<'a, C, P, TP>) -> Slot<'a, P, Self::T> {
        access.key(*self).fetch()
    }
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, C: AnyContainer> KeyAccess<'a, C, permit::Mut, All, &'a mut C>
    for Key<Ref<'b>, T>
{
    type T = T;
    fn from(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.as_mut().key(*self).fetch()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer, P: Permit, TP: Permits<T>>
    KeyAccess<'a, C, P, TP, Access<'a, C, P, TP>> for Key<Owned, T>
{
    type T = T;
    fn from(&self, access: Access<'a, C, P, TP>) -> Slot<'a, P, Self::T> {
        access.key(self.borrow()).fetch()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> KeyAccess<'a, C, permit::Mut, All, &'a mut C>
    for Key<Owned, T>
{
    type T = T;
    fn from(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.as_mut().key(self.borrow()).fetch()
    }
}

impl<'a: 'b + 'c, 'b, 'c, T: DynItem + ?Sized, C: AnyContainer, TP: Permits<T>>
    KeyAccess<'a, C, permit::Ref, TP, &'b Access<'a, C, permit::Ref, TP>> for Key<Ref<'c>, T>
{
    type T = T;
    fn from(&self, access: &'b Access<'a, C, permit::Ref, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(*self).fetch()
    }
}

impl<'a, 'b: 'a + 'c, 'c, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Mut, TP, &'a mut Access<'b, C, permit::Mut, TP>> for Key<Ref<'c>, T>
{
    type T = T;
    fn from(
        &self,
        access: &'a mut Access<'b, C, permit::Mut, TP>,
    ) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(*self).fetch()
    }
}

impl<'a, 'b: 'a + 'c, 'c, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, TP, &'a Access<'b, C, permit::Mut, TP>> for Key<Ref<'c>, T>
{
    type T = T;
    fn from(&self, access: &'a Access<'b, C, permit::Mut, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(*self).fetch()
    }
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, TP, &'b Access<'a, C, permit::Ref, TP>> for Key<Owned, T>
{
    type T = T;
    fn from(&self, access: &'b Access<'a, C, permit::Ref, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(self.borrow()).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Mut, TP, &'a mut Access<'b, C, permit::Mut, TP>> for Key<Owned, T>
{
    type T = T;
    fn from(
        &self,
        access: &'a mut Access<'b, C, permit::Mut, TP>,
    ) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(self.borrow()).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, TP, &'a Access<'b, C, permit::Mut, TP>> for Key<Owned, T>
{
    type T = T;
    fn from(&self, access: &'a Access<'b, C, permit::Mut, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, (), &'a Add<'b, C>> for Key<Owned, T>
{
    type T = T;
    fn from(&self, access: &'a Add<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).fetch()
    }
}

pub trait GraphAccess<'a, T: DynItem + ?Sized> {
    fn get(self, key: Key<Ref, T>) -> Slot<'a, permit::Ref, T>;
}

impl<'a, T: Item, C: Container<T>> GraphAccess<'a, T> for &'a mut C {
    fn get(self, key: Key<Ref, T>) -> Slot<'a, permit::Ref, T> {
        key.from(self.as_ref())
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>> GraphAccess<'b, T> for &'b Add<'a, C> {
    fn get(self, key: Key<Ref, T>) -> Slot<'b, permit::Ref, T> {
        key.from(self.as_ref())
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>, TP: Permits<T>> GraphAccess<'b, T>
    for &'b Access<'a, C, permit::Mut, TP>
{
    fn get(self, key: Key<Ref, T>) -> Slot<'b, permit::Ref, T> {
        key.from(self.as_ref())
    }
}

impl<'a: 'b, 'b, C: Container<T>, TP: Permits<T>, T: Item> GraphAccess<'a, T>
    for &'b Access<'a, C, permit::Ref, TP>
{
    fn get(self, key: Key<Ref, T>) -> Slot<'a, permit::Ref, T> {
        key.from(self)
    }
}

pub trait MutGraphAccess<'a, T: DynItem + ?Sized> {
    fn slot_mut(self, key: Key<Ref, T>) -> Slot<'a, permit::Mut, T>;
}

impl<'a: 'b, 'b, T: Item, C: Container<T>> MutGraphAccess<'b, T> for &'b mut Add<'a, C> {
    fn slot_mut(self, key: Key<Ref, T>) -> Slot<'b, permit::Mut, T> {
        key.from(self.as_mut())
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>, TP: Permits<T>> MutGraphAccess<'b, T>
    for &'b mut Access<'a, C, permit::Mut, TP>
{
    fn slot_mut(self, key: Key<Ref, T>) -> Slot<'b, permit::Mut, T> {
        key.from(self)
    }
}
