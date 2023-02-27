use super::{AnyKey, AnySlotLocality, Key, SlotLocality, TypeInfo};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    marker::Unsize,
    ptr::Pointee,
};

/// ! NOTE: Displacement/Move and Clone/Duplication operations should be their own trait.

// TODO: Clone ima samo smisla s itemima koji imaju više verzija. Ako se nema vise verzija onda po samoj naravi je Item jedinstven po svojem mjestu u grafu. To nije istina samo ako se
// TODO  je graf samosličan, primjer je stablo. Ali stablo nije primarni oblik grafa koji se hoće poduprijeti, nijedan oblik zapravo, samo generalni oblik grafa. A što se tiče više verzija
// TODO  cak i tada nije bas uporabljivo. Postoje dvije varijante: 1. Postoji Version item na koji se pokazuje koji se ne klonira stoga je ovo beskorisno. 2. Uistinu se pookazuje na neki
// TODO  Item, ali kako ce to funkcionirati? Item koji pokazuje na dupliciran mora napraviti nesto od sljedečeg: Imati Vec<> s razlicitim verzijama dupliranog. 2. Nista nemijenjati. 3.
// TODO sebe duplicirati, sto nije bas doboro jer tom logikom ce se cijeli graph duplicirati.
// TODO: Clone se da izvesti preko dyn Trait, gdje je zadaca onoga koji klonira item da provjeri pokazatelje na taj item imaju li trait za kloniranje itema preko kojeg se može pitati
// TODO  žele li biti klonirani te izvesti kloniranje. Gurnuti ovu sposobnost u zasebni trait je odlična opcija.

// TODO: Najradije bi uklonio operaciju move. Ako nije bitno za Item gdje se nalazi onda move nije potreban. A ako je bitno
// TODO  onda njegova poicija je dio njegovog identiteta ili nije. Ako je nije dio identiteta onda se ima drugih opcija, poput delegata, ili versioninga, i slično.
// TODO  A ako je dio identiteta onda promjena koja ga je učinila je velika promjena i bilo bi ok delegirati korisinicima da se sami nose s potrebnim izmjenama. Clone več ima smisla.

pub type ItemTraits = &'static [(TypeId, &'static (dyn Any + Send + Sync))];

/*
TODO:
* Potreban je Permit koji ima mjesto za jedan Key.
* Clone i displace extra traitovi.
*/

// TODO: Item folder with files for each core trait.

// TODO: Move edges to separate file

/// Source & Drain model for reference.
/// Edge = source[data] -> drain
pub struct Edge<S: DynItem + ?Sized, D: DynItem + ?Sized> {
    pub source: Key<S>,
    pub drain: Key<D>,
}

pub type AnyPartialEdge = PartialEdge<dyn AnyItem>;

/// Edge where one side is described with key while other by it's side type.
pub struct PartialEdge<T: DynItem + ?Sized>(pub Side, pub Key<T>);

pub enum Side {
    /// Edge source where edge data can be inlined.
    Source,
    /// Edge drain
    Drain,
}

pub trait Item: Sized + Any + Sync + Send {
    // TODO: Can this be unified under Locality data?
    /// Allocator used by item.
    type Alloc: Allocator + Any + Clone + 'static + Send + Sync;

    /// Data shared by local items.
    type LocalityData: Any + Send + Sync;

    type Edges<'a>: Iterator<Item = AnyPartialEdge>;

    // TODO: Is stable order needed?
    /// All internal edges.
    ///
    /// Must have stable iteration order.
    fn edges(&self, locality: SlotLocality<'_, Self>, filter: Option<Side>) -> Self::Edges<'_>;

    // TODO: a & b as Ref<T> ?
    /// Replaces all of it's edges of `a` with `b`, 1 to 1, if self contains such edges.
    fn replace_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: SlotLocality<'_, Self>,
        a: Key<T>,
        b: Key<T>,
    );

    /// Should remove one drain edge/all source edges shared with other, if self contains such edge,
    /// or can return true to remove self.
    fn remove_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: SlotLocality<'_, Self>,
        edge: PartialEdge<T>,
    ) -> bool;

    fn localized_drop(self, locality: SlotLocality<'_, Self>);

    /// TypeIds of traits with their Metadata that this Item implements.
    /// Including Self and AnyItem.
    /// `item_traits_method!` macro should be used to implement this.
    fn traits() -> ItemTraits;
}

/// Item which can be drain.
pub trait DrainItem: Item {
    // TODO: Source as Ref<T> ?
    /// Additive if called for same `source` multiple times.
    fn add_drain_edge<T: DynItem + ?Sized>(
        &mut self,
        locality: SlotLocality<'_, Self>,
        source: Key<T>,
    );
}

/// Marker trait for dyn compliant traits of items.
pub trait DynItem: Any + Pointee {}
impl<T: Any + Pointee + ?Sized> DynItem for T {}

/// Methods supported by any Item.
pub trait AnyItem: Any + Unsize<dyn Any> + Sync {
    fn item_type_id(self: *const Self) -> TypeId;

    fn type_info(self: *const Self) -> TypeInfo;

    fn edges_any(
        &self,
        locality: AnySlotLocality<'_>,
        filter: Option<Side>,
    ) -> Option<Box<dyn Iterator<Item = AnyPartialEdge> + '_>>;

    /// True if edge was added, false if self isn't drain item.
    fn add_drain_edge_any(&mut self, locality: AnySlotLocality<'_>, source: AnyKey) -> bool;

    fn replace_edge_any(&mut self, locality: AnySlotLocality<'_>, a: AnyKey, b: AnyKey);

    fn remove_edge_any(&mut self, locality: AnySlotLocality<'_>, edge: AnyPartialEdge) -> bool;

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
        locality: AnySlotLocality<'_>,
        filter: Option<Side>,
    ) -> Option<Box<dyn Iterator<Item = AnyPartialEdge> + '_>> {
        let edges = self.edges(locality.downcast(), filter);
        if let (0, Some(0)) = edges.size_hint() {
            None
        } else {
            Some(Box::new(edges))
        }
    }

    default fn add_drain_edge_any(&mut self, _: AnySlotLocality<'_>, _: AnyKey) -> bool {
        false
    }

    fn replace_edge_any(&mut self, locality: AnySlotLocality<'_>, a: AnyKey, b: AnyKey) {
        self.replace_edge(locality.downcast(), a, b)
    }

    fn remove_edge_any(&mut self, locality: AnySlotLocality<'_>, edge: AnyPartialEdge) -> bool {
        self.remove_edge(locality.downcast(), edge)
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

impl<T: DrainItem + Item> AnyItem for T {
    fn add_drain_edge_any(&mut self, locality: AnySlotLocality<'_>, source: AnyKey) -> bool {
        self.add_drain_edge(locality.downcast(), source);
        true
    }
}

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
