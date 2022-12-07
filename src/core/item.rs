use super::{AnyItemContext, AnyKey, AnyRef, Index, ItemContext, KeyPrefix};
use getset::{CopyGetters, Getters};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    fmt::Debug,
};

/// An item of a model.
pub trait Item: Sized + Any + Sync + Send {
    type Alloc: Allocator + Any + Clone + 'static;

    /// Locality of item.
    type LocalityKey: Debug + Copy;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type Iter<'a>: Iterator<Item = AnyRef>;

    /// All internal references.
    ///
    /// Must have stable iteration order.
    fn iter_references(&self, context: ItemContext<'_, Self>) -> Self::Iter<'_>;

    /// True if this should also be removed, else should remove all references to other.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn remove_reference(&mut self, context: ItemContext<'_, Self>, other: AnyKey) -> bool;

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn replace_reference(&mut self, context: ItemContext<'_, Self>, other: AnyKey, to: Index);

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Some if this should be displaced under given prefix.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn displace_reference(
        &mut self,
        context: ItemContext<'_, Self>,
        other: AnyKey,
        to: Index,
    ) -> Option<KeyPrefix> {
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
        context: ItemContext<'_, Self>,
        other: AnyKey,
        to: Index,
    ) -> Option<KeyPrefix>;

    /// Clone this item from context to context.
    /// None if it can't be duplicated/cloned.
    // /// If Some, displace must also me Some.
    fn duplicate(&self, _from: ItemContext<'_, Self>, _to: ItemContext<'_, Self>) -> Option<Self> {
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
    fn displace(&mut self, from: ItemContext<'_, Self>, to: Option<ItemContext<'_, Self>>);
}

/// Methods correspond 1 to 1 to Item methods.
pub trait AnyItem: Any + Sync + Send {
    fn iter_references_any(
        &self,
        context: AnyItemContext<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    fn remove_reference_any(&mut self, context: AnyItemContext<'_>, other: AnyKey) -> bool;

    fn replace_reference_any(&mut self, context: AnyItemContext<'_>, other: AnyKey, to: Index);

    fn displace_reference_any(
        &mut self,
        context: AnyItemContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<KeyPrefix> {
        self.replace_reference_any(context, other, to);
        None
    }

    fn duplicate_reference_any(
        &mut self,
        context: AnyItemContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<KeyPrefix>;

    fn duplicate_any(
        &self,
        _from: AnyItemContext<'_>,
        _to: AnyItemContext<'_>,
    ) -> Option<Box<dyn Any>> {
        None
    }

    fn displace_any(&mut self, from: AnyItemContext<'_>, to: Option<AnyItemContext<'_>>);
}

impl<T: Item> AnyItem for T {
    fn iter_references_any(
        &self,
        context: AnyItemContext<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        let iter = self.iter_references(context.downcast());
        if let (0, Some(0)) = iter.size_hint() {
            None
        } else {
            Some(Box::new(iter))
        }
    }

    fn remove_reference_any(&mut self, context: AnyItemContext<'_>, other: AnyKey) -> bool {
        self.remove_reference(context.downcast(), other)
    }

    fn replace_reference_any(&mut self, context: AnyItemContext<'_>, other: AnyKey, to: Index) {
        self.replace_reference(context.downcast(), other, to)
    }

    fn displace_reference_any(
        &mut self,
        context: AnyItemContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<KeyPrefix> {
        self.displace_reference(context.downcast(), other, to)
    }

    fn duplicate_reference_any(
        &mut self,
        context: AnyItemContext<'_>,
        other: AnyKey,
        to: Index,
    ) -> Option<KeyPrefix> {
        self.duplicate_reference(context.downcast(), other, to)
    }

    fn duplicate_any(
        &self,
        from: AnyItemContext<'_>,
        to: AnyItemContext<'_>,
    ) -> Option<Box<dyn Any>> {
        self.duplicate(from.downcast(), to.downcast())
            .map(|x| Box::new(x) as Box<dyn Any>)
    }

    fn displace_any(&mut self, from: AnyItemContext<'_>, to: Option<AnyItemContext<'_>>) {
        self.displace(from.downcast(), to.map(|to| to.downcast()))
    }
}
