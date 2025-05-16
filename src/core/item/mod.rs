mod any;
mod dym;
mod traits;

pub use any::*;
pub use dym::*;
pub use traits::*;

use super::{Grc, ItemLocality, Key, MultiOwned, Owned, Ptr, Ref};
use std::{alloc::Allocator, any::Any, ptr::Thin};

/// Result of removing an edge.
pub enum Removed<T: DynItem + ?Sized = dyn AnyItem, D = ()> {
    Yes(MultiOwned<T>),
    No(D),
}

/// Structure whose lifetime and edges can be managed by a container/model.
pub trait Item: Sized + Any + Sync + Send + Thin {
    /// Allocator used by item.
    type Alloc: Allocator + Any + Clone + 'static + Send + Sync = std::alloc::Global;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync = ();

    type Edges<'a>: Iterator<Item = Key<Ref<'a>>>;

    /// Traits implemented by Self that others can use to view it as.
    const TRAITS: &'static [ItemTrait<Self>] = &[];

    /// Edges of this item.
    /// Must have stable iteration order.
    fn iter_edges(&self, locality: ItemLocality<'_, Self>) -> Self::Edges<'_>;

    /// Should remove edges to target and return object refs.
    /// Some with removed result.
    /// None if it doesn't exist.
    #[must_use]
    fn remove_edges<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        target: Key<Ptr, T>,
    ) -> Option<Removed<T>>;

    /// Caller should properly dispose of the edges.
    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<Key<Owned>>;
}

/// Item which can be drain.
/// UNSAFE: Implementations MUST follow add_drain_edge SAFETY contract.
pub unsafe trait DrainItem: Item {
    /// SAFETY: add_drain_edge MUST ensure source is returned by `iter_edges` and `localized_drop`.
    /// Callers should create the resulting Key<Owned,Self>.
    fn add_drain_edge(&mut self, locality: ItemLocality<'_, Self>, source: Key<Owned>);

    /// Removes drain edge and returns object ref.
    /// Some success.
    /// None if it doesn't exist.
    #[must_use]
    fn remove_drain_edge(
        &mut self,
        locality: ItemLocality<'_, Self>,
        source: Key,
    ) -> Option<Key<Owned>>;
}

/// Item which can be have bi edges.
/// UNSAFE: Implementations MUST follow add_bi_edge SAFETY contract.
pub unsafe trait BiItem<D, T: DynItem + ?Sized>: Item {
    /// SAFETY: add_bi_edge MUST ensure other is returned by `iter_edges` and `localized_drop`.
    /// Callers should create the resulting Key<Owned,Self>.
    fn add_bi_edge(&mut self, locality: ItemLocality<'_, Self>, data: D, other: Key<Owned, T>);

    /// Removes bi edge and returns object ref.
    /// Some success.
    /// None if it doesn't exist.
    #[must_use]
    fn remove_bi_edge(
        &mut self,
        locality: ItemLocality<'_, Self>,
        data: D,
        other: Key<Ptr, T>,
    ) -> Option<Key<Owned, T>>;
}

/// Item that doesn't depend on any edge so it can have Key<Owned> without edges.
pub trait StandaloneItem: DrainItem {
    #[must_use]
    fn inc_owners(&mut self, locality: ItemLocality<'_, Self>) -> Grc<Self>;

    /// Panics if Grc is not owned by this.
    fn dec_owners(&mut self, locality: ItemLocality<'_, Self>, this: Grc<Self>);

    /// True if there is counted Owned somewhere.
    fn has_owner(&self, locality: ItemLocality<'_, Self>) -> bool;
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

// /// Statically constructs ItemTraits with all of the listed traits, Self, and AnyItem.
// /// An example: `item_traits!(Node<T>: dyn Node);`
// #[macro_export]
// macro_rules! item_traits {
//     ($t:ty: $($tr:ty),*) => {
//         &[
//             $(

//                     (std::any::TypeId::of::<$tr>(),
//                         {const METADATA: <$tr as std::ptr::Pointee>::Metadata = std::ptr::metadata(std::ptr::null::<$t>() as *const $tr);
//                         &METADATA as &(dyn std::any::Any + Send + Sync)}
//                     ),
//             )*
//             (std::any::TypeId::of::<dyn $crate::core::AnyItem>(),
//                 {const METADATA: <dyn $crate::core::AnyItem as std::ptr::Pointee>::Metadata = std::ptr::metadata(std::ptr::null::<$t>() as *const dyn $crate::core::AnyItem);
//                 &METADATA as &(dyn std::any::Any + Send + Sync)}
//             ),
//             (std::any::TypeId::of::<$t>(),&() as &(dyn std::any::Any + Send + Sync)),
//         ] as $crate::core::ItemTraits
//     };
// }
