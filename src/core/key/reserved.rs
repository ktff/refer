use super::{Index, SubKey};
use log::*;
use std::{any::TypeId, mem::forget};

/// Helps to make allocate process easier to do correctly.
pub struct ReservedKey<T: 'static>(SubKey<T>);

impl<T: 'static> ReservedKey<T> {
    /// Should only be constructed by Containers.
    pub const fn new(key: SubKey<T>) -> Self {
        Self(key)
    }

    pub fn key(&self) -> SubKey<T> {
        self.0
    }

    pub fn type_id(&self) -> TypeId {
        self.0.type_id()
    }

    /// Adds prefix of given len.
    pub fn push(self, prefix_len: u32, prefix: usize) -> Self {
        Self(self.take().push(prefix_len, prefix))
    }

    /// Splits of prefix of given len and suffix.
    pub fn pop(self, prefix_len: u32) -> (usize, Self) {
        let (prefix, suffix) = self.take().pop(prefix_len);
        (prefix, Self(suffix))
    }

    /// Splits of prefix of given len and suffix.
    /// Fails if there is no suffix.
    pub fn pop_try(self, prefix_len: u32) -> Result<(usize, Self), Index> {
        match self.take().pop_try(prefix_len) {
            Ok((prefix, suffix)) => Ok((prefix, Self(suffix))),
            Err(index) => Err(index),
        }
    }

    /// Only proper way to finish reservation.
    pub fn take(self) -> SubKey<T> {
        let key = self.0;
        forget(self);
        key
    }
}

impl<T: 'static> Drop for ReservedKey<T> {
    fn drop(&mut self) {
        warn!("Leaked reserved key {:?}", self.0);
    }
}
