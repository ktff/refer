#![allow(type_alias_bounds)]

mod any_unsafe;
mod dyn_slot;
pub mod permit;
mod slot;
mod unsafe_slot;

pub use any_unsafe::AnyUnsafeSlot;
pub use dyn_slot::{AnySlot, DynSlot};
pub use permit::{AnyPermit, KeySplitPermit, Permit, TypePermit, TypeSplitPermit};
pub use slot::Slot;
pub use unsafe_slot::UnsafeSlot;

use crate::core::{AnyItem, Shell};

use super::KeyPath;

// TODO: Test permit system, test compile failures?

// *************************** Useful aliases *************************** //

pub type MutAnyShells<'a, C> = AnyPermit<'a, permit::Mut, permit::Shell, C>;
pub type MutAnyItems<'a, C> = AnyPermit<'a, permit::Mut, permit::Item, C>;
pub type MutAnySlots<'a, C> = AnyPermit<'a, permit::Mut, permit::Slot, C>;
pub type RefAnyShells<'a, C> = AnyPermit<'a, permit::Ref, permit::Shell, C>;
pub type RefAnyItems<'a, C> = AnyPermit<'a, permit::Ref, permit::Item, C>;
pub type RefAnySlots<'a, C> = AnyPermit<'a, permit::Ref, permit::Slot, C>;

pub type RefSlots<'a, T, C> = TypePermit<'a, T, permit::Ref, permit::Slot, C>;
pub type MutSlots<'a, T, C> = TypePermit<'a, T, permit::Mut, permit::Slot, C>;
pub type RefShells<'a, T, C> = TypePermit<'a, T, permit::Ref, permit::Shell, C>;
pub type MutShells<'a, T, C> = TypePermit<'a, T, permit::Mut, permit::Shell, C>;
pub type RefItems<'a, T, C> = TypePermit<'a, T, permit::Ref, permit::Item, C>;
pub type MutItems<'a, T, C> = TypePermit<'a, T, permit::Mut, permit::Item, C>;

pub type RefSlot<'a, T: AnyItem, S: Shell<T = T>> = Slot<'a, T, S, permit::Ref, permit::Slot>;
pub type MutSlot<'a, T: AnyItem, S: Shell<T = T>> = Slot<'a, T, S, permit::Mut, permit::Slot>;
pub type RefShell<'a, T: AnyItem, S: Shell<T = T>> = Slot<'a, T, S, permit::Ref, permit::Shell>;
pub type MutShell<'a, T: AnyItem, S: Shell<T = T>> = Slot<'a, T, S, permit::Mut, permit::Shell>;
pub type RefItem<'a, T: AnyItem, S: Shell<T = T>> = Slot<'a, T, S, permit::Ref, permit::Item>;
pub type MutItem<'a, T: AnyItem, S: Shell<T = T>> = Slot<'a, T, S, permit::Mut, permit::Item>;
