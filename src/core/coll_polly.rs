use super::{AnyKey, Error};
/// Collection over multiple types.
/// Should implement relevant/specialized Collection<T> traits.
pub trait PollyCollection {
    type AE<'a>: AnyEntry<'a>
    where
        Self: 'a;

    type IterAny<'a>: Iterator<Item = Self::AE<'a>> + 'a
    where
        Self: 'a;

    /// Errors:
    /// - KeyIsNotInUse
    /// - UnsupportedType
    fn get_any<'a>(&'a self, key: AnyKey) -> Result<Self::AE<'a>, Error>;

    fn iter_any<'a>(&'a self) -> Self::IterAny<'a>;
}

pub trait AnyEntry<'a>: 'a {
    type IterAny: Iterator<Item = AnyKey> + 'a;

    fn key_any(&self) -> AnyKey;

    /// Can change between get's.
    /// Bidirectional references.
    fn from_any(&self) -> Self::IterAny;

    /// True if this item is referenced by other items.
    fn referenced(&self) -> bool;
}
