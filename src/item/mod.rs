pub mod edge;
pub mod vertice;

// ***************************************** Default impl ********************************************* //

/// Implements `Item` for non generic `T` as if T doesn't have any reference.
#[macro_export]
macro_rules! impl_item {
    ($ty:ty) => {
        impl crate::core::Item for $ty {
            type I<'a> = std::iter::Empty<crate::core::AnyRef>;

            fn references(&self, _: crate::core::Index) -> Self::I<'_> {
                std::iter::empty()
            }
        }

        impl crate::core::AnyItem for $ty {
            fn references_any(
                &self,
                _: crate::core::Index,
            ) -> Option<Box<dyn Iterator<Item = crate::core::AnyRef> + '_>> {
                None
            }

            fn item_removed(&mut self, _: crate::core::Index, _: crate::core::AnyKey) -> bool {
                true
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
