#![allow(type_alias_bounds)]

#[macro_use]
pub mod container;
mod edge;
mod locality;
#[macro_use]
mod item;
mod key;
pub mod permit;
mod slot;

pub use container::{AnyContainer, Container};
pub use edge::*;
pub use item::*;
pub use key::*;
pub use locality::*;
pub use permit::{AddPermit, AnyPermit, Permit, SlotSplitPermit, TypePermit, TypeSplitPermit};

pub use slot::*;

// *************************** Useful aliases *************************** //

// pub type Result<T> = std::result::Result<T, ReferError>;
/*
NOTES
- Goal is to completely prevent memory errors, and to discourage logical errors.

- If a branch is not correct from the point of logic/expectations but the end result is the same then just log the
  the inconsistency and continue. And if the result is not the same return Option/Error. While for
  fatal/unrecoverable/inconsistent_states it should panic.

- Multi level containers must know/enforce levels on their children containers so to have an unique path for each key.

- Containers are not to be Items since that creates non trivial recursions on type and logic levels.
*/

//************************************* CONVENIENT ACCESS ************************************//

#[allow(dead_code)]
fn compile_check<'a, T: Item, C: Container<T>>(key: &Grc<T>, container: &'a mut C) {
    let rf: Key<Ref, T> = key.borrow();

    // AnyPermit
    let mut access = container.access_mut();
    key.get(&access);
    key.get(access.borrow());
    key.get(&access.borrow());
    rf.get(&access);
    rf.get(access.borrow());
    rf.get(&access.borrow());
    key.get(&mut access);
    rf.get(&mut access);
    key.get(access);
    rf.get(container.access_mut());

    // TypePermit
    let mut access = container.access_mut().ty();
    key.get(&access);
    key.get(access.borrow());
    key.get(&access.borrow());
    rf.get(&access);
    rf.get(access.borrow());
    rf.get(&access.borrow());
    key.get(&mut access);
    rf.get(&mut access);
    key.get(access);
    rf.get(container.access_mut().ty());

    // Container
    key.get(&mut *container);
    rf.get(&mut *container);
    key.get(container);
}

/// Enables key.get(permit) access when known Item is guaranteed to exist.
pub trait KeyAccess<'a, R, C, A> {
    type T: Item;
    fn get(&self, access: A) -> Slot<'a, R, Self::T>;
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Ref, C, AnyPermit<'a, permit::Ref, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: AnyPermit<'a, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Ref, C, TypePermit<'a, T, permit::Ref, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: TypePermit<'a, T, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Mut, C, AnyPermit<'a, permit::Mut, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: AnyPermit<'a, permit::Mut, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.ty().key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Mut, C, TypePermit<'a, T, permit::Mut, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: TypePermit<'a, T, permit::Mut, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Mut, C, &'a mut C> for Key<Ref<'a>, T> {
    type T = T;
    fn get(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.access_mut().ty().key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Ref, C, AnyPermit<'a, permit::Ref, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: AnyPermit<'a, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Ref, C, TypePermit<'a, T, permit::Ref, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: TypePermit<'a, T, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Mut, C, AnyPermit<'a, permit::Mut, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: AnyPermit<'a, permit::Mut, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.ty().key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Mut, C, TypePermit<'a, T, permit::Mut, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: TypePermit<'a, T, permit::Mut, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, permit::Mut, C, &'a mut C> for Key<Owned, T> {
    type T = T;
    fn get(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.access_mut().ty().key(self.borrow()).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'b AnyPermit<'a, permit::Ref, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'b AnyPermit<'a, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(*self).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'b TypePermit<'a, T, permit::Ref, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'b TypePermit<'a, T, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Mut, C, &'a mut AnyPermit<'b, permit::Mut, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a mut AnyPermit<'b, permit::Mut, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().ty().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Mut, C, &'a mut TypePermit<'b, T, permit::Mut, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(
        &self,
        access: &'a mut TypePermit<'b, T, permit::Mut, C>,
    ) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'a AnyPermit<'b, permit::Mut, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a AnyPermit<'b, permit::Mut, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().ty().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'a TypePermit<'b, T, permit::Mut, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a TypePermit<'b, T, permit::Mut, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(*self).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'b AnyPermit<'a, permit::Ref, C>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'b AnyPermit<'a, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(self.borrow()).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'b TypePermit<'a, T, permit::Ref, C>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'b TypePermit<'a, T, permit::Ref, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Mut, C, &'a mut AnyPermit<'b, permit::Mut, C>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a mut AnyPermit<'b, permit::Mut, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().ty().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Mut, C, &'a mut TypePermit<'b, T, permit::Mut, C>> for Key<Owned, T>
{
    type T = T;
    fn get(
        &self,
        access: &'a mut TypePermit<'b, T, permit::Mut, C>,
    ) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'a AnyPermit<'b, permit::Mut, C>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a AnyPermit<'b, permit::Mut, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().ty().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, permit::Ref, C, &'a TypePermit<'b, T, permit::Mut, C>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a TypePermit<'b, T, permit::Mut, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(self.borrow()).get()
    }
}
