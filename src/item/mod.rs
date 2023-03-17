pub mod addon;
pub mod data;
// pub mod edge;
// pub mod tagged_edge;
pub mod vertice;

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
            type Edges<'a> = std::iter::Empty<
                $crate::core::PartialEdge<$crate::core::Key<$crate::core::Ref<'a>>>,
            >;
            const TRAITS: $crate::core::ItemTraits<$ty> = &[];

            fn edges(
                &self,
                _: $crate::core::ItemLocality<'_, Self>,
                _: Option<$crate::core::Side>,
            ) -> Self::Edges<'_> {
                std::iter::empty()
            }

            #[must_use]
            fn try_remove_edge<T: $crate::core::DynItem + ?Sized>(
                &mut self,
                _: $crate::core::ItemLocality<'_, Self>,
                this: $crate::core::Key<$crate::core::Owned, Self>,
                _: $crate::core::PartialEdge<$crate::core::Key<$crate::core::Ptr, T>>,
            ) -> Result<
                $crate::core::Key<$crate::core::Owned, T>,
                (
                    $crate::core::Found,
                    $crate::core::Key<$crate::core::Owned, Self>,
                ),
            > {
                Err(($crate::core::Found::No, this))
            }

            fn localized_drop(
                self,
                _: $crate::core::ItemLocality<'_, Self>,
            ) -> Vec<$crate::core::PartialEdge<$crate::core::Key<$crate::core::Owned>>> {
                Vec::new()
            }
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
