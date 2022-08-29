use std::{
    any::{self, TypeId},
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU64,
    ops::{Add, Sub},
};

const INDEX_BITS: u32 = std::mem::size_of::<Index>() as u32 * 8;

pub const MAX_KEY_LEN: u32 = INDEX_BITS;

// NOTE: Key can't be ref since it's not possible for all but the basic library to statically guarantee that
// the key is valid so some kind of dynamic check is needed, hence the library needs to be able to check any key
// hence it needs to be able to know where something starts and ends which is not robustly possible for ref keys.

// NOTE: That leaves us with numerical keys.

// NOTE: Index could be larger than u64 so the possibility of changing that to u128 is left as an option.

/// Index shouldn't be zero. Instead impl can use this for optimizations and to check for invalid composite keys.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Index(pub NonZeroU64);

impl Index {
    pub const fn len_low(self) -> u32 {
        INDEX_BITS - self.0.get().leading_zeros()
    }

    pub const fn len_high(self) -> u32 {
        INDEX_BITS - self.0.get().trailing_zeros()
    }

    pub fn as_usize(self) -> usize {
        self.0.get() as usize
    }

    /// Pushes suffix under prefix/self from bottom.
    pub fn with_suffix(self, suffix_len: u32, suffix: Index) -> Self {
        debug_assert!(suffix.len_low() <= suffix_len, "Invalid suffix");

        let prefix = NonZeroU64::new(self.0.get() << suffix_len).expect("Invalid prefix");
        let suffix = suffix.0;

        Index(prefix | suffix)
    }

    /// Pushes prefix on suffix/self from top.
    pub fn with_prefix(self, prefix_len: u32, prefix: Index) -> Self {
        debug_assert!(prefix.len_low() <= prefix_len, "Invalid prefix");

        let prefix =
            NonZeroU64::new(prefix.0.get() << (INDEX_BITS - prefix_len)).expect("Invalid prefix");
        let suffix = NonZeroU64::new(self.0.get() >> prefix_len).expect("Invalid suffix");

        Index(prefix | suffix)
    }

    /// Splits of suffix from bottom of self.
    /// This is the inverse of with_suffix.
    pub fn split_suffix(self, suffix_len: u32) -> (Self, Self) {
        let prefix = NonZeroU64::new(self.0.get() >> suffix_len).expect("Invalid prefix");
        let suffix =
            NonZeroU64::new(self.0.get() & ((1 << suffix_len) - 1)).expect("Invalid suffix");

        (Index(prefix), Index(suffix))
    }

    /// Splits of prefix from top of self.
    /// This is the inverse of with_prefix.
    pub fn split_prefix(self, prefix_len: u32) -> (Self, Self) {
        let prefix =
            NonZeroU64::new(self.0.get() >> (INDEX_BITS - prefix_len)).expect("Invalid prefix");
        let suffix = NonZeroU64::new(self.0.get() << prefix_len).expect("Invalid suffix");

        (Index(prefix), Index(suffix))
    }

    /// Tries to split of prefix from top of self.
    /// Can fail if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> (Self, Option<Self>) {
        let prefix =
            NonZeroU64::new(self.0.get() >> (INDEX_BITS - prefix_len)).expect("Invalid prefix");
        let suffix = NonZeroU64::new(self.0.get() << prefix_len);

        (Index(prefix), suffix.map(Index))
    }
}

impl Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

/// This is builded from top by pushing prefixes on top from bottom.
/// And deconstructed from top by removing prefixes.
pub struct SubKey<T: ?Sized>(Index, PhantomData<T>);

impl<T: ?Sized> SubKey<T> {
    /// New Sub key with index of len.
    pub fn new(len: u32, index: Index) -> Self {
        let index = NonZeroU64::new(index.0.get() << (INDEX_BITS - len)).expect("Invalid suffix");
        Self(Index(index), PhantomData)
    }

    pub fn index(&self, len: u32) -> Index {
        Index(NonZeroU64::new((self.0).0.get() >> (INDEX_BITS - len)).expect("Invalid key"))
    }

    pub fn as_usize(&self, len: u32) -> usize {
        ((self.0).0.get() >> (INDEX_BITS - len)) as usize
    }

    /// Caller must ensure that the sub key is fully builded,
    /// otherwise any use has high chance of failing.
    pub fn into_key(self) -> Key<T> {
        Key::new(self.0)
    }

    /// Adds prefix of given len.
    pub fn with_prefix(self, prefix_len: u32, prefix: Index) -> Self {
        Self(self.0.with_prefix(prefix_len, prefix), PhantomData)
    }

    /// Splits of prefix of given len and suffix.
    pub fn split_prefix(self, prefix_len: u32) -> (Index, Self) {
        let (prefix, suffix) = self.0.split_prefix(prefix_len);
        (prefix, Self(suffix, PhantomData))
    }

    /// Splits of prefix of given len and suffix.
    /// Fails if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> (Index, Option<Self>) {
        let (prefix, suffix) = self.0.split_prefix_try(prefix_len);
        (prefix, suffix.map(|suffix| Self(suffix, PhantomData)))
    }
}

impl<T: ?Sized> Copy for SubKey<T> {}

impl<T: ?Sized> Clone for SubKey<T> {
    fn clone(&self) -> Self {
        SubKey(self.0, PhantomData)
    }
}

impl<T: ?Sized + 'static> From<Key<T>> for SubKey<T> {
    fn from(key: Key<T>) -> Self {
        SubKey(key.0, PhantomData)
    }
}

/// This is deconstructed from top by taking prefixes.
#[derive(Clone, Copy)]
pub struct AnySubKey(TypeId, Index);

impl AnySubKey {
    pub fn downcast<T: ?Sized + 'static>(self) -> Option<SubKey<T>> {
        if self.0 == TypeId::of::<T>() {
            Some(SubKey(self.1, PhantomData))
        } else {
            None
        }
    }

    pub fn ty_id(&self) -> TypeId {
        self.0
    }

    pub fn split_prefix(self, prefix_len: u32) -> (Index, Self) {
        let (prefix, suffix) = self.1.split_prefix(prefix_len);
        (prefix, Self(self.0, suffix))
    }

    pub fn split_prefix_try(self, prefix_len: u32) -> (Index, Option<Self>) {
        let (prefix, suffix) = self.1.split_prefix_try(prefix_len);
        (prefix, suffix.map(|suffix| Self(self.0, suffix)))
    }
}

impl<T: ?Sized + 'static> From<SubKey<T>> for AnySubKey {
    fn from(key: SubKey<T>) -> Self {
        AnySubKey(TypeId::of::<T>(), key.0)
    }
}

impl From<AnyKey> for AnySubKey {
    fn from(key: AnyKey) -> Self {
        AnySubKey(key.0, key.1)
    }
}

/// It's the responsibility of collections to issue keys in a way that close by indices have close by items.
pub struct Key<T: ?Sized>(Index, PhantomData<T>);

impl<T: ?Sized> Key<T> {
    pub fn new(index: Index) -> Self {
        Key(index, PhantomData)
    }

    pub fn index(&self) -> Index {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        (self.0).0.get().try_into().expect("Index is too large")
    }

    pub fn upcast(self) -> AnyKey
    where
        T: 'static,
    {
        self.into()
    }

    pub fn len(&self) -> u32 {
        self.0.len_low()
    }
}

impl<T: ?Sized> Eq for Key<T> {}

impl<T: ?Sized> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: ?Sized> PartialOrd for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ?Sized> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: ?Sized> Copy for Key<T> {}

impl<T: ?Sized> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0, PhantomData)
    }
}

impl<T: ?Sized> Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key<{}>({:?})", any::type_name::<T>(), self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnyKey(TypeId, Index);

impl AnyKey {
    pub fn new(ty_id: TypeId, index: Index) -> Self {
        AnyKey(ty_id, index)
    }

    pub fn downcast<T: ?Sized + 'static>(self) -> Option<Key<T>> {
        if self.0 == TypeId::of::<T>() {
            Some(Key::new(self.1))
        } else {
            None
        }
    }

    pub fn index(&self) -> Index {
        self.1
    }

    pub fn len(&self) -> u32 {
        self.1.len_low()
    }

    pub fn ty_id(&self) -> TypeId {
        self.0
    }
}

impl std::fmt::Debug for AnyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyKey<{:?}>({:?})", self.0, self.1)
    }
}

impl<T: ?Sized + 'static> From<Key<T>> for AnyKey {
    fn from(key: Key<T>) -> Self {
        AnyKey(TypeId::of::<T>(), key.0)
    }
}

/// A delta constructed from Key<T> - Index = Delta<T>
pub struct DeltaKey<T: ?Sized>(u64, PhantomData<T>);

impl<T: ?Sized> DeltaKey<T> {
    pub fn new(delta: u64) -> Self {
        DeltaKey(delta, PhantomData)
    }

    /// Delta will have a string of same upper bits. Either 000....
    /// or 111....
    ///
    /// The length depends on the proximity of the key and index used to construct it.
    pub fn delta(self) -> u64 {
        self.0
    }
}

impl<T: ?Sized> Sub<Index> for Key<T> {
    type Output = DeltaKey<T>;

    fn sub(self, other: Index) -> Self::Output {
        DeltaKey((self.0).0.get().wrapping_sub(other.0.get()), PhantomData)
    }
}

impl<T: ?Sized> Add<DeltaKey<T>> for Index {
    type Output = Key<T>;

    fn add(self, other: DeltaKey<T>) -> Self::Output {
        other + self
    }
}

impl<T: ?Sized> Add<Index> for DeltaKey<T> {
    type Output = Key<T>;

    fn add(self, other: Index) -> Self::Output {
        Key(
            Index(NonZeroU64::new(self.0.wrapping_add(other.0.get())).expect("Should not be zero")),
            PhantomData,
        )
    }
}

impl<T: ?Sized> Copy for DeltaKey<T> {}

impl<T: ?Sized> Clone for DeltaKey<T> {
    fn clone(&self) -> Self {
        DeltaKey(self.0, PhantomData)
    }
}

impl<T: ?Sized> Debug for DeltaKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeltaKey<{}>({:?})", any::type_name::<T>(), self.0)
    }
}
