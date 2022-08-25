use super::{AnyShell, RefShell};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
pub trait AnyEntity<'a>: AnyShell<'a> {
    fn item_any(&self) -> Option<&dyn Any>;
}

pub trait RefEntity<'a>: RefShell<'a> + AnyEntity<'a> {
    fn item(&self) -> &'a Self::T;
}

pub trait MutEntity<'a>: RefShell<'a> + AnyEntity<'a> + Deref<Target = Self::T> + DerefMut {}
