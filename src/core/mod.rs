mod key;

pub use key::*;

/// Enables directed acyclic graph.
///
/// Immutable collection of items that are:
/// - referencable
/// - can reference other items
/// - can fetch items that reference them
pub trait CollectionRef<T: 'static> {
    type E<'a>: Entry<'a, T = T>
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = (Key<T>, Self::E<'a>)> + 'a
    where
        Self: 'a;

    fn add(&mut self, value: T) -> Key<T>;

    // Implementations should specialize this for Composite items.
    fn get<'a>(&'a self, key: Key<T>) -> Option<Self::E<'a>>;

    // NOTE: Since Key is numerical hence countable and storage needs to be able ot check if a key is valid
    // hence iteration is always possible although maybe expensive.

    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}

/// Item that references other items.
pub trait Composite: 'static {
    /// Calls for each reference
    fn references(&self, call: impl FnMut(AnyKey));
}

// Responsibilities of this trait shouldn't be delegated to T.
pub trait Entry<'a>: 'a {
    type T;
    type Iter<T>: Iterator<Item = Key<T>> + 'a;

    fn item(&self) -> &Self::T;

    fn from<T>(&self) -> Self::Iter<T>;
}
