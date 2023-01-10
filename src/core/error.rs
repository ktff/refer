use crate::core::{AnyItem, Index, Item, Key, Path};
use std::{any::TypeId, fmt::Display};

use super::KeyPath;

/// Collection level errors.
/// Non fatal in theory but can be fatal in practice.
#[derive(Debug, Clone)]
pub enum ReferError {
    /// Collection for type and locality is full.
    OutOfKeys { ty: TypeInfo, locality: String },
    /// Item it was representing doesn't exist on given path.
    InvalidKey {
        ty: TypeInfo,
        key: Index,
        container: Path,
    },
    /// Path doesn't exist on given path.
    InvalidPath {
        ty: TypeInfo,
        path: Path,
        container: Path,
    },
    /// Item doesn't support operation.
    InvalidOperation {
        ty: TypeInfo,
        key: Index,
        op: &'static str,
    },
}

impl ReferError {
    pub fn out_of_keys<T: Item>(locality: T::LocalityKey) -> Self {
        Self::OutOfKeys {
            ty: TypeInfo::of::<T>(None),
            locality: format!("{:?}", locality),
        }
    }

    pub fn invalid_key<T: AnyItem + ?Sized>(key: Key<T>, container: Path) -> Self {
        Self::InvalidKey {
            ty: TypeInfo::of::<T>(Some(key.type_id())),
            key: key.index(),
            container,
        }
    }

    pub fn invalid_path<T: AnyItem + ?Sized>(path: KeyPath<T>, container: Path) -> Self {
        Self::InvalidPath {
            ty: TypeInfo::of::<T>(Some(path.type_id())),
            path: path.path(),
            container,
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
                ..
            } => key.type_id() == *ty && key.index() == *index,
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
                key,
                container: path,
            } => write!(
                f,
                "Item for key {}#{} doesn't exist in container {}.",
                ty, key, path
            ),
            Self::InvalidPath {
                ty,
                path,
                container,
            } => write!(
                f,
                "Path {}#{} doesn't exist in container {}.",
                path, ty, container
            ),
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
