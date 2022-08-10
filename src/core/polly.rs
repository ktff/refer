use super::{AnyKey, Error, Key};
/// Collection over one/multiple types.
/// Should implement relevant/specialized Collection<T> traits.
pub trait PollyCollection {
    type AE<'a>: AnyEntry<'a, Coll = Self>
    where
        Self: 'a;

    fn first_key(&self) -> Option<AnyKey>;

    /// Returns following key after given in ascending order.
    fn next_key(&self, key: AnyKey) -> Option<AnyKey>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - UnsupportedType
    fn get_any<'a>(&'a self, key: AnyKey) -> Result<Self::AE<'a>, Error>;
}

pub trait AnyEntry<'a>: 'a {
    type IterAny: Iterator<Item = AnyKey> + 'a;
    type Coll;

    fn key_any(&self) -> AnyKey;

    /// Can change between get's.
    /// Bidirectional references.
    fn from_any(&self) -> Self::IterAny;

    /// True if this item is referenced by other items.
    fn referenced(&self) -> bool;

    fn collection(&self) -> &Self::Coll;
}
