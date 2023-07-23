mod any;
mod traits;

pub use any::*;
pub use traits::*;

use super::{Grc, ItemLocality, Key, MultiOwned, Owned, PartialEdge, Ptr, Ref, Side};
use std::{alloc::Allocator, any::Any, ptr::Pointee};

/// Marker trait for dyn compliant traits of items.
pub trait DynItem: Any + Pointee {}
impl<T: Any + Pointee + ?Sized> DynItem for T {}

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
pub trait Item: Sized + Any + Sync + Send {
    /// Allocator used by item.
    type Alloc: Allocator + Any + Clone + 'static + Send + Sync;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type Edges<'a>: Iterator<Item = PartialEdge<Key<Ref<'a>>>>;

    /// Traits implemented by Self that others can use to view it as.
    const TRAITS: &'static [ItemTrait<Self>];

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
    ) -> Result<Key<Owned, T>, (Found, Key<Owned, Self>)>;

    /// Caller should properly dispose of the edges.
    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<PartialEdge<Key<Owned>>>;
}

/// Item which can be drain.
/// UNSAFE: Implementations MUST follow add_drain_edge SAFETY contract.
pub unsafe trait DrainItem: Item {
    /// SAFETY: add_drain_edge MUST ensure to add PartialEdge{object: source,side: Side::Drain} to edges of self.
    /// Callers should create the resulting Key<Owned,Self>.
    fn add_drain_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        source: Key<Owned, T>,
    );
}

/// Item that doesn't depend on any edge so it can have Key<Owned> without edges.
pub trait StandaloneItem: DrainItem {
    #[must_use]
    fn inc_owners(&mut self, locality: ItemLocality<'_, Self>) -> Grc<Self>;

    /// Panics if Grc is not owned by this.
    fn dec_owners(&mut self, locality: ItemLocality<'_, Self>, this: Grc<Self>);

    /// True if there is counted Owned somewhere.
    fn has_owner(&self, locality: ItemLocality<'_, Self>) -> bool;

    /// Must remove edge and return object ref.
    /// This also means try_remove_edge will always return Some.
    /// Panics if edge is not found.
    #[must_use]
    fn remove_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        edge: PartialEdge<Key<Ptr, T>>,
    ) -> Key<Owned, T> {
        self.try_remove_edge(locality, this, edge)
            .expect("Method invariant broken")
    }
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
