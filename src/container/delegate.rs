/// Implements Container<T> and TypeContainer<T> for type C by delegating to it's internal container I through expression e
/// that has access to self to access the field of I.
/// Implements AnyContainer and MultiTypeContainer for type C by delegating to it's internal containers through expressions e
/// and index that must correspond to index of internal container in this container region.
#[macro_export]
macro_rules! delegate_container {
    (impl for $c:ty {<$t:ty> => 0: $i:ty = $e:tt;} in $path:tt) => {
        impl $crate::core::container::TypeContainer<$t> for $c {
            type Sub =  $i;

            #[inline(always)]
            fn get(&self) -> Option<&Self::Sub> {
               Some(&self.$e)
            }

            fn get_mut(&mut self) -> Option<&mut Self::Sub> {
                Some(&mut self.$e)
            }

            fn fill(&mut self) -> &mut Self::Sub{
                &mut self.$e
            }
        }

        impl $crate::core::Container<$t> for $c{
            $crate::single_type_container!(impl Container<$t>);
        }


        impl $crate::core::AnyContainer for $c {
            $crate::single_type_container!(impl AnyContainer<$t>);

            fn container_path(&self) -> Path {
                self.$e.container_path()
            }
        }
    };
    (impl for $c:ty { $(<$t:ty> => $index:tt: $i:ty = $e:tt;)+} in $region:tt) => {
        $(
            impl $crate::core::container::TypeContainer<$t> for $c {
                type Sub =  $i;

                #[inline(always)]
                fn get(&self) -> Option<&Self::Sub> {
                   Some(&self.$e)
                }

                fn get_mut(&mut self) -> Option<&mut Self::Sub> {
                    Some(&mut self.$e)
                }

                fn fill(&mut self) -> &mut Self::Sub{
                    &mut self.$e
                }
            }

            impl $crate::core::Container<$t> for $c{
                $crate::multi_type_container!(impl Container<$t> prefer type);
            }
        )*


        impl $crate::core::container::MultiTypeContainer for $c {
            #[inline(always)]
            fn region(&self) -> $crate::core::RegionPath{
                self.$region
            }

            fn type_to_index(&self, type_id: std::any::TypeId) -> Option<usize>{
                $(
                    if std::any::TypeId::of::<$t>() == type_id{
                        return Some($index);
                    }
                )*
                None

            }

            #[inline(always)]
            fn get_any_index(&self, index: usize) -> Option<&dyn $crate::core::AnyContainer>{
                match index {
                    $(
                       $index => Some(&self.$e as &dyn $crate::core::AnyContainer),
                    )*
                    _ => None
                }
            }

            fn get_mut_any_index(&mut self, index: usize) -> Option<&mut dyn $crate::core::AnyContainer>{
                match index {
                    $(
                       $index => Some(&mut self.$e as &mut dyn $crate::core::AnyContainer),
                    )*
                    _ => None
                }
            }
        }

        impl $crate::core::AnyContainer for $c {
            $crate::multi_type_container!(impl AnyContainer);


            fn types(&self) -> std::collections::HashMap<std::any::TypeId, $crate::core::ItemTraits> {
                let mut set = std::collections::HashMap::new();
                $(
                    set.insert(std::any::TypeId::of::<$t>(),<$t as $crate::core::Item>::traits());
                )*
                set
            }
        }
    };

}

#[cfg(test)]
mod tests {
    use crate::{
        container::VecContainer,
        core::{container::*, *},
    };
    use std::{any::Any, num::NonZeroU32};

    struct ThreeFieldContainer {
        region: RegionPath,
        a: VecContainer<u32>,
        b: VecContainer<bool>,
        c: VecContainer<&'static str>,
    }

    delegate_container!(impl for ThreeFieldContainer {
        <u32> => 0: VecContainer<u32> = a;
        <bool> => 1: VecContainer<bool> = b;
        <&'static str> => 2: VecContainer<&'static str> = c;
    } in region);

    fn container() -> ThreeFieldContainer {
        let region = Path::default().region(NonZeroU32::new(2).unwrap()).unwrap();

        ThreeFieldContainer {
            region,
            a: VecContainer::new(Context::new_default(region.path_of(0).leaf().unwrap())),
            b: VecContainer::new(Context::new_default(region.path_of(1).leaf().unwrap())),
            c: VecContainer::new(Context::new_default(region.path_of(2).leaf().unwrap())),
        }
    }

    #[test]
    fn allocate_multi_type_item() {
        let mut container = container();

        let key_a = container.fill_slot((), 42).unwrap();
        let key_b = container.fill_slot((), true).unwrap();
        let key_c = container.fill_slot((), "Hello").unwrap();

        assert_eq!(
            container.access_mut().slot(key_a).get().unwrap().item(),
            &42
        );
        assert_eq!(
            container.access_mut().slot(key_b).get().unwrap().item(),
            &true
        );
        assert_eq!(
            container.access_mut().slot(key_c).get().unwrap().item(),
            &"Hello"
        );
    }

    #[test]
    fn get_any() {
        let mut container = container();

        let key_a = container.fill_slot((), 42).unwrap();
        let key_b = container.fill_slot((), true).unwrap();
        let key_c = container.fill_slot((), "Hello").unwrap();

        assert_eq!(
            (container
                .access_mut()
                .slot(key_a.any())
                .get_dyn()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&42u32)
        );
        assert_eq!(
            (container
                .access_mut()
                .slot(key_b.any())
                .get_dyn()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&true)
        );
        assert_eq!(
            (container
                .access_mut()
                .slot(key_c.any())
                .get_dyn()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&"Hello")
        );
    }

    struct SingleFieldContainer {
        a: VecContainer<i32>,
    }

    delegate_container!(impl for SingleFieldContainer {
        <i32> => 0: VecContainer<i32> = a;
    } in region);

    fn single_container() -> SingleFieldContainer {
        SingleFieldContainer {
            a: VecContainer::new(Context::new_default(Path::default().leaf().unwrap())),
        }
    }

    #[test]
    fn fill() {
        let mut container = single_container();

        let item = 42;
        let key = container.fill_slot((), item).unwrap();

        assert_eq!(
            container.access_mut().slot(key).get().unwrap().item(),
            &item
        );
        let mut iter = container.access_mut().ty::<i32>().path().iter().unwrap();
        assert_eq!(iter.next().unwrap().item(), &item);
        assert!(iter.next().is_none());
    }

    #[test]
    fn unfill() {
        let mut container = single_container();

        let item = 42;
        let key = container.fill_slot((), item).unwrap();

        assert_eq!(
            container.access_mut().slot(key).get().unwrap().item(),
            &item
        );

        assert_eq!(
            container.unfill_slot(key).map(|(item, _, _)| item),
            Some(item)
        );

        assert!(container.access_mut().slot(key).get().is_err());
        assert_eq!(
            container
                .access_mut()
                .ty::<i32>()
                .path()
                .iter()
                .unwrap()
                .count(),
            0
        );
    }
}
