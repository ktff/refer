use super::{DrainItem, Item, StandaloneItem};
use crate::core::{AnyItemLocality, Key, Owned, PartialEdge, Ref, Side, TypeInfo};
use std::{
    any::{Any, TypeId},
    marker::Unsize,
};

/// Methods supported by any Item.
pub trait AnyItem: Any + Unsize<dyn Any> + Sync {
    fn item_type_id(self: *const Self) -> TypeId;

    fn type_info(self: *const Self) -> TypeInfo;

    fn edges_any(
        &self,
        locality: AnyItemLocality<'_>,
        filter: Option<Side>,
    ) -> Option<Box<dyn Iterator<Item = PartialEdge<Key<Ref<'_>>>> + '_>>;

    /// Ok with key to self.
    /// Err with provided source.
    /// Err if self isn't drain item so it wasn't added.
    #[must_use]
    fn add_drain_edge_any(
        &mut self,
        locality: AnyItemLocality<'_>,
        source: Key<Owned>,
    ) -> Result<Key<Owned>, Key<Owned>>;

    // #[must_use]
    // fn replace_object_any(
    //     &mut self,
    //     locality: AnySlotLocality<'_>,
    //     a: Key,
    //     b: Key<Owned>,
    // ) -> Key<Owned>;

    /// Ok success.
    /// Err if can't remove it.
    #[must_use]
    fn remove_edge_any(
        &mut self,
        locality: AnyItemLocality<'_>,
        this: Key<Owned>,
        edge: PartialEdge<Key>,
    ) -> Result<Key<Owned>, Key<Owned>>;

    // TODO: Use prefix any_ everywhere?

    #[must_use]
    fn any_inc_owners(&mut self, locality: AnyItemLocality<'_>) -> Option<Key<Owned>>;

    fn any_dec_owners(&mut self, locality: AnyItemLocality<'_>, this: Key<Owned>);

    /// True if there is Ref without edge to this item.
    fn any_has_owner(&self, locality: AnyItemLocality<'_>) -> bool;

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

    fn edges_any(
        &self,
        locality: AnyItemLocality<'_>,
        filter: Option<Side>,
    ) -> Option<Box<dyn Iterator<Item = PartialEdge<Key<Ref<'_>>>> + '_>> {
        let edges = self.edges(locality.downcast(), filter);
        if let (0, Some(0)) = edges.size_hint() {
            None
        } else {
            Some(Box::new(edges))
        }
    }

    default fn add_drain_edge_any(
        &mut self,
        _: AnyItemLocality<'_>,
        source: Key<Owned>,
    ) -> Result<Key<Owned>, Key<Owned>> {
        Err(source)
    }

    // fn replace_object_any(
    //     &mut self,
    //     locality: AnySlotLocality<'_>,
    //     a: Key,
    //     b: Key<Owned>,
    // ) -> Key<Owned> {
    //     self.replace_object(locality.downcast(), a, b)
    // }

    fn remove_edge_any(
        &mut self,
        locality: AnyItemLocality<'_>,
        this: Key<Owned>,
        edge: PartialEdge<Key>,
    ) -> Result<Key<Owned>, Key<Owned>> {
        self.try_remove_edge(locality.downcast(), this.assume(), edge)
            .map_err(Key::any)
    }

    default fn any_inc_owners(&mut self, _: AnyItemLocality<'_>) -> Option<Key<Owned>> {
        None
    }

    default fn any_dec_owners(&mut self, _: AnyItemLocality<'_>, _: Key<Owned>) {}

    default fn any_has_owner(&self, _: AnyItemLocality<'_>) -> bool {
        false
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

default impl<T: DrainItem> AnyItem for T {
    fn add_drain_edge_any(
        &mut self,
        locality: AnyItemLocality<'_>,
        source: Key<Owned>,
    ) -> Result<Key<Owned>, Key<Owned>> {
        Ok(self.add_drain_edge(locality.downcast(), source).any())
    }
}

impl<T: StandaloneItem> AnyItem for T {
    fn any_inc_owners(&mut self, locality: AnyItemLocality<'_>) -> Option<Key<Owned>> {
        Some(self.inc_owners(locality.downcast()).any())
    }

    fn any_dec_owners(&mut self, locality: AnyItemLocality<'_>, this: Key<Owned>) {
        self.dec_owners(locality.downcast(), this.assume());
    }

    fn any_has_owner(&self, locality: AnyItemLocality<'_>) -> bool {
        self.has_owner(locality.downcast())
    }
}
