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
pub use slot::*;

// *************************** Useful aliases *************************** //
pub type Add<'a, C> = permit::add::AddPermit<'a, C>;
pub type Remove<'a, C> = permit::remove::RemovePermit<'a, C>;
pub type Access<'a, C, R = permit::Ref, T = permit::All, K = permit::All> =
    permit::access::AccessPermit<'a, C, R, T, K>;

pub type MutAccess<'a, C> = Access<'a, C, permit::Mut>;
pub type TypeAccess<'a, C, T> = Access<'a, C, permit::Ref, T>;
pub type MutTypeAccess<'a, C, T> = Access<'a, C, permit::Mut, T>;
pub type ObjectAccess<'a, C, T> = Access<'a, C, permit::Mut, T, permit::Not<Key<Ptr, T>>>;

pub type RefSlot<'a, T> = Slot<'a, permit::Ref, T>;
pub type MutSlot<'a, T> = Slot<'a, permit::Mut, T>;

//************************************* CONVENIENT ACCESS ************************************//

/// Examples of what's possible with KeyAccess/DynKeyAccess traits.
#[allow(dead_code)]
fn compile_check<'a, T: Item, C: Container<T>>(key: &Grc<T>, container: &'a mut C) {
    let rf: Key<Ref, T> = key.borrow();
    let dy = rf.any();

    // Access
    let mut access = container.access_mut();
    key.get(&access);
    key.get(access.borrow());
    key.get(&access.borrow());
    rf.get(&access);
    rf.get(access.borrow());
    rf.get(&access.borrow());
    key.get_dyn(&access);
    key.get_dyn(access.borrow());
    key.get_dyn(&access.borrow());
    rf.get_dyn(&access);
    rf.get_dyn(access.borrow());
    rf.get_dyn(&access.borrow());
    dy.get_dyn(&access);
    dy.get_dyn(access.borrow());
    dy.get_dyn(&access.borrow());
    key.get(&mut access);
    rf.get(&mut access);
    key.get_dyn(&mut access);
    rf.get_dyn(&mut access);
    dy.get_dyn(&mut access);
    key.get(access);
    rf.get(container.access_mut());
    key.get_dyn(container.access_mut());
    rf.get_dyn(container.access_mut());
    dy.get_dyn(container.access_mut());

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
    key.get_dyn(&mut *container);
    rf.get_dyn(&mut *container);
    dy.get_dyn(&mut *container);
    key.get(container);
}

/// Enables key.get(permit) access when known Item is guaranteed to exist.
pub trait KeyAccess<'a, C, R, A> {
    type T: Item;
    fn get(&self, access: A) -> Slot<'a, R, Self::T>;
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, Access<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, TypeAccess<'a, C, T>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: TypeAccess<'a, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, MutAccess<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: MutAccess<'a, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.ty().key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, MutTypeAccess<'a, C, T>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: MutTypeAccess<'a, C, T>) -> Slot<'a, permit::Mut, Self::T> {
        access.key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, &'a mut C> for Key<Ref<'a>, T> {
    type T = T;
    fn get(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.access_mut().ty().key(*self).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, Access<'a, C>> for Key<Owned, T> {
    type T = T;
    fn get(&self, access: Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, TypeAccess<'a, C, T>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: TypeAccess<'a, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, MutAccess<'a, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: MutAccess<'a, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.ty().key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, MutTypeAccess<'a, C, T>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: MutTypeAccess<'a, C, T>) -> Slot<'a, permit::Mut, Self::T> {
        access.key(self.borrow()).get()
    }
}

impl<'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, &'a mut C> for Key<Owned, T> {
    type T = T;
    fn get(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.access_mut().ty().key(self.borrow()).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'b Access<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'b Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(*self).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'b TypeAccess<'a, C, T>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'b TypeAccess<'a, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, &'a mut MutAccess<'b, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a mut MutAccess<'b, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().ty().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, C, permit::Mut, &'a mut MutTypeAccess<'b, C, T>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a mut MutTypeAccess<'b, C, T>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'a MutAccess<'b, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a MutAccess<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().ty().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, C, permit::Ref, &'a MutTypeAccess<'b, C, T>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a MutTypeAccess<'b, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(*self).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'b Access<'a, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'b Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.ty().key(self.borrow()).get()
    }
}

impl<'a: 'b, 'b, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'b TypeAccess<'a, C, T>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'b TypeAccess<'a, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Mut, &'a mut MutAccess<'b, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a mut MutAccess<'b, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().ty().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, C, permit::Mut, &'a mut MutTypeAccess<'b, C, T>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a mut MutTypeAccess<'b, C, T>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'a MutAccess<'b, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a MutAccess<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().ty().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, C, permit::Ref, &'a MutTypeAccess<'b, C, T>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a MutTypeAccess<'b, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.borrow().key(self.borrow()).get()
    }
}

/// Enables key.get(permit) access when known Item is guaranteed to exist.
pub trait DynKeyAccess<'a, C, R, A> {
    type T: DynItem + ?Sized;
    fn get_dyn(&self, access: A) -> DynSlot<'a, R, Self::T>;
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Ref, Access<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: Access<'a, C>) -> DynSlot<'a, permit::Ref, Self::T> {
        access.key(*self).get_dyn()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, MutAccess<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: MutAccess<'a, C>) -> DynSlot<'a, permit::Mut, Self::T> {
        access.key(*self).get_dyn()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, &'a mut C>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut C) -> DynSlot<'a, permit::Mut, Self::T> {
        access.access_mut().key(*self).get_dyn()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Ref, Access<'a, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: Access<'a, C>) -> DynSlot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get_dyn()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, MutAccess<'a, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: MutAccess<'a, C>) -> DynSlot<'a, permit::Mut, Self::T> {
        access.key(self.borrow()).get_dyn()
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, &'a mut C>
    for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut C) -> DynSlot<'a, permit::Mut, Self::T> {
        access.access_mut().key(self.borrow()).get_dyn()
    }
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'b Access<'a, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'b Access<'a, C>) -> DynSlot<'a, permit::Ref, Self::T> {
        access.key(*self).get_dyn()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Mut, &'a mut MutAccess<'b, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut MutAccess<'b, C>) -> DynSlot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(*self).get_dyn()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'a MutAccess<'b, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a MutAccess<'b, C>) -> DynSlot<'a, permit::Ref, Self::T> {
        access.borrow().key(*self).get_dyn()
    }
}

impl<'a: 'b, 'b, T: DynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'b Access<'a, C>> for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'b Access<'a, C>) -> DynSlot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get_dyn()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Mut, &'a mut MutAccess<'b, C>> for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut MutAccess<'b, C>) -> DynSlot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(self.borrow()).get_dyn()
    }
}

impl<'a, 'b: 'a, T: DynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'a MutAccess<'b, C>> for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a MutAccess<'b, C>) -> DynSlot<'a, permit::Ref, Self::T> {
        access.borrow().key(self.borrow()).get_dyn()
    }
}
