#![allow(type_alias_bounds)]

#[macro_use]
pub mod container;
mod edge;
mod locality;
#[macro_use]
mod item;
#[cfg(feature = "dag")]
pub mod iter;
mod key;
pub mod permit;
mod slot;

pub use container::{AnyContainer, Container, DynContainer};
pub use edge::*;
pub use item::*;
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
    key.get(&access);
    key.get(access.as_ref());
    key.get(&access.as_ref());
    rf.get(&access);
    rf.get(access.as_ref());
    rf.get(&access.as_ref());
    key.borrow().any().get(&access);
    key.borrow().any().get(access.as_ref());
    key.borrow().any().get(&access.as_ref());
    rf.any().get(&access);
    rf.any().get(access.as_ref());
    rf.any().get(&access.as_ref());
    dy.get(&access);
    dy.get(access.as_ref());
    dy.get(&access.as_ref());
    key.get(&mut access);
    rf.get(&mut access);
    key.borrow().any().get(&mut access);
    rf.any().get(&mut access);
    dy.get(&mut access);
    key.get(access);
    rf.get(container.as_mut());
    key.borrow().any().get(container.as_mut());
    rf.any().get(container.as_mut());
    dy.get(container.as_mut());

    // TypePermit
    let mut access = container.as_mut().ty();
    key.get(&access);
    key.get(access.as_ref());
    key.get(&access.as_ref());
    rf.get(&access);
    rf.get(access.as_ref());
    rf.get(&access.as_ref());
    key.get(&mut access);
    rf.get(&mut access);
    key.get(access);
    rf.get(container.as_mut().ty());

    // Container
    key.get(&mut *container);
    rf.get(&mut *container);
    key.borrow().any().get(&mut *container);
    rf.any().get(&mut *container);
    dy.get(&mut *container);
    key.get(container);
}

/// Enables key.get(permit) access when known Item is guaranteed to exist.
pub trait KeyAccess<'a, C, R, TP, A> {
    type T: DynItem + ?Sized;
    fn get(&self, access: A) -> Slot<'a, R, Self::T>;
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer, P: Permit, TP: Permits<T>>
    KeyAccess<'a, C, P, TP, Access<'a, C, P, TP>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: Access<'a, C, P, TP>) -> Slot<'a, P, Self::T> {
        access.key(*self).fetch()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> KeyAccess<'a, C, permit::Mut, All, &'a mut C>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.as_mut().key(*self).fetch()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer, P: Permit, TP: Permits<T>>
    KeyAccess<'a, C, P, TP, Access<'a, C, P, TP>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: Access<'a, C, P, TP>) -> Slot<'a, P, Self::T> {
        access.key(self.borrow()).fetch()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> KeyAccess<'a, C, permit::Mut, All, &'a mut C>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.as_mut().key(self.borrow()).fetch()
    }
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, C: AnyContainer, TP: Permits<T>>
    KeyAccess<'a, C, permit::Ref, TP, &'b Access<'a, C, permit::Ref, TP>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'b Access<'a, C, permit::Ref, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(*self).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Mut, TP, &'a mut Access<'b, C, permit::Mut, TP>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(
        &self,
        access: &'a mut Access<'b, C, permit::Mut, TP>,
    ) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(*self).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, TP, &'a Access<'b, C, permit::Mut, TP>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a Access<'b, C, permit::Mut, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(*self).fetch()
    }
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, TP, &'b Access<'a, C, permit::Ref, TP>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'b Access<'a, C, permit::Ref, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(self.borrow()).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, TP: Permits<T>, C: AnyContainer>
    KeyAccess<'a, C, permit::Mut, TP, &'a mut Access<'b, C, permit::Mut, TP>> for Key<Owned, T>
{
    type T = T;
    fn get(
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
    fn get(&self, access: &'a Access<'b, C, permit::Mut, TP>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).fetch()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, C: AnyContainer>
    KeyAccess<'a, C, permit::Ref, (), &'a Add<'b, C>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a Add<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).fetch()
    }
}
