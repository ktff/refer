#![allow(type_alias_bounds)]

mod container;
mod context;
mod error;
mod item;
mod key;
pub mod permit;
mod reference;
mod shell;
mod slot;

pub use container::*;
pub use context::*;
pub use error::*;
pub use item::*;
pub use key::*;
pub use permit::{
    AnyPermit, ExclusivePermit, Permit, SlotSplitPermit, TypePermit, TypeSplitPermit,
};
pub use reference::*;
pub use shell::*;
pub use slot::*;

// *************************** Useful aliases *************************** //

pub type Result<T> = std::result::Result<T, ReferError>;

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

/*
NOTES
- Goal is to completely prevent memory errors, and to discourage logical errors.

- If a branch is not correct from the point of logic/expectations but the end result is the same then just log the
  the inconsistency and continue. And if the result is not the same return Option/Error. While for
  fatal/unrecoverable/inconsistent_states it should panic.

- Multi level containers must know/enforce levels on their children containers so to have an unique path for each key.

- Containers are not to be Items since that creates non trivial recursions on type and logic levels.
*/
