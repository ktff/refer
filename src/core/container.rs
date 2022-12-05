//! Containers can panic, if you try to use a key that was not produced at any
//! point by that container.

use super::{
    AnyItem, AnyItemContext, AnyKey, AnySubKey, AnyUnsafeSlot, Item, KeyPrefix, LocalityPrefix,
    Shell, SubKey, UnsafeSlot,
};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    collections::HashSet,
    fmt::Debug,
};

#[derive(Debug, Clone, Copy)]
pub enum ContainerError {
    OutOfKeys,
    UnknownType,
    UndefinedLocality,
    IllegalPlacement,
}

/// A family of containers.
pub trait ContainerFamily: Send + Sync + 'static {
    type C<T: Item>: AnyContainer + 'static;

    fn new<T: Item>(key_len: u32) -> Self::C<T>;
}

/// Fully compliant container for item T.
pub trait Model<T: Item>:
    Container<T, Alloc = T::Alloc, Locality = T::Locality, LocalityData = T::LocalityData>
{
}

impl<
        T: Item,
        C: Container<T, Alloc = T::Alloc, Locality = T::Locality, LocalityData = T::LocalityData>,
    > Model<T> for C
{
}

/// It's responsibility is to allocate/contain/deallocate items and shells, not to manage access to them or
/// to call their methods.
/// TODO: remove this panics.
/// May panic if some inputs don't correspond to this container:
/// - Locality
/// - SubKey
pub trait Container<T: AnyItem>: AnyContainer {
    /// Allocator used for items and shells.
    type Alloc: Allocator + Clone + 'static;

    /// Allocator can select placement for item based on this.
    type Locality: Debug + Copy;

    /// Data of locality.
    type LocalityData: Any;

    /// Shell of item.
    type Shell: Shell<T = T>;

    type SlotIter<'a>: Iterator<
            Item = (
                SubKey<T>,
                UnsafeSlot<'a, T, Self::LocalityData, Self::Shell, Self::Alloc>,
            ),
        > + Send
    where
        Self: 'a;

    /// Iterates in ascending order of key.
    /// No slot is returned twice in returned iterator.
    fn iter_slot(&self) -> Option<Self::SlotIter<'_>>;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<UnsafeSlot<T, Self::LocalityData, Self::Shell, Self::Alloc>>;

    /// None if there is no more place in locality.
    fn fill_slot(&mut self, locality: Self::Locality, item: T) -> Result<SubKey<T>, T>;

    /// Removes from container.
    fn unfill_slot(
        &mut self,
        key: SubKey<T>,
    ) -> Option<(T, Self::Shell, (&Self::LocalityData, &Self::Alloc))>;

    /// Some if locality exists.
    fn locality_to_prefix(&self, locality: Self::Locality) -> Option<LocalityPrefix>;

    /// Selects locality that corresponds to given data.
    fn select_locality(&mut self, locality: Self::Locality) -> LocalityPrefix;

    // // *************** Alt method set
    // /// None if there is no more place for localities.
    // fn fill_locality_2(
    //     &mut self,
    //     locality: Self::Locality,
    // ) -> Option<(KeyPrefix, &Self::LocalityData, &Self::Alloc)>;

    // /// None if there is no more place under prefix.
    // fn fill_slot_2(&mut self, prefix: KeyPrefix, item: T) -> Result<SubKey<T>, T>;
}

/*
    TODO: Revisit naming of locality methods and AnyContainer methods
*/

pub trait AnyContainer: Any + Sync + Send {
    /// None if such locality doesn't exist.
    fn context_any(&self, prefix: LocalityPrefix) -> Option<AnyItemContext>;

    // /// None if there is no more place in localized group or type is unknown.
    // fn fill_slot_any(
    //     &mut self,
    //     ty: TypeId,
    //     near: AnySubKey,
    //     item: Box<dyn Any>,
    // ) -> Option<AnySubKey>;

    /// Err if:
    /// - no more place in localized group
    /// - type is unknown
    /// - locality is undefined
    /// - item can't be placed in locality
    fn fill_slot_any(
        &mut self,
        ty: TypeId,
        locality: LocalityPrefix,
        item: Box<dyn Any>,
    ) -> Result<AnySubKey, ContainerError>;

    /// Chooses fill locality for under given prefix, or enclosing locality.
    fn choose_locality(&mut self, prefix: KeyPrefix) -> LocalityPrefix;

    // fn key_prefix(&self, key: AnyKey) -> Option<KeyPrefix>;

    fn unfill_slot_any(&mut self, key: AnySubKey);

    fn get_slot_any(&self, key: AnySubKey) -> Option<AnyUnsafeSlot>;

    /// Returns first key for given type
    fn first(&self, key: TypeId) -> Option<AnySubKey>;

    /// Returns following key after given in ascending order
    /// for the same type.
    fn next(&self, key: AnySubKey) -> Option<AnySubKey>;

    /// All types in the container.
    fn types(&self) -> HashSet<TypeId>;
}
