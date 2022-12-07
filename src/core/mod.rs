pub mod collection;
mod container;
mod item;
mod key;
// mod layer;
mod context;
mod reference;
mod shell;
mod slot;

use std::{
    any::{Any, TypeId},
    fmt::Display,
};

pub use collection::{Access, Collection};
pub use container::*;
pub use item::*;
pub use key::*;
// pub use layer::*;
pub use context::*;
pub use reference::*;
pub use shell::*;
pub use slot::*;

/*
NOTES

- Goal is to completely prevent memory errors, and to discourage logical errors.

- If a branch is not correct from the point of logic/expectations but the end result is the same then just log the
  the inconsistency and continue. And if the result is not the same return Option/Error. While for
  fatal/unrecoverable/inconsistent_states it should panic.

- Multi level containers must know/enforce levels on their children containers so to have an unique path for each key.

- Containers are not to be Items since that creates non trivial recursions on type and logic levels.

TODO:

   * LocalBox<T>

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
            ty: TypeInfo::of::<T>(),
            locality: format!("{:?}", locality),
        }
    }

    pub fn invalid_op<T: Item>(key: Key<T>, op: &'static str) -> Self {
        Self::InvalidOperation {
            ty: TypeInfo::of::<T>(),
            key: key.index(),
            op,
        }
    }

    pub fn invalid_op_any(key: AnyKey, op: &'static str) -> Self {
        Self::InvalidOperation {
            ty: TypeInfo::any(key.type_id()),
            key: key.index(),
            op,
        }
    }

    pub fn is_invalid_key(&self, key: impl Into<AnyKey>) -> bool {
        match self {
            Self::InvalidKey {
                ty: TypeInfo { ty, .. },
                key: index,
            } => {
                let key = key.into();
                key.type_id() == *ty && key.index() == *index
            }
            _ => false,
        }
    }
}

impl<T: Any> From<Key<T>> for CollectionError {
    fn from(key: Key<T>) -> Self {
        Self::InvalidKey {
            ty: TypeInfo::of::<T>(),
            key: key.index(),
        }
    }
}

impl From<AnyKey> for CollectionError {
    fn from(key: AnyKey) -> Self {
        Self::InvalidKey {
            ty: TypeInfo::any(key.type_id()),
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
    pub fn any(ty: TypeId) -> Self {
        Self { ty, ty_name: "Any" }
    }

    pub fn of<T: Any>() -> Self {
        Self {
            ty: TypeId::of::<T>(),
            ty_name: std::any::type_name::<T>(),
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", self.ty_name, self.ty)
    }
}
