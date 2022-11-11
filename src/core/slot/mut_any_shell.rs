use super::MutShellSlot;
use crate::{
    core::{AnyItem, AnyKey, AnyShell},
    Access,
};
use getset::{CopyGetters, Getters};
use std::any::Any;

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutAnyShellSlot<'a> {
    pub(super) key: AnyKey,
    #[getset(skip)]
    pub(super) shell: &'a mut dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a> MutAnyShellSlot<'a> {
    pub fn shell(&self) -> &dyn AnyShell {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        self.shell
    }

    /// Additive if called for same `from` multiple times.
    pub fn add_from_any(&mut self, from: AnyKey) {
        self.shell.add_from_any(from, self.alloc);
    }

    pub fn downcast<T: AnyItem, C: Access<T>>(
        self,
    ) -> Result<MutShellSlot<'a, T, C::Shell, C::Alloc>, Self> {
        if let Some(key) = self.key.downcast() {
            if let Some(alloc) = self.alloc_any.downcast_ref() {
                if (self.shell as &dyn Any).is::<C::Shell>() {
                    Ok(MutShellSlot {
                        key,
                        shell: (self.shell as &'a mut dyn Any)
                            .downcast_mut::<C::Shell>()
                            .expect("Should succeed"),
                        alloc,
                    })
                } else {
                    Err(self)
                }
            } else {
                Err(self)
            }
        } else {
            Err(self)
        }
    }
}
