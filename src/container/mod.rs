pub mod all;
pub mod chunked;
pub mod data;
pub mod item;
pub mod table;
pub mod vec;

pub use all::AllContainer;
pub use chunked::{Chunk, Chunked, ChunkingLogic};
pub use data::ContainerData;
pub use item::{ItemContainer, SizedShell};
pub use table::TableContainer;
pub use vec::{VecContainer, VecContainerFamily};

/// Implements Allocator<T> and Container<T> for type C by delegating to it's internal container I through expression e
/// that has access to self to access the field of I.
/// Implements AnyContainer for type C by delegating to it's internal containers through expressions e.
#[macro_export]
macro_rules! delegate_container {
    (impl for $c:ty { $(<$t:ty> => $i:ty = $e:tt;)+}) => {
        $(
            impl $crate::Allocator<$t> for $c
            {
                type Alloc = <$i as Allocator<$t>>::Alloc;

                type R = <$i as Allocator<$t>>::R;

                fn reserve(
                    &mut self,
                    item: Option<&$t>,
                    r: Self::R,
                ) -> Option<($crate::ReservedKey<$t>, &Self::Alloc)> {
                    self.$e.reserve(item, r)
                }

                fn cancel(&mut self, key: $crate::ReservedKey<$t>) {
                    self.$e.cancel(key)
                }

                fn fulfill(&mut self, key: $crate::ReservedKey<$t>, item: $t) -> $crate::SubKey<$t>
                where
                    $t: Sized,
                {
                    self.$e.fulfill(key, item)
                }

                fn unfill(&mut self, key: $crate::SubKey<$t>) -> Option<($t, &Self::Alloc)>
                where
                    $t: Sized,
                {
                    self.$e.unfill(key)
                }
            }

            impl $crate::Container<$t> for $c
            {
                type GroupItem = <$i as Container<$t>>::GroupItem;

                type Shell = <$i as Container<$t>>::Shell;

                type SlotIter<'a> = <$i as Container<$t>>::SlotIter<'a> where Self: 'a;

                fn get_slot(
                    &self,
                    key: $crate::SubKey<$t>,
                ) -> Option<(
                    (&std::cell::SyncUnsafeCell<$t>, &Self::GroupItem),
                    &std::cell::SyncUnsafeCell<Self::Shell>,
                    &Self::Alloc,
                )> {
                    self.$e.get_slot(key)
                }

                unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
                    self.$e.iter_slot()
                }
            }
        )*

        impl $crate::AnyContainer for $c
        {
            fn any_get_slot(
                &self,
                key: $crate::AnySubKey,
            ) -> Option<(
                (&std::cell::SyncUnsafeCell<dyn $crate::AnyItem>, &dyn std::any::Any),
                &std::cell::SyncUnsafeCell<dyn $crate::AnyShell>,
                &dyn std::alloc::Allocator,
            )> {
                $(
                    if std::any::TypeId::of::<$t>() == key.type_id(){
                        return self.$e.any_get_slot(key);
                    }
                )*
                None
            }

            fn unfill_any(&mut self, key: $crate::AnySubKey) {
               $(
                    if std::any::TypeId::of::<$t>() == key.type_id(){
                        return self.$e.unfill_any(key);
                    }
                )*
            }

            fn first(&self, key: std::any::TypeId) -> Option<$crate::AnySubKey> {
                $(
                    if std::any::TypeId::of::<$t>() == key{
                        return self.$e.first(key);
                    }
                )*
                None
            }

            fn next(&self, key: $crate::AnySubKey) -> Option<$crate::AnySubKey> {
                $(
                    if std::any::TypeId::of::<$t>() == key.type_id(){
                        return self.$e.next(key);
                    }
                )*
                None
            }

            fn types(&self) -> std::collections::HashSet<std::any::TypeId> {
                let mut set = std::collections::HashSet::new();
                $(
                    set.insert(std::any::TypeId::of::<$t>());
                )*
                set
            }
        }
    }
}
