use super::{AnyShell, Item, RefShell};
use std::any::Any;

/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
pub trait AnyEntity<'a>: AnyShell<'a> {
    fn item_any(&self) -> Option<&dyn Any>;
}

pub trait RefEntity<'a, I: ?Sized + 'static>: RefShell<'a, I> + AnyEntity<'a> {
    fn item(&self) -> &'a I;
}

pub trait MutEntity<'a, I: ?Sized + 'static>: RefShell<'a, I> + AnyEntity<'a> {
    fn item(&self) -> &I;

    fn item_mut(&mut self) -> &mut I;

    /// The point of this is that it will update references.
    /// Err if some of the references don't exist.
    fn set(&mut self, set: I) -> Result<I, I>
    where
        I: Item + Sized;

    /// The point of this is I :?Sized which can have different sizes
    /// True if set. False if some of it's references don't exist.
    fn set_copy(&mut self, set: &I) -> bool
    where
        I: Item + Copy;
}
