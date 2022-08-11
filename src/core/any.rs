use super::{AnyEntry, AnyKey, Error, LayerRef};
/// Collection over one/multiple types.
/// Should implement relevant/specialized Collection<T> traits.
pub trait AnyCollection {
    type AE<'a, L: AnyCollection + LayerRef<Down = Self> + 'a>: AnyEntry<'a, Coll = L>
    where
        Self: 'a;

    fn first_key_any(&self) -> Option<AnyKey>;

    /// Returns following key after given with indices in ascending order.
    /// Order according to type is undefined and depends on implementation.
    fn next_key_any(&self, key: AnyKey) -> Option<AnyKey>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - UnsupportedType
    fn get_any<'a, L: AnyCollection + LayerRef<Down = Self> + 'a>(
        top: &'a L,
        key: AnyKey,
    ) -> Result<Self::AE<'a, L>, Error>;

    /// A list of (first,last) keys representing in memory grouped items.
    /// In order of first -> next keys_any
    fn chunks_any(&self) -> Vec<(AnyKey, AnyKey)>;
}
