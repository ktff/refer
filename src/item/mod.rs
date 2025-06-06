// pub mod addon;
pub mod data;
pub mod drain;
pub mod itemize;
pub mod own;
pub mod vec_edges;
pub mod vertice;

// TODO: Wrappers for items
// TODO  * Drain
// TODO  * Standalone
// TODO  * Bi
// TODO  * Edges

// pub use addon::Addon;
// pub use data::Data;
pub use vertice::Vertice;
// pub type Edge<D, T = ()> = Addon<T, 2, D>;

// ***************************************** Default impl ********************************************* //

/// Implements `Item` for `T` without any edges.
#[macro_export]
macro_rules! impl_edgeless_item {
    ($ty:ty) => {
        impl $crate::core::Item for $ty {
            type Edges<'a> = std::iter::Empty<$crate::core::Key<$crate::core::Ref<'a>>>;

            fn iter_edges(&self, _: $crate::core::ItemLocality<'_, Self>) -> Self::Edges<'_> {
                std::iter::empty()
            }

            fn remove_edges<T: $crate::core::DynItem + ?Sized>(
                &mut self,
                _: $crate::core::ItemLocality<'_, Self>,
                _: $crate::core::Key<$crate::core::Ptr, T>,
            ) -> Option<$crate::core::Removed<T>> {
                None
            }

            fn localized_drop(
                self,
                _: $crate::core::ItemLocality<'_, Self>,
            ) -> Vec<$crate::core::Key<$crate::core::Owned>> {
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
