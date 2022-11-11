use super::RefShellSlot;
use crate::core::{AnyItem, AnyKey, AnyShell, AnyShells, Shells};
use getset::CopyGetters;
use std::any::Any;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefAnyShellSlot<'a, C: AnyShells> {
    pub(super) container: &'a C,
    pub(super) key: AnyKey,
    pub(super) shell: &'a dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a, C: AnyShells> RefAnyShellSlot<'a, C> {
    pub fn downcast<T: AnyItem>(&self) -> Option<RefShellSlot<'a, T, C>>
    where
        C: Shells<T>,
    {
        let key = self.key.downcast()?;
        Some(RefShellSlot {
            container: self.container,
            key,
            shell: (self.shell as &'a dyn Any)
                .downcast_ref()
                .expect("Shell of wrong type"),
            alloc: self
                .alloc_any
                .downcast_ref()
                .expect("Allocator of wrong type"),
        })
    }
}
