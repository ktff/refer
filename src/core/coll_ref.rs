use super::{Error, Key};

/// Enables directed acyclic graph.
///
/// Immutable collection of items that are:
/// - referencable
/// - can reference other items
/// - can fetch items that reference them
///
/// Polly collection can implement this trait for each type.
pub trait CollectionRef<T: ?Sized + 'static> {
    type RE<'a>: RefEntry<'a, T = T>
    where
        Self: 'a;

    type IterRef<'a>: Iterator<Item = Self::RE<'a>> + 'a
    where
        Self: 'a;

    // Implementations should specialize this for Composite items.
    fn add(&mut self, item: T) -> Key<T>
    where
        T: Sized;

    // Implementations should specialize this for Composite items.
    fn add_clone(&mut self, item: &T) -> Key<T>
    where
        T: Clone;

    /// Errors if key is not in use.
    fn get<'a>(&'a self, key: Key<T>) -> Result<Self::RE<'a>, Error>;

    // NOTE: Since Key is numerical hence countable and storage needs to be able ot check if a key is valid
    // hence iteration is always possible although maybe expensive.

    fn iter<'a>(&'a self) -> Self::IterRef<'a>;
}

// Responsibilities of this trait shouldn't be delegated to T.
pub trait RefEntry<'a>: 'a {
    type T: ?Sized;
    type Iter<T: ?Sized>: Iterator<Item = Key<T>> + 'a;

    fn key(&self) -> Key<Self::T>;

    fn item(&self) -> &Self::T;

    /// Can change between get's.
    fn from<T: ?Sized>(&self) -> Self::Iter<T>;
}
