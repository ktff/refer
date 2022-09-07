use std::{
    any::{self, TypeId},
    fmt::{self, Debug},
    marker::PhantomData,
    num::NonZeroU64,
};

use super::{AnyKey, Index, Key, MAX_KEY_LEN};

/// This is builded from top by pushing prefixes on top from bottom.
/// And deconstructed from top by removing prefixes.
pub struct SubKey<T: ?Sized + 'static>(Index, PhantomData<T>);

impl<T: ?Sized + 'static> SubKey<T> {
    /// New Sub key with index of len.
    pub const fn new(len: u32, index: Index) -> Self {
        let index = NonZeroU64::new(index.0.get() << (MAX_KEY_LEN - len)).expect("Invalid suffix");
        Self(Index(index), PhantomData)
    }

    pub fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    pub fn index(&self, len: u32) -> Index {
        Index(NonZeroU64::new(self.as_u64(len)).expect("Invalid key"))
    }

    pub fn as_usize(&self, len: u32) -> usize {
        self.as_u64(len) as usize
    }

    fn as_u64(&self, len: u32) -> u64 {
        (self.0).0.get() >> (MAX_KEY_LEN - len)
    }
    /// Caller must ensure that the sub key is fully builded,
    /// otherwise any use has high chance of failing.
    ///
    /// Can produce key with full MAX_KEY_LEN depth, but can
    /// also produce key with less depth for items on higher levels.
    pub fn into_key(self) -> Key<T> {
        Key::new(self.0)
    }

    /// Adds prefix of given len.
    pub fn with_prefix(self, prefix_len: u32, prefix: usize) -> Self {
        Self(self.0.with_prefix(prefix_len, prefix), PhantomData)
    }

    /// Splits of prefix of given len and suffix.
    pub fn split_prefix(self, prefix_len: u32) -> (usize, Self) {
        let (prefix, suffix) = self.0.split_prefix(prefix_len);
        (prefix, Self(suffix, PhantomData))
    }

    /// Splits of prefix of given len and suffix.
    /// Fails if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> Result<(usize, Self), Index> {
        match self.0.split_prefix_try(prefix_len) {
            Ok((prefix, suffix)) => Ok((prefix, Self(suffix, PhantomData))),
            Err(suffix) => Err(suffix),
        }
    }
}

impl<T: ?Sized + 'static> PartialEq for SubKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized + 'static> Copy for SubKey<T> {}

impl<T: ?Sized + 'static> Clone for SubKey<T> {
    fn clone(&self) -> Self {
        SubKey(self.0, PhantomData)
    }
}

impl<T: ?Sized + 'static> From<Key<T>> for SubKey<T> {
    fn from(key: Key<T>) -> Self {
        SubKey(key.0, PhantomData)
    }
}

impl<T: ?Sized + 'static> Debug for SubKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubKey<{}>({:?})", any::type_name::<T>(), self.0)
    }
}

/// This is deconstructed from top by taking prefixes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AnySubKey(TypeId, Index);

impl AnySubKey {
    pub fn downcast<T: ?Sized + 'static>(self) -> Option<SubKey<T>> {
        if self.0 == TypeId::of::<T>() {
            Some(SubKey(self.1, PhantomData))
        } else {
            None
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.0
    }

    /// Caller must ensure that the sub key is fully builded,
    /// otherwise any use has high chance of failing.
    ///
    /// Can produce key with full MAX_KEY_LEN depth, but can
    /// also produce key with less depth for items on higher levels.
    pub fn into_key(self) -> AnyKey {
        AnyKey::new(self.0, self.1)
    }

    pub fn with_prefix(self, prefix_len: u32, prefix: usize) -> Self {
        Self(self.0, self.1.with_prefix(prefix_len, prefix))
    }

    pub fn split_prefix(self, prefix_len: u32) -> (usize, Self) {
        let (prefix, suffix) = self.1.split_prefix(prefix_len);
        (prefix, Self(self.0, suffix))
    }

    /// Splits of prefix of given len and suffix.
    /// Fails if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> Result<(usize, Self), Index> {
        match self.1.split_prefix_try(prefix_len) {
            Ok((prefix, suffix)) => Ok((prefix, Self(self.0, suffix))),
            Err(suffix) => Err(suffix),
        }
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

impl Debug for AnySubKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnySubKey<{:?}>({:?})", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_key() {
        let index = Index(NonZeroU64::new(0x8f).unwrap());
        let sub_key = SubKey::<u8>::new(8, index);
        assert_eq!(sub_key.index(8), index);
    }

    #[test]
    fn test_any_sub_key() {
        let index = Index(NonZeroU64::new(0x8f).unwrap());
        let sub_key = SubKey::new(8, index);
        let any_sub_key: AnySubKey = sub_key.into();
        assert_eq!(any_sub_key.downcast::<u8>(), Some(sub_key));
        assert_eq!(any_sub_key.downcast::<u16>(), None);
    }

    #[test]
    fn test_any_sub_key_with_prefix() {
        let index = Index(NonZeroU64::new(0x8f).unwrap());
        let sub_key = SubKey::new(8, index);
        let any_sub_key: AnySubKey = sub_key.into();
        let any_sub_key = any_sub_key.with_prefix(4, 0x1);
        assert_eq!(
            any_sub_key.downcast::<u8>(),
            Some(sub_key.with_prefix(4, 0x1))
        );
        assert_eq!(any_sub_key.downcast::<u16>(), None);
    }
}
