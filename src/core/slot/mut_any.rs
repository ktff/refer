use super::{MutAnyItemSlot, MutAnyShellSlot, MutSlot};
use crate::core::{Access, AnyItem, AnyKey, AnyShell};
use getset::{CopyGetters, Getters};
use std::any::Any;

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutAnySlot<'a> {
    pub(super) key: AnyKey,
    #[getset(skip)]
    pub(super) item: &'a mut dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    #[getset(skip)]
    pub(super) shell: &'a mut dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a> MutAnySlot<'a> {
    pub fn item(&self) -> &dyn AnyItem {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        self.item
    }

    pub fn shell(&self) -> &dyn AnyShell {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        self.shell
    }

    pub fn split_mut(&mut self) -> (&mut dyn AnyItem, &mut dyn AnyShell) {
        (self.item, self.shell)
    }

    pub fn split(self) -> (MutAnyItemSlot<'a>, MutAnyShellSlot<'a>) {
        (
            MutAnyItemSlot {
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
            MutAnyShellSlot {
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
        )
    }

    pub fn downcast<T: AnyItem, C: Access<T>>(
        self,
    ) -> Result<MutSlot<'a, T, C::GroupItem, C::Shell, C::Alloc>, Self> {
        if let Some(key) = self.key.downcast() {
            if let Some(alloc) = self.alloc_any.downcast_ref() {
                if let Some(group_item) = self.group_item.downcast_ref() {
                    if (self.shell as &dyn Any).is::<C::Shell>() {
                        Ok(MutSlot {
                            key,
                            shell: (self.shell as &'a mut dyn Any)
                                .downcast_mut::<C::Shell>()
                                .expect("Should succeed"),
                            alloc,
                            group_item,
                            item: (self.item as &'a mut dyn Any)
                                .downcast_mut::<T>()
                                .expect("Mismatched key-item types"),
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
        } else {
            Err(self)
        }
    }
}
