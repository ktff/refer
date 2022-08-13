use super::{AnyEntry, AnyKey, Error};

/// Collection over one/multiple types.
/// Should implement relevant/specialized Collection<T> traits.
pub trait AnyCollection {
    fn first_key_any(&self) -> Option<AnyKey>;

    /// Returns following key after given with indices in ascending order.
    /// Order according to type is undefined.
    fn next_key_any(&self, key: AnyKey) -> Option<AnyKey>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - UnsupportedType
    fn get_any<'a>(&'a self, key: AnyKey) -> Result<Box<dyn AnyEntry<'a>>, Error>;
}
