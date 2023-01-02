use std::{
    alloc::{Allocator, Layout},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// Same as Box but doesn't remember it's allocator.
pub struct ShardBox<T>(NonNull<T>);

impl<T> ShardBox<T> {
    pub fn new(value: T, allocator: &impl Allocator) -> Self {
        let uninit = allocator
            .allocate(Layout::for_value(&value))
            .expect("Failed to allocate")
            .cast::<T>();
        // This is safe since we allocated memory for it and
        // there is no other access to it.
        unsafe { uninit.as_uninit_mut().write(value) };
        Self(uninit)
    }

    pub fn into_inner(self, allocator: &impl Allocator) -> T {
        let Self(ptr) = self;
        unsafe {
            let value = ptr.as_ptr().read();
            let layout = Layout::for_value(&value);
            allocator.deallocate(ptr.cast(), layout);
            value
        }
    }

    /// Allocator must be the same or must be able to free the memory
    /// of any other allocator.
    pub fn drop(self, allocator: &impl Allocator) {
        let Self(ptr) = self;
        unsafe {
            let layout = Layout::for_value(&*ptr.as_ptr());
            std::ptr::drop_in_place(ptr.as_ptr());
            allocator.deallocate(ptr.cast(), layout);
        };
    }
}

// This are safe since AllocBox has ownership of T.
unsafe impl<T: Sync> Sync for ShardBox<T> {}
unsafe impl<T: Send> Send for ShardBox<T> {}

impl<T> Deref for ShardBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for ShardBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
