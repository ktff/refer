use super::RefAnyShellSlot;
use crate::{core::Key, AnyItem, Shell};
use getset::CopyGetters;
use std::{any::Any, ops::Deref};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefShellSlot<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> {
    pub(super) key: Key<T>,
    pub(super) shell: &'a S,
    pub(super) alloc: &'a A,
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> RefShellSlot<'a, T, S, A> {
    pub fn upcast(self) -> RefAnyShellSlot<'a> {
        RefAnyShellSlot {
            key: self.key.upcast(),
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> Deref
    for RefShellSlot<'a, T, S, A>
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.shell
    }
}
