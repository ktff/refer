pub mod addon;
pub mod data;
pub mod vertice;

pub use addon::Addon;
pub use data::Data;
pub use vertice::Vertice;
pub type Edge<D, T = ()> = Addon<T, 2, D>;

// ***************************************** Default impl ********************************************* //

/// Implements `Item` for `T` without any edges.
#[macro_export]
macro_rules! impl_edgeless_item {
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

impl_edgeless_item!(());
impl_edgeless_item!(u8);
impl_edgeless_item!(u16);
impl_edgeless_item!(u32);
impl_edgeless_item!(u64);
impl_edgeless_item!(u128);
impl_edgeless_item!(usize);
impl_edgeless_item!(i8);
impl_edgeless_item!(i16);
impl_edgeless_item!(i32);
impl_edgeless_item!(i64);
impl_edgeless_item!(i128);
impl_edgeless_item!(isize);
impl_edgeless_item!(f32);
impl_edgeless_item!(f64);
impl_edgeless_item!(bool);
impl_edgeless_item!(char);
impl_edgeless_item!(String);
impl_edgeless_item!(&'static str);
