use super::{AnyKey, AnyRef, AnySlotContext, Index, Path, SlotContext};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    fmt::Debug,
    marker::Unsize,
};

/// An item of a model.
pub trait Item: Sized + Any {
    type Alloc: Allocator + Any + Clone + 'static;

    /// Locality of item.
    type LocalityKey: Debug + Copy;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type Iter<'a>: Iterator<Item = AnyRef>;

    /// All internal references.
    ///
    /// Must have stable iteration order.
    fn iter_references(&self, context: SlotContext<'_, Self>) -> Self::Iter<'_>;

    /// True if this should also be removed, else should remove all references to other.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn remove_reference(&mut self, context: SlotContext<'_, Self>, other: AnyKey) -> bool;

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn replace_reference(&mut self, context: SlotContext<'_, Self>, other: AnyKey, to: Index);

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Some if this should be displaced under given prefix.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn displace_reference(
        &mut self,
        context: SlotContext<'_, Self>,
        other: AnyKey,
        to: Index,
    ) -> Option<Path> {
        self.replace_reference(context, other, to);
        None
    }

    /// Some if this should be duplicated under given prefix and then replace duplicated reference in duplicated item,
    /// else should duplicate all references to other with to, 1 to 1.
    ///
    /// If Some, fn duplicate must return Some.
    ///
    /// If None, it's duplicate must also return None.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn duplicate_reference(
        &mut self,
        context: SlotContext<'_, Self>,
        other: AnyKey,
        to: Index,
    ) -> Option<Path>;

    /// Clone this item from context to context.
    /// None if it can't be duplicated/cloned.
    // /// If Some, displace must also me Some.
    fn duplicate(&self, _from: SlotContext<'_, Self>, _to: SlotContext<'_, Self>) -> Option<Self> {
        None
    }

    // /// Localized Items vs Global Items
    // /// - Localized items depend on their context.
    // /// - Global items don't depend on their context.
    // /// Global items can always be moved.
    // fn global() -> bool {
    //     // A safe default.
    //     false
    // }

    /// This is being displaced.
    ///
    /// If not placed in a new context, drop local data and any remaining reference should be considered invalid.
    ///
    /// If method is not empty, don't make Item Clone, instead use fn duplicate.
    // /// If this method is empty, consider returning true from fn global.
    fn displace(&mut self, from: SlotContext<'_, Self>, to: Option<SlotContext<'_, Self>>);
}

/// Marker trait for dyn compliant traits of items.
pub trait DynItem: AnyItem + Unsize<dyn AnyItem> {}
impl<T: AnyItem + Unsize<dyn AnyItem> + ?Sized> DynItem for T {}

/// Methods correspond 1 to 1 to Item methods.
pub trait AnyItem: Any + Unsize<dyn Any> {
    fn item_type_id(self: *const Self) -> TypeId;

    fn iter_references_any(
        &self,
        context: AnySlotContext<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    fn remove_reference_any(&mut self, context: AnySlotContext<'_>, other: AnyKey) -> bool;

    fn replace_reference_any(&mut self, context: AnySlotContext<'_>, other: AnyKey, to: Index);

    fn displace_reference_any(
        &mut self,
        context: AnySlotContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<Path>;

    fn duplicate_reference_any(
        &mut self,
        context: AnySlotContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<Path>;

    fn duplicate_any(
        &self,
        _from: AnySlotContext<'_>,
        _to: AnySlotContext<'_>,
    ) -> Option<Box<dyn AnyItem>>;

    fn displace_any(&mut self, from: AnySlotContext<'_>, to: Option<AnySlotContext<'_>>);
}

impl<T: Item> AnyItem for T {
    // NOTE: This must never be overwritten since it's used for type checking.
    fn item_type_id(self: *const Self) -> TypeId {
        TypeId::of::<T>()
    }

    fn iter_references_any(
        &self,
        context: AnySlotContext<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        let iter = self.iter_references(context.downcast());
        if let (0, Some(0)) = iter.size_hint() {
            None
        } else {
            Some(Box::new(iter))
        }
    }

    fn remove_reference_any(&mut self, context: AnySlotContext<'_>, other: AnyKey) -> bool {
        self.remove_reference(context.downcast(), other)
    }

    fn replace_reference_any(&mut self, context: AnySlotContext<'_>, other: AnyKey, to: Index) {
        self.replace_reference(context.downcast(), other, to)
    }

    fn displace_reference_any(
        &mut self,
        context: AnySlotContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<Path> {
        self.displace_reference(context.downcast(), other, to)
    }

    fn duplicate_reference_any(
        &mut self,
        context: AnySlotContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<Path> {
        self.duplicate_reference(context.downcast(), other, to)
    }

    fn duplicate_any(
        &self,
        from: AnySlotContext<'_>,
        to: AnySlotContext<'_>,
    ) -> Option<Box<dyn AnyItem>> {
        self.duplicate(from.downcast(), to.downcast())
            .map(|x| Box::new(x) as Box<dyn AnyItem>)
    }

    fn displace_any(&mut self, from: AnySlotContext<'_>, to: Option<AnySlotContext<'_>>) {
        self.displace(from.downcast(), to.map(|to| to.downcast()))
    }
}
