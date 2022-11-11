use super::RefAnyShellSlot;
use crate::core::{AnyShells, Key, Shells};
use getset::CopyGetters;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefShellSlot<'a, T: ?Sized + 'static, C: Shells<T>> {
    pub(super) container: &'a C,
    pub(super) key: Key<T>,
    pub(super) shell: &'a C::Shell,
    pub(super) alloc: &'a C::Alloc,
}

impl<'a, T: ?Sized + 'static, C: Shells<T>> RefShellSlot<'a, T, C> {
    pub fn upcast(self) -> RefAnyShellSlot<'a, C>
    where
        C: AnyShells,
    {
        RefAnyShellSlot {
            container: self.container,
            key: self.key.upcast(),
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
