use super::{DrainItem, Item, StandaloneItem};
use crate::core::{AnyItemLocality, Key, Owned, PartialEdge, Ref, Side};
use std::{
    any::{Any, TypeId},
    fmt::Display,
    marker::Unsize,
};

#[derive(Debug, Clone, Copy)]
pub struct TypeInfo {
    pub ty: TypeId,
    pub name: &'static str,
}

impl TypeInfo {
    pub fn of<T: ?Sized + 'static>() -> Self {
        Self {
            ty: TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", self.name, self.ty)
    }
}

/// Methods supported by any Item.
pub trait AnyItem: Any + Unsize<dyn Any> + Sync {
    fn item_type_id(self: *const Self) -> TypeId;

    fn type_info(self: *const Self) -> TypeInfo;

    fn any_edges(
        &self,
        locality: AnyItemLocality<'_>,
        filter: Option<Side>,
    ) -> Option<Box<dyn Iterator<Item = PartialEdge<Key<Ref<'_>>>> + '_>>;

    /// Ok with key to self.
    /// Err with provided source.
    /// Err if self isn't drain item so it wasn't added.
    #[must_use]
    fn any_add_drain_edge(
        &mut self,
        locality: AnyItemLocality<'_>,
        source: Key<Owned>,
    ) -> Result<Key<Owned>, Key<Owned>>;

    /// Ok success.
    /// Err if can't remove it.
    #[must_use]
    fn any_remove_edge(
        &mut self,
        locality: AnyItemLocality<'_>,
        this: Key<Owned>,
        edge: PartialEdge<Key>,
    ) -> Result<Key<Owned>, Key<Owned>>;

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

    fn any_edges(
        &self,
        locality: AnyItemLocality<'_>,
        filter: Option<Side>,
    ) -> Option<Box<dyn Iterator<Item = PartialEdge<Key<Ref<'_>>>> + '_>> {
        let edges = self.edges(locality.downcast().expect("Unexpected item type"), filter);
        if let (0, Some(0)) = edges.size_hint() {
            None
        } else {
            Some(Box::new(edges))
        }
    }

    default fn any_add_drain_edge(
        &mut self,
        _: AnyItemLocality<'_>,
        source: Key<Owned>,
    ) -> Result<Key<Owned>, Key<Owned>> {
        Err(source)
    }

    fn any_remove_edge(
        &mut self,
        locality: AnyItemLocality<'_>,
        this: Key<Owned>,
        edge: PartialEdge<Key>,
    ) -> Result<Key<Owned>, Key<Owned>> {
        self.try_remove_edge(
            locality.downcast().expect("Unexpected item type"),
            this.assume(),
            edge,
        )
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
    fn any_add_drain_edge(
        &mut self,
        locality: AnyItemLocality<'_>,
        source: Key<Owned>,
    ) -> Result<Key<Owned>, Key<Owned>> {
        Ok(self
            .add_drain_edge(locality.downcast().expect("Unexpected item type"), source)
            .any())
    }
}

impl<T: StandaloneItem> AnyItem for T {
    fn any_inc_owners(&mut self, locality: AnyItemLocality<'_>) -> Option<Key<Owned>> {
        Some(
            self.inc_owners(locality.downcast().expect("Unexpected item type"))
                .any(),
        )
    }

    fn any_dec_owners(&mut self, locality: AnyItemLocality<'_>, this: Key<Owned>) {
        self.dec_owners(
            locality.downcast().expect("Unexpected item type"),
            this.assume(),
        );
    }

    fn any_has_owner(&self, locality: AnyItemLocality<'_>) -> bool {
        self.has_owner(locality.downcast().expect("Unexpected item type"))
    }
}
