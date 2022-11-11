use super::{MutAnyItemSlot, MutAnyShellSlot, MutSlot};
use crate::core::{Access, AnyAccess, AnyItem, AnyItems, AnyKey, AnyShell, AnyShells};
use getset::{CopyGetters, Getters};
use std::{any::Any, marker::PhantomData};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutAnySlot<'a, C: AnyAccess> {
    #[getset(skip)]
    pub(super) _container: PhantomData<&'a mut C>,
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

impl<'a, C: AnyAccess> MutAnySlot<'a, C> {
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

    pub fn split(self) -> (MutAnyItemSlot<'a, C>, MutAnyShellSlot<'a, C>)
    where
        C: AnyItems + AnyShells,
    {
        (
            MutAnyItemSlot {
                _container: self._container,
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
            MutAnyShellSlot {
                _container: self._container,
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
        )
    }

    pub fn downcast<T: AnyItem>(self) -> Result<MutSlot<'a, T, C>, Self>
    where
        C: Access<T>,
    {
        if let Some(key) = self.key.downcast() {
            Ok(MutSlot {
                key,
                item: (self.item as &'a mut dyn Any)
                    .downcast_mut::<T>()
                    .expect("Item of wrong type"),
                group_item: self
                    .group_item
                    .downcast_ref()
                    .expect("Group item of wrong type"),
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
