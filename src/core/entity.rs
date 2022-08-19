use super::{AnyShell, RefShell};
use std::any::Any;

/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
pub trait AnyEntity<'a>: AnyShell<'a> {
    fn item_any(&self) -> Option<&dyn Any>;
}

pub trait RefEntity<'a>: RefShell<'a> + AnyEntity<'a> {
    fn item(&self) -> &'a Self::T;
}

pub trait MutEntity<'a>: RefShell<'a> + AnyEntity<'a> {
    fn item(&self) -> &Self::T;

    fn item_mut(&mut self) -> &mut Self::T;

    // The point of this is T :?Sized which can have different sizes.
    fn set_copy(&mut self, set: &Self::T)
    where
        Self::T: Copy;
}
