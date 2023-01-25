// pub mod attach;
// pub mod data;
// pub mod edge;
// pub mod tagged_edge;
// pub mod vertice;

// TODO: Versioned Item. Versioned<I:Item>{version: Version, state: State<I>, item: I}
// TODO  enum State<I> { Active, Removed(Version), Replaced(Key<Versioned<I>>)}

// #[macro_export]
// macro_rules! delegate_item {
//     (impl for $t:ty => $d:ty = $e:tt) => {
//         impl $crate::core::Item for $t {
//             type I<'a> = <$d as $crate::core::Item>::I<'a>;

//             fn references(&self, this: $crate::core::Index) -> Self::I<'_> {
//                 self.$e.references(this)
//             }
//         }

//         impl $crate::core::AnyItem for $t {
//             fn references_any<'a>(
//                 &'a self,
//                 this: $crate::core::Index,
//             ) -> Option<Box<dyn Iterator<Item = $crate::core::AnyRef> + 'a>> {
//                 self.$e.references_any(this)
//             }

//             fn item_removed(
//                 &mut self,
//                 this: $crate::core::Index,
//                 key: $crate::core::AnyKey,
//             ) -> bool {
//                 self.$e.item_removed(this, key)
//             }

//             fn item_moved(&mut self, from: $crate::core::AnyKey, to: $crate::core::AnyKey) {
//                 self.$e.item_moved(from, to)
//             }
//         }
//     };
// }

// ***************************************** Default impl ********************************************* //

/// Implements `Item` for non generic `T` as if T doesn't have any reference.
#[macro_export]
macro_rules! impl_item {
    ($ty:ty) => {
        impl $crate::core::Item for $ty {
            type Alloc = std::alloc::Global;
            type LocalityData = ();
            type Iter<'a> = std::iter::Empty<$crate::core::AnyRef>;

            fn iter_references(&self, _: $crate::core::SlotLocality<'_, Self>) -> Self::Iter<'_> {
                std::iter::empty()
            }

            fn remove_reference(
                &mut self,
                _: $crate::core::SlotLocality<'_, Self>,
                _: $crate::core::AnyKey,
            ) -> bool {
                false
            }

            fn replace_reference(
                &mut self,
                _: $crate::core::SlotLocality<'_, Self>,
                _: $crate::core::AnyKey,
                _: $crate::core::AnyKey,
            ) {
            }

            fn duplicate_reference(
                &mut self,
                _: $crate::core::SlotLocality<'_, Self>,
                _: $crate::core::AnyKey,
                _: $crate::core::AnyKey,
            ) -> Option<$crate::core::Path> {
                None
            }

            fn duplicate(
                &self,
                _: $crate::core::SlotLocality<'_, Self>,
                _: $crate::core::SlotLocality<'_, Self>,
            ) -> Option<Self> {
                Some(self.clone())
            }

            fn displace(
                &mut self,
                _: $crate::core::SlotLocality<'_, Self>,
                _: Option<$crate::core::SlotLocality<'_, Self>>,
            ) {
            }

            item_traits_method!($ty: dyn std::fmt::Debug);
        }
    };
}

impl_item!(());
impl_item!(u8);
impl_item!(u16);
impl_item!(u32);
impl_item!(u64);
impl_item!(u128);
impl_item!(usize);
impl_item!(i8);
impl_item!(i16);
impl_item!(i32);
impl_item!(i64);
impl_item!(i128);
impl_item!(isize);
impl_item!(f32);
impl_item!(f64);
impl_item!(bool);
impl_item!(char);
impl_item!(String);
impl_item!(&'static str);
