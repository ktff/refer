use super::RefShellSlot;
use crate::{
    core::{AnyItem, AnyKey, AnyShell},
    Access,
};
use getset::CopyGetters;
use std::any::Any;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefAnyShellSlot<'a> {
    pub(super) key: AnyKey,
    pub(super) shell: &'a dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a> RefAnyShellSlot<'a> {
    pub fn downcast<T: AnyItem, C: Access<T>>(
        &self,
    ) -> Option<RefShellSlot<'a, T, C::Shell, C::Alloc>> {
        let key = self.key.downcast()?;
        Some(RefShellSlot {
            key,
            shell: (self.shell as &'a dyn Any).downcast_ref()?,
            alloc: self.alloc_any.downcast_ref()?,
        })
    }
}
