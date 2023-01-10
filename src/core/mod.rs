#![allow(type_alias_bounds)]

pub mod collection;
mod container;
mod context;
mod item;
mod key;
pub mod permit;
mod reference;
mod shell;
mod slot;

use std::{any::TypeId, fmt::Display};

pub use collection::{Access, Collection, Result};
pub use container::*;
pub use context::*;
pub use item::*;
pub use key::*;
pub use permit::{AnyPermit, Permit, SlotSplitPermit, TypePermit, TypeSplitPermit};
pub use reference::*;
pub use shell::*;
pub use slot::*;

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

/*
NOTES
- Goal is to completely prevent memory errors, and to discourage logical errors.

- If a branch is not correct from the point of logic/expectations but the end result is the same then just log the
  the inconsistency and continue. And if the result is not the same return Option/Error. While for
  fatal/unrecoverable/inconsistent_states it should panic.

- Multi level containers must know/enforce levels on their children containers so to have an unique path for each key.

- Containers are not to be Items since that creates non trivial recursions on type and logic levels.
*/

/// Collection level errors.
/// Non fatal in theory but can be fatal in practice.
#[derive(Debug, Clone)]
pub enum CollectionError {
    /// Collection for type and locality is full.
    OutOfKeys { ty: TypeInfo, locality: String },
    /// Item it was representing doesn't exist
    InvalidKey { ty: TypeInfo, key: Index },
    /// Item doesn't support operation.
    InvalidOperation {
        ty: TypeInfo,
        key: Index,
        op: &'static str,
    },
}

impl CollectionError {
    pub fn out_of_keys<T: Item>(locality: T::LocalityKey) -> Self {
        Self::OutOfKeys {
            ty: TypeInfo::of::<T>(None),
            locality: format!("{:?}", locality),
        }
    }

    pub fn invalid_op<T: AnyItem + ?Sized>(key: Key<T>, op: &'static str) -> Self {
        Self::InvalidOperation {
            ty: TypeInfo::of::<T>(Some(key.type_id())),
            key: key.index(),
            op,
        }
    }

    pub fn is_invalid_key<T: AnyItem + ?Sized>(&self, key: Key<T>) -> bool {
        match self {
            Self::InvalidKey {
                ty: TypeInfo { ty, .. },
                key: index,
            } => key.type_id() == *ty && key.index() == *index,
            _ => false,
        }
    }
}

impl<T: AnyItem + ?Sized> From<Key<T>> for CollectionError {
    fn from(key: Key<T>) -> Self {
        Self::InvalidKey {
            ty: TypeInfo::of::<T>(Some(key.type_id())),
            key: key.index(),
        }
    }
}

impl Display for CollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfKeys { ty, locality } => write!(
                f,
                "Collection for type {} and locality {} is full.",
                ty, locality
            ),
            Self::InvalidKey { ty, key } => write!(f, "Item for key {}#{} doesn't exist.", ty, key),
            Self::InvalidOperation { ty, key, op } => {
                write!(
                    f,
                    "Item for key {}#{} doesn't support operation '{}'.",
                    ty, key, op
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeInfo {
    pub ty: TypeId,
    pub ty_name: &'static str,
}

impl TypeInfo {
    pub fn of<T: AnyItem + ?Sized>(ty: Option<TypeId>) -> Self {
        Self {
            ty: ty.unwrap_or_else(|| TypeId::of::<T>()),
            ty_name: std::any::type_name::<T>(),
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", self.ty_name, self.ty)
    }
}
