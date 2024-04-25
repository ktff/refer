#![allow(type_alias_bounds)]

#[macro_use]
pub mod container;
mod edge;
mod locality;
#[macro_use]
mod item;
pub mod iter;
mod key;
pub mod permit;
mod slot;

pub use container::{AnyContainer, Container};
pub use edge::*;
pub use item::*;
pub use key::*;
pub use locality::*;
pub use permit::ContainerExt;
pub use slot::*;

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
    key.borrow().any().get_dyn(&access);
    key.borrow().any().get_dyn(access.as_ref());
    key.borrow().any().get_dyn(&access.as_ref());
    rf.any().get_dyn(&access);
    rf.any().get_dyn(access.as_ref());
    rf.any().get_dyn(&access.as_ref());
    dy.get_dyn(&access);
    dy.get_dyn(access.as_ref());
    dy.get_dyn(&access.as_ref());
    key.get(&mut access);
    rf.get(&mut access);
    key.borrow().any().get_dyn(&mut access);
    rf.any().get_dyn(&mut access);
    dy.get_dyn(&mut access);
    key.get(access);
    rf.get(container.as_mut());
    key.borrow().any().get_dyn(container.as_mut());
    rf.any().get_dyn(container.as_mut());
    dy.get_dyn(container.as_mut());

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
    key.borrow().any().get_dyn(&mut *container);
    rf.any().get_dyn(&mut *container);
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
        access.as_mut().ty().key(*self).get()
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
        access.as_mut().ty().key(self.borrow()).get()
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
        access.as_ref().ty().key(*self).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, C, permit::Ref, &'a MutTypeAccess<'b, C, T>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get(&self, access: &'a MutTypeAccess<'b, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(*self).get()
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
        access.as_ref().ty().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>>
    KeyAccess<'a, C, permit::Ref, &'a MutTypeAccess<'b, C, T>> for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a MutTypeAccess<'b, C, T>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).get()
    }
}

impl<'a, 'b: 'a, T: Item, C: Container<T>> KeyAccess<'a, C, permit::Ref, &'a Add<'b, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get(&self, access: &'a Add<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).get()
    }
}

/// Enables key.get(permit) access when known Item is guaranteed to exist.
pub trait DynKeyAccess<'a, C, R, A> {
    type T: DynItem + ?Sized;
    fn get_dyn(&self, access: A) -> Slot<'a, R, Self::T>;
}

impl<'a, T: AnyDynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Ref, Access<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(*self).get_dyn()
    }
}

impl<'a, T: AnyDynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, MutAccess<'a, C>>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: MutAccess<'a, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.key(*self).get_dyn()
    }
}

impl<'a, T: AnyDynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, &'a mut C>
    for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.as_mut().key(*self).get_dyn()
    }
}

impl<'a, T: AnyDynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Ref, Access<'a, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get_dyn()
    }
}

impl<'a, T: AnyDynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, MutAccess<'a, C>>
    for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: MutAccess<'a, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.key(self.borrow()).get_dyn()
    }
}

impl<'a, T: AnyDynItem + ?Sized, C: AnyContainer> DynKeyAccess<'a, C, permit::Mut, &'a mut C>
    for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut C) -> Slot<'a, permit::Mut, Self::T> {
        access.as_mut().key(self.borrow()).get_dyn()
    }
}

impl<'a: 'b, 'b, T: AnyDynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'b Access<'a, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'b Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(*self).get_dyn()
    }
}

impl<'a, 'b: 'a, T: AnyDynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Mut, &'a mut MutAccess<'b, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut MutAccess<'b, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(*self).get_dyn()
    }
}

impl<'a, 'b: 'a, T: AnyDynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'a MutAccess<'b, C>> for Key<Ref<'a>, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a MutAccess<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(*self).get_dyn()
    }
}

impl<'a: 'b, 'b, T: AnyDynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'b Access<'a, C>> for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'b Access<'a, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.key(self.borrow()).get_dyn()
    }
}

impl<'a, 'b: 'a, T: AnyDynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Mut, &'a mut MutAccess<'b, C>> for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a mut MutAccess<'b, C>) -> Slot<'a, permit::Mut, Self::T> {
        access.borrow_mut().key(self.borrow()).get_dyn()
    }
}

impl<'a, 'b: 'a, T: AnyDynItem + ?Sized, C: AnyContainer>
    DynKeyAccess<'a, C, permit::Ref, &'a MutAccess<'b, C>> for Key<Owned, T>
{
    type T = T;
    fn get_dyn(&self, access: &'a MutAccess<'b, C>) -> Slot<'a, permit::Ref, Self::T> {
        access.as_ref().key(self.borrow()).get_dyn()
    }
}
