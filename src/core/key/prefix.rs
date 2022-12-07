use super::{Index, Key};
use std::{any::TypeId, num::NonZeroU64};

// TODO: Interweave this with Index and Sub keys.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct KeyPrefix {
    prefix_len: u32,
    prefix: usize,
}

impl KeyPrefix {
    pub fn new(prefix_len: u32, prefix: usize) -> Self {
        KeyPrefix { prefix_len, prefix }
    }

    /// Leaves only common prefix.
    pub fn intersect(self, other: Self) -> Self {
        unimplemented!()
    }

    /// Iterates prefixes under this one.
    /// Prefixes are of max sub_level lower.
    pub fn iter_sub(self, sub_level: u32) -> impl Iterator<Item = Self> {
        std::iter::empty()
        // unimplemented!()
    }

    pub fn key<T: ?Sized + 'static>(self, index: Index) -> Key<T> {
        Key::new(index.with_prefix(self.prefix_len, self.prefix))
    }

    // pub fn prefix_len(&self) -> u32 {
    //     self.prefix_len
    // }

    // pub fn prefix(&self) -> usize {
    //     self.prefix
    // }

    pub fn take(prefix_len: u32, i: NonZeroU64) -> (KeyPrefix, Option<NonZeroU64>) {
        let rotated = i.get().rotate_left(prefix_len);
        let prefix = rotated & (!((!0u64) << prefix_len));
        let suffix = NonZeroU64::new(i.get() << prefix_len);

        // Can't handle prefix_len == 0
        // let prefix = self.0.get() >> (INDEX_BITS - prefix_len);
        // let suffix = NonZeroU64::new(self.0.get() << prefix_len);

        (
            KeyPrefix {
                prefix_len,
                prefix: (prefix as usize),
            },
            suffix,
        )
    }

    pub fn prefix_of(self, i: NonZeroU64) -> bool {
        let (prefix, suffix) = KeyPrefix::take(self.prefix_len, i);
        prefix.prefix == self.prefix && suffix.is_some()
    }
}

impl std::fmt::Display for KeyPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0width$b}",
            self.prefix,
            width = self.prefix_len as usize
        )
    }
}

impl Default for KeyPrefix {
    fn default() -> Self {
        KeyPrefix {
            prefix_len: 0,
            prefix: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnyKeyPrefix(TypeId, KeyPrefix);

impl AnyKeyPrefix {
    pub fn new(ty: TypeId, prefix: KeyPrefix) -> Self {
        Self(ty, prefix)
    }

    pub fn ty(&self) -> TypeId {
        self.0
    }

    pub fn prefix(&self) -> KeyPrefix {
        self.1
    }
}