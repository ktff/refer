use super::{AnyKey, Error, Key};
/// Collection over one/multiple types.
/// Should implement relevant/specialized Collection<T> traits.
pub trait AnyCollection {
    type AE<'a>: AnyEntry<'a, Coll = Self>
    where
        Self: 'a;

    fn first_key_any(&self) -> Option<AnyKey>;

    /// Returns following key after given with indices in ascending order.
    /// Order according to type is undefined and depends on implementation.
    fn next_key_any(&self, key: AnyKey) -> Option<AnyKey>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - UnsupportedType
    fn get_any<'a>(&'a self, key: AnyKey) -> Result<Self::AE<'a>, Error>;

    /// A list of (first,last) keys representing in memory grouped items.
    fn chunks_any(&self) -> Vec<(AnyKey, AnyKey)>;
}

pub trait AnyEntry<'a>: 'a {
    type IterAny: Iterator<Item = AnyKey> + 'a;
    type Coll: ?Sized;

    fn key_any(&self) -> AnyKey;

    /// Bidirectional references.
    fn from_any(&self) -> Self::IterAny;

    /// True if this item is referenced by other items.
    fn referenced(&self) -> bool;

    fn collection(&self) -> &Self::Coll;
}
