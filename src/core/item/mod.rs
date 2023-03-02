mod any;

pub use any::*;

use super::{Key, Owned, PartialEdge, Ptr, Ref, Side, SlotLocality};
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

    type DroppedEdges<'a>: Iterator<Item = PartialEdge<Key<Owned>>> + 'a;

    /// Edges where self is side.
    ///
    /// Must have stable iteration order.
    fn edges(&self, locality: SlotLocality<'_, Self>, side: Option<Side>) -> Self::Edges<'_>;

    /// Replace one Ref of a with b and return replaced Ref.
    fn replace_object<T: DynItem + ?Sized>(
        &mut self,
        locality: SlotLocality<'_, Self>,
        a: Key<Ptr, T>,
        b: Key<Owned, T>,
    ) -> Key<Owned, T>;

    /// Should remove edge and return object ref.
    /// On None, localized_drop should be called.
    #[must_use]
    fn try_remove_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: SlotLocality<'_, Self>,
        this: Key<Owned, Self>,
        edge: PartialEdge<Key<Ptr, T>>,
    ) -> Option<Key<Owned, T>>;

    /// Caller should properly dispose of the edges.
    fn localized_drop(self, locality: SlotLocality<'_, Self>) -> Self::DroppedEdges<'_>;

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
        locality: SlotLocality<'_, Self>,
        source: Key<Owned, T>,
    ) -> Key<Owned, Self>;
}

/// Item that doesn't depend on any edge so it can have Refs without edges.
pub trait StandaloneItem: DrainItem {
    #[must_use]
    fn create_ref(&mut self, locality: SlotLocality<'_, Self>) -> Key<Owned, Self>;

    fn delete_ref(&mut self, locality: SlotLocality<'_, Self>, this: Key<Owned, Self>);

    /// True if there is Ref without edge to this item.
    fn edgeless_ref(&self, locality: SlotLocality<'_, Self>) -> bool;

    /// Must remove edge and return object ref.
    /// This also means try_remove_edge will always return Some.
    #[must_use]
    fn remove_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: SlotLocality<'_, Self>,
        this: Key<Owned, Self>,
        edge: PartialEdge<Key<Ptr, T>>,
    ) -> Key<Owned, T>;
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
