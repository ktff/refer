mod any;

pub use any::*;

use super::{ItemLocality, Key, Owned, PartialEdge, Ptr, Ref, Side};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    ptr::Pointee,
};

pub type ItemTraits = &'static [(TypeId, &'static (dyn Any + Send + Sync))];

/// Marker trait for dyn compliant traits of items.
pub trait DynItem: Any + Pointee {}
impl<T: Any + Pointee + ?Sized> DynItem for T {}

/// Structure whose lifetime and edges can be managed by a container/model.
pub trait Item: Sized + Any + Sync + Send {
    /// Allocator used by item.
    type Alloc: Allocator + Any + Clone + 'static + Send + Sync;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type Edges<'a>: Iterator<Item = PartialEdge<Key<Ref<'a>>>>;

    /// Edges where self is side.
    ///
    /// Must have stable iteration order.
    fn edges(&self, locality: ItemLocality<'_, Self>, side: Option<Side>) -> Self::Edges<'_>;

    /// Should remove edge and return object ref.
    /// Ok success.
    /// Err if can't remove it, which may cause for this item to be removed.
    #[must_use]
    fn try_remove_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        edge: PartialEdge<Key<Ptr, T>>,
    ) -> Result<Key<Owned, T>, Key<Owned, Self>>;

    /// Caller should properly dispose of the edges.
    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<PartialEdge<Key<Owned>>>;

    /// TypeIds of traits with their Metadata that this Item implements.
    /// Including Self and AnyItem.
    /// `item_traits_method!` macro should be used to implement this.
    fn traits() -> ItemTraits;
}

/// Item which can be drain.
pub trait DrainItem: Item {
    #[must_use]
    fn add_drain_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        source: Key<Owned, T>,
    ) -> Key<Owned, Self>;
}

/// Item that doesn't depend on any edge so it can have Key<Owned> without edges.
pub trait StandaloneItem: DrainItem {
    #[must_use]
    fn inc_owners(&mut self, locality: ItemLocality<'_, Self>) -> Key<Owned, Self>;

    fn dec_owners(&mut self, locality: ItemLocality<'_, Self>, this: Key<Owned, Self>);

    /// True if there is counted Owned somewhere.
    fn has_owner(&self, locality: ItemLocality<'_, Self>) -> bool;

    /// Must remove edge and return object ref.
    /// This also means try_remove_edge will always return Some.
    #[must_use]
    fn remove_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        edge: PartialEdge<Key<Ptr, T>>,
    ) -> Key<Owned, T>;
}

pub trait IsStandaloneItem: Item {
    const IS_STANDALONE: bool;
}

impl<I: Item> IsStandaloneItem for I {
    default const IS_STANDALONE: bool = false;
}

impl<I: StandaloneItem> IsStandaloneItem for I {
    const IS_STANDALONE: bool = true;
}

// TODO: Auto include other traits, DrainItem, StandaloneItem, etc.
/// Statically constructs ItemTraits with all of the listed traits, Self, and AnyItem.
/// An example: `item_traits_method!(Node<T>: dyn Node);`
#[macro_export]
macro_rules! item_traits_method {
    ($t:ty: $($tr:ty),*) => {
        fn traits()-> $crate::core::ItemTraits{
            /// Array with traits/Self type name and its metadata.
            static TRAITS: $crate::core::ItemTraits = &[
                $(

                        (std::any::TypeId::of::<$tr>(),
                            {const METADATA: <$tr as std::ptr::Pointee>::Metadata = std::ptr::metadata(std::ptr::null::<$t>() as *const $tr);
                            &METADATA as &(dyn std::any::Any + Send + Sync)}
                        ),
                )*
                (std::any::TypeId::of::<dyn $crate::core::AnyItem>(),
                    {const METADATA: <dyn $crate::core::AnyItem as std::ptr::Pointee>::Metadata = std::ptr::metadata(std::ptr::null::<$t>() as *const dyn $crate::core::AnyItem);
                    &METADATA as &(dyn std::any::Any + Send + Sync)}
                ),
                (std::any::TypeId::of::<$t>(),&() as &(dyn std::any::Any + Send + Sync)),
            ];

            TRAITS
        }
    };
}
