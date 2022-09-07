use std::{
    any,
    fmt::{self, Debug},
    marker::PhantomData,
    num::NonZeroU64,
    ops::{Add, Sub},
};

use super::{Index, Key};

/// A delta constructed from Key<T> - Index = Delta<T>
pub struct DeltaKey<T: ?Sized + 'static>(u64, PhantomData<T>);

impl<T: ?Sized + 'static> DeltaKey<T> {
    pub fn new(delta: u64) -> Self {
        DeltaKey(delta, PhantomData)
    }

    /// Delta will have a string of same upper bits. Either 000....
    /// or 111...., but can also have a string on the lower bits if it's a key to
    /// a high up item although that can be ignored for optimization.
    ///
    /// The length depends on the proximity of the key and index used to construct it.
    pub fn delta(self) -> u64 {
        self.0
    }
}

impl<T: ?Sized + 'static> Sub<Index> for Key<T> {
    type Output = DeltaKey<T>;

    fn sub(self, other: Index) -> Self::Output {
        DeltaKey((self.0).0.get().wrapping_sub(other.0.get()), PhantomData)
    }
}

impl<T: ?Sized + 'static> Add<DeltaKey<T>> for Index {
    type Output = Key<T>;

    fn add(self, other: DeltaKey<T>) -> Self::Output {
        other + self
    }
}

impl<T: ?Sized + 'static> Add<Index> for DeltaKey<T> {
    type Output = Key<T>;

    fn add(self, other: Index) -> Self::Output {
        Key(
            Index(NonZeroU64::new(self.0.wrapping_add(other.0.get())).expect("Should not be zero")),
            PhantomData,
        )
    }
}

impl<T: ?Sized + 'static> Copy for DeltaKey<T> {}

impl<T: ?Sized + 'static> Clone for DeltaKey<T> {
    fn clone(&self) -> Self {
        DeltaKey(self.0, PhantomData)
    }
}

impl<T: ?Sized + 'static> Debug for DeltaKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeltaKey<{}>({:?})", any::type_name::<T>(), self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta() {
        let key = Key::<u8>::new(Index(NonZeroU64::new(0xffff_0000_0020).unwrap()));
        let index = Index(NonZeroU64::new(0xffff_0000_0000).unwrap());
        let delta = key - index;
        assert_eq!(delta.delta(), 0x20);
        assert_eq!(delta + index, key);
    }
}
