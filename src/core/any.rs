use super::{AnyEntry, AnyKey, Error, PathRef};

/// Collection over one/multiple types.
/// Should implement relevant/specialized Collection<T> traits.
pub trait AnyCollection {
    type AE<'a, P: PathRef<'a, Bottom = Self>>: AnyEntry<'a, P>
    where
        Self: 'a,
        P::Top: AnyCollection;

    fn first_key_any(&self) -> Option<AnyKey>;

    /// Returns following key after given with indices in ascending order.
    /// Order according to type is undefined and depends on implementation.
    fn next_key_any(&self, key: AnyKey) -> Option<AnyKey>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - UnsupportedType
    fn get_any<'a, P: PathRef<'a, Bottom = Self>>(
        path: P,
        key: AnyKey,
    ) -> Result<Self::AE<'a, P>, Error>
    where
        P::Top: AnyCollection;
}
