use super::{Index, SubKey};
use log::*;
use std::mem::forget;

/// Helps to make allocate process easier to do correctly.
pub struct ReservedKey<T>(SubKey<T>);

impl<T> ReservedKey<T> {
    /// Should only be constructed by Containers.
    pub const fn new(key: SubKey<T>) -> Self {
        Self(key)
    }

    pub fn key(&self) -> SubKey<T> {
        self.0
    }

    /// Adds prefix of given len.
    pub fn with_prefix(self, prefix_len: u32, prefix: usize) -> Self {
        Self(self.0.with_prefix(prefix_len, prefix))
    }

    /// Splits of prefix of given len and suffix.
    pub fn split_prefix(self, prefix_len: u32) -> (usize, Self) {
        let (prefix, suffix) = self.0.split_prefix(prefix_len);
        (prefix, Self(suffix))
    }

    /// Splits of prefix of given len and suffix.
    /// Fails if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> Result<(usize, Self), Index> {
        match self.0.split_prefix_try(prefix_len) {
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

impl<T> Drop for ReservedKey<T> {
    fn drop(&mut self) {
        warn!("Leaked reserved key {:?}", self.0);
    }
}
