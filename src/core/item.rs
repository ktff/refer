use super::{AnyKey, AnyRef, Index};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    fmt::Debug,
};

/*
Localized Items vs Global Items
- Localized items depend on their locality. That includes Index, GroupData and Allocator.
- Global items don't depend on their locality.
*/

// /// Marker trait for items that are independent of locality/placement context, they don't have local data.
// pub trait GlobalItem: Item {}

/// Builds an Item in given context.
/// Item and Context must correspond to the same Item type.
pub type ItemBuilder = Box<dyn FnOnce(AnyItemContext) -> Box<dyn Any>>;

/// An item of a model.
pub trait Item: AnyItem + Sized {
    type Alloc: Allocator + Any + Clone + 'static;

    /// Locality of item.
    type Locality: Debug + Copy;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references.
    ///
    /// Must have stable iteration order.
    fn iter_references(&self, context: ItemContext<'_, Self>) -> Self::I<'_>;
}

pub trait AnyItem: Any + Sync + Send {
    /// All internal references.
    ///
    /// Must have stable iteration order.
    fn iter_references_any(
        &self,
        context: AnyItemContext<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    /// True if this should also be removed, else should remove all references to other.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn remove_reference(&mut self, context: AnyItemContext<'_>, other: AnyKey) -> bool;

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn replace_reference(&mut self, context: AnyItemContext<'_>, other: AnyKey, to: Index);

    /// True if this should be duplicated and then replace duplicated reference, else should duplicate all references to other
    /// with to, 1 to 1. If true, fn duplicate must return Some.
    ///
    /// Will be called for references of self, but can be called for other references.
    ///
    /// If false, it's duplicate must also return false.
    fn duplicate_reference(
        &mut self,
        context: AnyItemContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> bool;

    // /// If Some, its locality is tied to given key. // //Also fn displace must return Some.
    // fn preferred_locality(&self, _: AnyItemContext<'_>) -> Option<AnyKey> {
    //     None
    // }

    /// None if it can't be duplicated/cloned.
    // /// If Some, displace must also me Some.
    fn duplicate(&self, _: AnyItemContext<'_>) -> Option<ItemBuilder> {
        None
    }

    // /// None if it can't be displaced/moved.
    // fn displace(&mut self, context: AnyItemContext<'_>) -> Option<ItemBuilder> {
    //     self.duplicate(context).map(|builder| {
    //         self.drop_local(context);
    //         builder
    //     })
    // }

    // /// Localized Items vs Global Items
    // /// - Localized items depend on their context.
    // /// - Global items don't depend on their context.
    // /// Global items can always be moved.
    // fn global() -> bool {
    //     // A safe default.
    //     false
    // }

    /// Drop local data.
    /// Also any remaining reference should be considered invalid.
    ///
    /// If method is not empty, don't make Item Clone, instead use fn duplicate.
    // /// If this method is empty, consider returning true from fn global.
    fn drop_local(&mut self, context: AnyItemContext<'_>);
}

// ******************************** OTHER ******************************** //

#[derive(Clone, Copy)]
pub struct ItemContext<'a, I: Item> {
    locality_data: &'a I::LocalityData,
    allocator: &'a I::Alloc,
}

impl<'a, I: Item> ItemContext<'a, I> {
    pub fn new((locality_data, allocator): (&'a I::LocalityData, &'a I::Alloc)) -> Self {
        Self {
            locality_data,
            allocator,
        }
    }

    pub fn locality_data(&self) -> &'a I::LocalityData {
        self.locality_data
    }

    pub fn allocator(&self) -> &'a I::Alloc {
        self.allocator
    }

    pub fn upcast(self) -> AnyItemContext<'a> {
        AnyItemContext {
            ty: TypeId::of::<I>(),
            locality_data: self.locality_data,
            allocator: self.allocator,
            alloc_any: self.allocator,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AnyItemContext<'a> {
    ty: TypeId,
    locality_data: &'a dyn Any,
    allocator: &'a dyn std::alloc::Allocator,
    alloc_any: &'a dyn Any,
}

impl<'a> AnyItemContext<'a> {
    pub fn new(
        ty: TypeId,
        locality_data: &'a dyn Any,
        allocator: &'a dyn std::alloc::Allocator,
        alloc_any: &'a dyn Any,
    ) -> Self {
        Self {
            ty,
            locality_data,
            allocator,
            alloc_any,
        }
    }

    pub fn locality_data(&self) -> &'a dyn Any {
        self.locality_data
    }

    pub fn allocator(&self) -> &'a dyn std::alloc::Allocator {
        self.allocator
    }

    pub fn downcast<I: Item>(self) -> ItemContext<'a, I> {
        self.downcast_try().expect("Unexpected item type")
    }

    pub fn downcast_try<I: Item>(self) -> Option<ItemContext<'a, I>> {
        if self.ty == TypeId::of::<I>() {
            Some(ItemContext {
                locality_data: self
                    .locality_data
                    .downcast_ref()
                    .expect("Mismatched locality data type"),
                allocator: self
                    .alloc_any
                    .downcast_ref()
                    .expect("Mismatched allocator type"),
            })
        } else {
            None
        }
    }
}
