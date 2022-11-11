use super::MutAnyShellSlot;
use crate::core::{Key, ShellsMut};
use getset::{CopyGetters, Getters};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutShellSlot<'a, T: ?Sized + 'static, C: ShellsMut<T>> {
    pub(super) key: Key<T>,
    #[getset(skip)]
    pub(super) shell: &'a mut C::Shell,
    pub(super) alloc: &'a C::Alloc,
}

impl<'a, T: ?Sized + 'static, C: ShellsMut<T>> MutShellSlot<'a, T, C> {
    pub fn shell(&self) -> &C::Shell {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut C::Shell {
        self.shell
    }

    pub fn upcast(self) -> MutAnyShellSlot<'a, C> {
        MutAnyShellSlot {
            _container: std::marker::PhantomData,
            key: self.key.upcast(),
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
