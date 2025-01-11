mod any;
mod dym;
mod traits;

pub use any::*;
pub use dym::*;
pub use traits::*;

use super::{Grc, ItemLocality, Key, MultiOwned, Owned, PartialEdge, Ptr, Ref, Side};
use std::{alloc::Allocator, any::Any, ptr::Thin};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Found {
    Yes,
    No,
}

impl Found {
    pub fn found(found: bool) -> Self {
        if found {
            Self::Yes
        } else {
            Self::No
        }
    }
}

/// Structure whose lifetime and edges can be managed by a container/model.
pub trait Item: Sized + Any + Sync + Send + Thin {
    /// Allocator used by item.
    type Alloc: Allocator + Any + Clone + 'static + Send + Sync = std::alloc::Global;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync = ();

    type Edges<'a>: Iterator<Item = PartialEdge<Key<Ref<'a>>>>;

    /// Traits implemented by Self that others can use to view it as.
    const TRAITS: &'static [ItemTrait<Self>] = &[];

    /// Edges where self is side.
    ///
    /// Must have stable iteration order.
    fn iter_edges(&self, locality: ItemLocality<'_, Self>, side: Option<Side>) -> Self::Edges<'_>;

    // /// Should remove edge and return object ref.
    // /// Ok success.
    // /// Err if can't remove it, which may cause for this item to be removed.
    // #[must_use]
    // fn try_remove_edge<T: DynItem + ?Sized>(
    //     &mut self,
    //     locality: ItemLocality<'_, Self>,
    //     this: Key<Owned, Self>,
    //     edge: PartialEdge<Key<Ptr, T>>,
    // ) -> Result<Key<Owned, T>, (Found, Key<Owned, Self>)>;

    /// Should remove applicable (source,drain,bi) edges and return object refs.
    /// Ok success.
    /// Err if can't remove it, which may cause for this item to be removed.
    #[must_use]
    fn try_remove_edges<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        edge: PartialEdge<Key<Ptr, T>>,
    ) -> Result<MultiOwned<T>, Found>;

    /// Caller should properly dispose of the edges.
    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<PartialEdge<Key<Owned>>>;
}

/// Item which can be drain.
/// UNSAFE: Implementations MUST follow add_drain_edge SAFETY contract.
pub unsafe trait DrainItem: Item {
    /// SAFETY: add_drain_edge MUST ensure to add PartialEdge{object: source,side: Side::Drain} to edges of self.
    /// Callers should create the resulting Key<Owned,Self>.
    fn add_drain_edge(&mut self, locality: ItemLocality<'_, Self>, source: Key<Owned>);

    /// Removes drain edge and returns object ref.
    /// Ok success.
    /// Err if doesn't exist.
    #[must_use]
    fn try_remove_drain_edge(
        &mut self,
        locality: ItemLocality<'_, Self>,
        source: Key,
    ) -> Option<Key<Owned>>;
}

/// Item which can be have bi edges.
/// UNSAFE: Implementations MUST follow add_bi_edge SAFETY contract.
pub unsafe trait BiItem<D, T: DynItem + ?Sized>: Item {
    /// SAFETY: add_bi_edge MUST ensure to add PartialEdge{object: other,side: Side::Bi} to edges of self.
    /// Callers should create the resulting Key<Owned,Self>.
    fn add_bi_edge(&mut self, locality: ItemLocality<'_, Self>, data: D, other: Key<Owned, T>);

    /// Removes bi edge and returns object ref.
    /// Ok success.
    /// Err if doesn't exist.
    #[must_use]
    fn try_remove_bi_edge(
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
