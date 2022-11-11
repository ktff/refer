use super::MutAnyShellSlot;
use crate::core::{AnyItem, AnyKey, Key, Shell};
use getset::{CopyGetters, Getters};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutShellSlot<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> {
    pub(super) key: Key<T>,
    #[getset(skip)]
    pub(super) shell: &'a mut S,
    pub(super) alloc: &'a A,
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> MutShellSlot<'a, T, S, A> {
    pub fn shell(&self) -> &S {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut S {
        self.shell
    }

    /// Additive if called for same `from` multiple times.
    pub fn add_from(&mut self, from: AnyKey) {
        self.shell.add_from(from, self.alloc);
    }

    pub fn upcast(self) -> MutAnyShellSlot<'a> {
        MutAnyShellSlot {
            key: self.key.upcast(),
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> Deref
    for MutShellSlot<'a, T, S, A>
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.shell
    }
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> DerefMut
    for MutShellSlot<'a, T, S, A>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shell
    }
}
