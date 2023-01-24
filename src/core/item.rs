use super::{AnyKey, AnyRef, AnySlotLocality, Path, SlotLocality, TypeInfo};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    marker::Unsize,
    ptr::Pointee,
};

// TODO: Najradije bi uklonio operaciju move. Ako nije bitno za Item gdje se nalazi onda move nije potreban. A ako je bitno
// TODO  onda njegova poicija je dio njegovog identiteta ili nije. Ako je nije dio identiteta onda se ima drugih opcija, poput delegata, ili versioninga, i slično.
// TODO  A ako je dio identiteta onda promjena koja ga je učinila je velika promjena i bilo bi ok delegirati korisinicima da se sami nose s potrebnim izmjenama. Clone več ima smisla.

pub type ItemTraits = &'static [(TypeId, &'static (dyn Any + Send + Sync))];

/// An item of a model.
pub trait Item: Sized + Any + Sync + Send {
    type Alloc: Allocator + Any + Clone + 'static + Send + Sync;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type Iter<'a>: Iterator<Item = AnyRef>;

    /// All internal references.
    ///
    /// Must have stable iteration order.
    fn iter_references(&self, locality: SlotLocality<'_, Self>) -> Self::Iter<'_>;

    /// True if this should also be removed, else should remove all references to other.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn remove_reference(&mut self, locality: SlotLocality<'_, Self>, other: AnyKey) -> bool;

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn replace_reference(&mut self, locality: SlotLocality<'_, Self>, other: AnyKey, to: AnyKey);

    /// Should replace all of it's references to other with to, 1 to 1.
    ///
    /// Some if this can be displaced under given prefix.
    ///
    /// Will be called for references of self, but can be called for other references.
    fn displace_reference(
        &mut self,
        locality: SlotLocality<'_, Self>,
        other: AnyKey,
        to: AnyKey,
    ) -> Option<Path> {
        self.replace_reference(locality, other, to);
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
        locality: SlotLocality<'_, Self>,
        other: AnyKey,
        to: AnyKey,
    ) -> Option<Path>;

    /// Clone this item from locality to locality.
    /// None if it can't be duplicated/cloned.
    // /// If Some, displace must also me Some.
    fn duplicate(
        &self,
        _from: SlotLocality<'_, Self>,
        _to: SlotLocality<'_, Self>,
    ) -> Option<Self> {
        None
    }

    // /// Localized Items vs Global Items
    // /// - Localized items depend on their locality.
    // /// - Global items don't depend on their locality.
    // /// Global items can always be moved.
    // fn global() -> bool {
    //     // A safe default.
    //     false
    // }

    /// This is being displaced.
    ///
    /// If not placed in a new locality, drop local data and any remaining reference should be considered invalid.
    ///
    /// If method is not empty, don't make Item Clone, instead use fn duplicate.
    // /// If this method is empty, consider returning true from fn global.
    fn displace(&mut self, from: SlotLocality<'_, Self>, to: Option<SlotLocality<'_, Self>>);

    /// TypeIds of traits with their Metadata that this Item implements.
    /// Including Self and AnyItem.
    /// `item_traits_method!` macro should be used to implement this.
    fn traits() -> ItemTraits;
}

/// Marker trait for dyn compliant traits of items.
pub trait DynItem: Any + Pointee {}
impl<T: Any + Pointee + ?Sized> DynItem for T {}

/// Methods correspond 1 to 1 to Item methods.
pub trait AnyItem: Any + Unsize<dyn Any> + Sync {
    fn item_type_id(self: *const Self) -> TypeId;

    fn type_info(self: *const Self) -> TypeInfo;

    fn iter_references_any(
        &self,
        locality: AnySlotLocality<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    fn remove_reference_any(&mut self, locality: AnySlotLocality<'_>, other: AnyKey) -> bool;

    fn replace_reference_any(&mut self, locality: AnySlotLocality<'_>, other: AnyKey, to: AnyKey);

    fn displace_reference_any(
        &mut self,
        locality: AnySlotLocality<'_>,
        other: AnyKey,
        to: AnyKey,
    ) -> Option<Path>;

    fn duplicate_reference_any(
        &mut self,
        locality: AnySlotLocality<'_>,
        other: AnyKey,
        to: AnyKey,
    ) -> Option<Path>;

    fn duplicate_any(
        &self,
        _from: AnySlotLocality<'_>,
        _to: AnySlotLocality<'_>,
    ) -> Option<Box<dyn AnyItem>>;

    fn displace_any(&mut self, from: AnySlotLocality<'_>, to: Option<AnySlotLocality<'_>>);

    /// TypeId<dyn D> -> <dyn D>::Metadata for this item.
    /// Including Self and AnyItem.
    fn trait_metadata(
        self: *const Self,
        dyn_trait: TypeId,
    ) -> Option<&'static (dyn std::any::Any + Send + Sync)>;
}

impl<T: Item> AnyItem for T {
    // NOTE: This must never be overwritten since it's used for type checking.
    fn item_type_id(self: *const Self) -> TypeId {
        TypeId::of::<T>()
    }

    fn type_info(self: *const Self) -> TypeInfo {
        TypeInfo::of::<T>()
    }

    fn iter_references_any(
        &self,
        locality: AnySlotLocality<'_>,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        let iter = self.iter_references(locality.downcast());
        if let (0, Some(0)) = iter.size_hint() {
            None
        } else {
            Some(Box::new(iter))
        }
    }

    fn remove_reference_any(&mut self, locality: AnySlotLocality<'_>, other: AnyKey) -> bool {
        self.remove_reference(locality.downcast(), other)
    }

    fn replace_reference_any(&mut self, locality: AnySlotLocality<'_>, other: AnyKey, to: AnyKey) {
        self.replace_reference(locality.downcast(), other, to)
    }

    fn displace_reference_any(
        &mut self,
        locality: AnySlotLocality<'_>,
        other: AnyKey,
        to: AnyKey,
    ) -> Option<Path> {
        self.displace_reference(locality.downcast(), other, to)
    }

    fn duplicate_reference_any(
        &mut self,
        locality: AnySlotLocality<'_>,
        other: AnyKey,
        to: AnyKey,
    ) -> Option<Path> {
        self.duplicate_reference(locality.downcast(), other, to)
    }

    fn duplicate_any(
        &self,
        from: AnySlotLocality<'_>,
        to: AnySlotLocality<'_>,
    ) -> Option<Box<dyn AnyItem>> {
        self.duplicate(from.downcast(), to.downcast())
            .map(|x| Box::new(x) as Box<dyn AnyItem>)
    }

    fn displace_any(&mut self, from: AnySlotLocality<'_>, to: Option<AnySlotLocality<'_>>) {
        self.displace(from.downcast(), to.map(|to| to.downcast()))
    }

    /// TypeId::of::<dyn D> => <dyn D>::Metadata for this item.
    fn trait_metadata(
        self: *const Self,
        dyn_trait: TypeId,
    ) -> Option<&'static (dyn std::any::Any + Send + Sync)> {
        T::traits()
            .iter()
            .find(|(id, _)| *id == dyn_trait)
            .map(|(_, meta)| *meta)
    }
}

/// Adds static with all of the traits and their metadata.fn Item::traits().
/// Self and AnyItem is always included.
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
