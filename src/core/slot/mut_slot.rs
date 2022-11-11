use super::{MutAnySlot, MutItemSlot, MutShellSlot};
use crate::core::{Access, Allocator, AnyItem, ItemsMut, Key, ShellsMut};
use getset::{CopyGetters, Getters};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutSlot<'a, T: AnyItem, C: Access<T>> {
    pub(super) key: Key<T>,
    #[getset(skip)]
    pub(super) item: &'a mut T,
    pub(super) group_item: &'a C::GroupItem,
    #[getset(skip)]
    pub(super) shell: &'a mut C::Shell,
    pub(super) alloc: &'a C::Alloc,
}

impl<'a, T: AnyItem, C: Access<T>> MutSlot<'a, T, C> {
    pub fn item(&self) -> &T {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut T {
        self.item
    }

    pub fn shell(&self) -> &C::Shell {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut C::Shell {
        self.shell
    }

    pub fn split_mut(&mut self) -> (&mut T, &mut C::Shell) {
        (self.item, self.shell)
    }

    pub fn split(self) -> (MutItemSlot<'a, T, C>, MutShellSlot<'a, T, C>)
    where
        C: ItemsMut<T, GroupItem = <C as Access<T>>::GroupItem, Alloc = <C as Allocator<T>>::Alloc>
            + ShellsMut<T, Shell = <C as Access<T>>::Shell, Alloc = <C as Allocator<T>>::Alloc>,
    {
        (
            MutItemSlot {
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
            },
            MutShellSlot {
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
            },
        )
    }

    pub fn upcast(self) -> MutAnySlot<'a, C> {
        MutAnySlot {
            _container: std::marker::PhantomData,
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
