use crate::core::{Index, Item, Key, Path};
use std::{
    any::{Any, TypeId},
    fmt::Display,
};

use super::{LocalityPath, Ptr};

/// Collection level errors.
/// Non fatal in theory but can be fatal in practice.
#[derive(Debug, Clone)]
pub enum ReferError {
    /// Collection for type and locality is full.
    OutOfKeys { ty: TypeInfo, locality: String },
    /// Expected for Item to be drain.
    ItemNotDrain { ty: TypeInfo, index: Index },
    /// Item it was representing doesn't exist on given path.
    InvalidKey {
        ty: TypeInfo,
        index: Index,
        container: Path,
    },
    InvalidCastType {
        expected: TypeInfo,
        found: TypeInfo,
        index: Index,
    },
    /// Item doesn't support operation.
    InvalidOperation {
        ty: TypeInfo,
        index: Index,
        op: &'static str,
    },
}

impl ReferError {
    pub fn out_of_keys<T: Item>(locality: &impl LocalityPath) -> Self {
        Self::OutOfKeys {
            ty: TypeInfo::of::<T>(),
            locality: format!("{:?}", locality),
        }
    }

    pub fn invalid_key<T: Any + ?Sized>(key: Key<Ptr, T>, container: Path) -> Self {
        Self::InvalidKey {
            ty: TypeInfo::of::<T>(),
            index: key.index(),
            container,
        }
    }

    pub fn invalid_op<T: Any + ?Sized>(key: Key<Ptr, T>, op: &'static str) -> Self {
        Self::InvalidOperation {
            ty: TypeInfo::of::<T>(),
            index: key.index(),
            op,
        }
    }

    pub fn not_drain<T: Any + ?Sized>(key: Key<Ptr, T>) -> Self {
        Self::ItemNotDrain {
            ty: TypeInfo::of::<T>(),
            index: key.index(),
        }
    }

    pub fn is_invalid_key<T: ?Sized + 'static>(&self, key: Key<Ptr, T>) -> bool {
        match self {
            Self::InvalidKey { index, .. } => key.index() == *index,
            _ => false,
        }
    }
}

impl Display for ReferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfKeys { ty, locality } => write!(
                f,
                "Collection for type {} and locality {} is full.",
                ty, locality
            ),
            Self::InvalidKey {
                ty,
                index: key,
                container: path,
            } => write!(
                f,
                "Item on key {}#{} doesn't exist in container {}.",
                ty, key, path
            ),
            Self::InvalidCastType {
                expected,
                found,
                index,
            } => write!(
                f,
                "Item on key {}:{} can't be casted to {}.",
                index, found, expected
            ),
            Self::InvalidOperation { ty, index: key, op } => {
                write!(
                    f,
                    "Item for key {}#{} doesn't support operation '{}'.",
                    ty, key, op
                )
            }
            Self::ItemNotDrain { ty, index: key } => {
                write!(f, "Item on key {}#{} is not drain.", ty, key)
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
    pub fn of<T: ?Sized + 'static>() -> Self {
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
