use super::MutShellSlot;
use crate::core::{AnyItem, AnyKey, AnyShell, AnyShells, ShellsMut};
use getset::{CopyGetters, Getters};
use std::{any::Any, marker::PhantomData};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutAnyShellSlot<'a, C: AnyShells> {
    #[getset(skip)]
    pub(super) _container: PhantomData<&'a mut C>,
    pub(super) key: AnyKey,
    #[getset(skip)]
    pub(super) shell: &'a mut dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a, C: AnyShells> MutAnyShellSlot<'a, C> {
    pub fn shell(&self) -> &dyn AnyShell {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        self.shell
    }

    pub fn downcast<T: AnyItem>(self) -> Result<MutShellSlot<'a, T, C>, Self>
    where
        C: ShellsMut<T>,
    {
        if let Some(key) = self.key.downcast() {
            Ok(MutShellSlot {
                key,
                shell: (self.shell as &'a mut dyn Any)
                    .downcast_mut()
                    .expect("Shell of wrong type"),
                alloc: self
                    .alloc_any
                    .downcast_ref()
                    .expect("Allocator of wrong type"),
            })
        } else {
            Err(self)
        }
    }
}
