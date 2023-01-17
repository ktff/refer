#![allow(unused)]

// Such Vec would:
// - require local alloc ref when doing resizing operations.
// - have minimal memory footprint.
//
// Such vec has three modes:
// - inline, has data and limited size/capacity
// - small_heap, has ptr to data and small size/capacity
// - big_heap, has ptr to data, size, and capacity
//
// When on heap, size and capacity can be stored on heap. Therefor inline needs
// to be able to store ptr. And inline needs to be aligned to T. Heap can also
// store index of allocator so to be able to drop always.
//
// So what is needed are two pieces:
// - an enum inline or ptr.
// - a vec with size/capacity/allocator_index on heap and two modes.
//
// Such vec can encode size,capacity, and allocator index, to reduce memory consumption.
//
// Ptr can be restricted to 6B size. That allows for 48bit address space which is currently used by
// all architectures besides maybe some niche ones. This allows for 256TB memory, so if
// you have a system that uses more than 48bit, then you highly likely have memory to spare
// so don't use this. If you do, it will just panic.
//
// Then such enum can also assume Copy for it's elements since anything less than usize,8B size,
// can't contain reference, or at least not safely, so it's safe to restrict to Copy. That would
// allow for such enum to forgo alignment and allow only move access. Without an alignment, it's
// possible to reduce memory consumption by removing padding.
//
// To reduce enum header size we can enforce alignment on vec of at least 2. Then we can use lowest
// bit as a flag to indicate if it's inline or not. If not, the rest is a ptr to vec, and if yes,
// the next 7bits in lowest byte are len, and the rest is data.
//
// With that, enum can have min 6B size.
//
// SmallVec can in initial phase use 1B for len and allocate element by element until either 1B is not enough
// or realloc fails. After that switch to Vec like behavior. Also make it have min alignment of 4B.
//
// No, SmallVec wont have index/ptr to allocator. It will be responsibility of user to pass it in always and
// to drop it. If it's not done then it will just leak memory. But probably not even then since its supposed
// to be used with allocator that has lifetime of container. So SmallVec should only accept Copy types.
//
// There could be special case for global allocator but lets leave that for later.
//
// Go with 6B or 8B? 8B for now. It's easier and should be good enough, 6B can be specialization.
//
// Also, max 256B/elements is good enough for now.

use super::{read_u64, write_u64};
use modular_bitfield::prelude::*;
use std::{
    alloc::Layout,
    fmt,
    mem::MaybeUninit,
    ops::{Bound, RangeBounds},
    ptr::{self, NonNull},
};

// 8B version
// Constraints:
// - force min alignment of 2B for allocated data
//
//

// TODO: All reallocation in here are unsafe since they can't be guaranteed to be
//       called on the same allocator as the current allocation.

/// Inline allocator for small Copy elements.
///
/// Can hold N elements inline, but no less than 6B.
/// Alignment is that of T.
///
/// !!! WARNING !!! will leek any allocated memory on drop. Call `clear_dealloc` before dropping it.
pub struct ShardVec<T: Copy, const N: usize> {
    payload: Payload<T, N>,
    header: InlineHeader,
}

impl<T: Copy, const N: usize> ShardVec<T, N> {
    pub fn new() -> Self {
        debug_assert!(N < 0xf);
        Self {
            payload: Payload {
                inline: [MaybeUninit::uninit(); N],
            },
            header: InlineHeader::new().with_len(0).with_capacity(N as u8),
        }
    }

    // pub fn clone_in(&self, alloc: &(impl std::alloc::Allocator + ?Sized)) -> Self {
    //     let mut new = Self::new();
    //     for e in self.iter() {
    //         new.push(*e, alloc);
    //     }
    //     new
    // }

    pub fn push(&mut self, element: T, allocator: &(impl std::alloc::Allocator + ?Sized)) {
        let len = self.len();
        let slot = if let Some(slot) = self.get_mut().get_mut(len) {
            slot
        } else {
            self.grow(len + 1, allocator);
            self.get_mut()
                .get_mut(len)
                .expect("Grow should have increased capacity")
        };

        slot.write(element);
        self.set_len(len + 1);
    }

    /// Panics if index larger than len.
    pub fn insert(
        &mut self,
        index: usize,
        element: T,
        allocator: &(impl std::alloc::Allocator + ?Sized),
    ) {
        let len = self.len();
        assert!(index <= len);
        if index == len {
            self.push(element, allocator);
            return;
        } else if len == self.capacity() {
            self.grow(len + 1, allocator);
            assert!(
                self.len() < self.capacity(),
                "Grow should have increased capacity"
            );
        }

        // Copy all elements after index to the right.
        let slice = self.get_mut();
        let from = slice[index].as_ptr();
        let to = slice[index + 1].as_mut_ptr();
        // SAFETY: The range is valid.
        unsafe {
            ptr::copy(from, to, len - index);
        }

        // Write element to index.
        slice[index].write(element);
        self.set_len(len + 1);
    }

    /// Removes element at index and returns it.
    /// Last element is moved to fill the gap.
    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        assert!(index < len, "Index out of bounds");
        let len = len.checked_sub(1).expect("Can't remove from empty vec") as usize;
        let slice = self.get_mut();

        if len == index {
            let element= // This is safe since it was initialized according to len.
            unsafe { slice[len].assume_init_read() };
            // Last element, just pop it.
            self.set_len(len);
            element
        } else {
            // Swap last element with removed element.
            // This is safe since it was initialized according to len.
            let last_element = unsafe { slice[len].assume_init_read() };
            // This is safe since it was initialized according to len.
            let removed_element = unsafe { slice[index].assume_init_read() };
            slice[index].write(last_element);
            self.set_len(len);
            removed_element
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let len = self.len().checked_sub(1)?;
        self.set_len(len);

        let slot = self.get_mut().get_mut(len).expect("len is always valid");
        // This is safe since it was initialized according to len.
        Some(unsafe { slot.assume_init_read() })
    }

    pub fn as_slice(&self) -> &[T] {
        // This is safe if self.len of first elements are indeed initialized.
        unsafe { MaybeUninit::slice_assume_init_ref(&self.get()[..self.len()]) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let len = self.len();
        // This is safe if self.len of first elements are indeed initialized.
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.get_mut()[..len]) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_slice_mut().iter_mut()
    }

    pub fn capacity(&self) -> usize {
        if let Some(heap) = self.heap_payload() {
            heap.capacity as usize
        } else {
            self.header.capacity() as usize
        }
    }

    pub fn len(&self) -> usize {
        if let Some(heap) = self.heap_payload() {
            heap.len as usize
        } else {
            self.header.len() as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self, allocator: &(impl std::alloc::Allocator + ?Sized)) {
        // T is copy so they don't have Drop.
        self.set_len(0);
        // Dealloc
        if let Some(heap) = self.heap_payload_mut() {
            // Safe since we know that this is a valid ptr since it came from ref.
            let ptr = unsafe { NonNull::new_unchecked(heap as *mut HeapPayload<T> as *mut u8) };
            let layout = Layout::for_value::<HeapPayload<T>>(heap);
            self.header.set_len(0);
            self.header.set_capacity(N as u8);
            // This is safe since ShardAllocator can handle
            // allocations from different ShardAllocators.
            unsafe { allocator.deallocate(ptr, layout) }
        }
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        // Modified version from Vec::retain_mut

        let original_len = self.len();
        // There is no drop. So this code can be simplified.

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |<-              processed len   ->| ^- next to check
        //                  |<-  deleted cnt     ->|
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        struct Track<'a, T> {
            v: &'a mut [T],
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        let mut g = Track {
            v: self.as_slice_mut(),
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        fn process_loop<F, T, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut Track<'_, T>,
        ) where
            F: FnMut(&mut T) -> bool,
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe { ptr::drop_in_place(cur) };
                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted.
        process_loop::<F, T, false>(original_len, &mut f, &mut g);

        // Stage 2: Some elements were deleted.
        process_loop::<F, T, true>(original_len, &mut f, &mut g);

        // All item are processed. This can be optimized to `set_len` by LLVM.
        if g.deleted_cnt > 0 {
            // SAFETY: Trailing unchecked items must be valid since we never touch them.
            unsafe {
                ptr::copy(
                    g.v.as_ptr().add(g.processed_len),
                    g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt),
                    g.original_len - g.processed_len,
                );
            }
        }
        // SAFETY: After filling holes, all items are in contiguous memory.
        let new_len = original_len - g.deleted_cnt;
        self.set_len(new_len);
    }

    /// Moves all the elements of other into self, leaving other cleared.
    pub fn append(&mut self, other: &mut Self, allocator: &(impl std::alloc::Allocator + ?Sized)) {
        for &item in other.iter() {
            self.push(item, allocator);
        }
        other.clear(allocator);
    }

    pub fn remove(&mut self, index: usize) -> T {
        #[cold]
        #[inline(never)]
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (is {index}) should be < len (is {len})");
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }
        unsafe {
            // infallible
            let ret;
            {
                // the place we are taking from.
                let ptr = self.as_slice_mut().as_mut_ptr().add(index);
                // copy it out, unsafely having a copy of the value on
                // the stack and in the vector at the same time.
                ret = ptr::read(ptr);

                // Shift everything down to fill in that spot.
                ptr::copy(ptr.add(1), ptr, len - index - 1);
            }
            self.set_len(len - 1);
            ret
        }
    }

    // Panics if out of range
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) {
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };
        assert!(start <= end);
        assert!(end <= len);
        if end == len {
            self.set_len(start);
        } else {
            let slice = self.get_mut();
            let from = slice[end].as_ptr();
            let to = slice[start].as_mut_ptr();
            // SAFETY: The range is valid.
            unsafe {
                ptr::copy(from, to, len - end);
            }
            self.set_len(len - (end - start));
        }
    }

    fn grow(&mut self, min_capacity: usize, allocator: &(impl std::alloc::Allocator + ?Sized)) {
        let capacity = self.capacity();
        let len = self.len();
        let new_capacity = (capacity.max(1) * 2).max(min_capacity);

        // This is safe since we are constructing a UnSized with a slice.
        let new_layout = unsafe {
            let ptr: *const HeapPayload<T> =
                std::ptr::from_raw_parts(std::ptr::null(), new_capacity);
            Layout::for_value_raw(ptr)
        };
        let ptr: NonNull<[u8]> = match self.state() {
            State::Inline => {
                let ptr = allocator.allocate(new_layout).expect("Failed to grow");

                // Init heap
                let extra_capacity = (ptr.len() - new_layout.size()) / std::mem::size_of::<T>();
                let final_capacity = new_capacity + extra_capacity;
                u16::try_from(final_capacity).expect("Too large capacity");

                // Safe since heap is freshly allocated
                // NOTE: This should be wrapped in MaybeUninit but it doesn't support unsized types.
                //        But having uninit 2 usize should be fine.
                let mut heap: &mut HeapPayload<T> = unsafe {
                    &mut *std::ptr::from_raw_parts_mut(ptr.as_ptr() as *mut (), final_capacity)
                };

                heap.len = 0;
                heap.capacity = final_capacity as u16;

                // Copy
                // This is safe since memories don't overlap and since dst has greater capacity.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        self.payload.inline.as_ptr(),
                        heap.slice.as_mut_ptr(),
                        len,
                    )
                }
                heap.len = len as u16;

                ptr
            }
            State::Heap => {
                // This is safe since we ShardAllocator can handle
                // allocations from different ShardAllocators.
                unsafe {
                    let payload = self.heap_payload_mut().expect("Heap payload should exist");
                    let layout = Layout::for_value::<HeapPayload<T>>(payload);
                    let ptr = allocator
                        .grow(
                            // Safe since we know that this is a valid ptr since it came from ref.
                            NonNull::new_unchecked(payload as *mut HeapPayload<T> as *mut u8),
                            layout,
                            new_layout,
                        )
                        .expect("Failed to grow");
                    // update capacity
                    let extra_capacity = (ptr.len() - new_layout.size()) / std::mem::size_of::<T>();
                    let final_capacity = new_capacity + extra_capacity;
                    u16::try_from(final_capacity).expect("Too large capacity");

                    let heap: *mut HeapPayload<T> =
                        std::ptr::from_raw_parts_mut(ptr.as_ptr() as *mut (), final_capacity);
                    // Safe since heap is freshly allocated
                    (&mut *heap).capacity = final_capacity as u16;

                    ptr
                }
            }
        };

        // Update to heap
        let ptr = ptr.as_ptr() as *const u8 as usize as u64;
        // Limited by amount of on inline storage
        assert!(ptr < 0x1_00_00_00_00_00_00);
        self.payload = Payload {
            // This is safe since ptr is larger than 6B
            heap: write_u64(ptr),
        };
        self.header.set_capacity(0xfu8);
    }

    fn get_mut(&mut self) -> &mut [MaybeUninit<T>] {
        match self.state() {
            State::Heap => {
                &mut self
                    .heap_payload_mut()
                    .expect("Heap payload should exist")
                    .slice
            }
            State::Inline => {
                // This is safe if contract of this struct that T is indeed inline is upheld.
                unsafe { &mut self.payload.inline }
            }
        }
    }

    fn get(&self) -> &[MaybeUninit<T>] {
        if let Some(heap) = self.heap_payload() {
            &heap.slice
        } else {
            // This is safe if contract of this struct that T is indeed inline is upheld.
            unsafe { &self.payload.inline }
        }
    }

    fn set_len(&mut self, len: usize) {
        if let Some(heap) = self.heap_payload_mut() {
            heap.len = len as u16;
        } else {
            self.header.set_len(len as u8)
        }
    }

    fn heap_payload(&self) -> Option<&HeapPayload<T>> {
        match self.state() {
            State::Heap => {
                unsafe {
                    // This is safe since we've checked that the state is HeapPayload.
                    let ptr = read_u64(self.payload.heap) as *const ();

                    // Len 0 is certainly valid to read capacity.
                    let zero: *const HeapPayload<T> = std::ptr::from_raw_parts(ptr, 0);
                    let capacity = (&*zero).capacity;

                    // Payload is valid for capacity.
                    let payload: *const HeapPayload<T> =
                        std::ptr::from_raw_parts(ptr, capacity as usize);

                    Some(&*payload)
                }
            }
            _ => None,
        }
    }

    fn heap_payload_mut(&mut self) -> Option<&mut HeapPayload<T>> {
        match self.state() {
            State::Heap => {
                unsafe {
                    // This is safe since we've checked that the state is HeapPayload.
                    let ptr = read_u64(self.payload.heap) as *mut ();

                    // Len 0 is certainly valid to read capacity.
                    let zero: *const HeapPayload<T> = std::ptr::from_raw_parts(ptr, 0);
                    let capacity = (&*zero).capacity;

                    // Payload is valid for capacity.
                    let payload: *mut HeapPayload<T> =
                        std::ptr::from_raw_parts_mut(ptr, capacity as usize);

                    Some(&mut *payload)
                }
            }
            _ => None,
        }
    }

    fn state(&self) -> State {
        if self.header.capacity() as usize <= N {
            State::Inline
        } else {
            State::Heap
        }
    }
}

impl<T: Eq + Copy, const N: usize> Eq for ShardVec<T, N> {}

impl<T: PartialEq + Copy, const N: usize> PartialEq for ShardVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

// Indexing
impl<T: Copy, const N: usize> std::ops::Index<usize> for ShardVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T: Copy, const N: usize> std::ops::IndexMut<usize> for ShardVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}

/// Default
impl<T: Copy, const N: usize> Default for ShardVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Copy + fmt::Debug, const N: usize> fmt::Debug for ShardVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for e in self.iter() {
            write!(f, "{:?}, ", e)?;
        }
        write!(f, "]")
    }
}

enum State {
    /// Everything is inline
    Inline,
    /// Item slice and header are on heap
    Heap,
}

#[bitfield]
struct InlineHeader {
    len: B4,
    capacity: B4,
}

union Payload<T: Copy, const N: usize> {
    inline: [MaybeUninit<T>; N],
    heap: [u8; 6],
}

/// Heap form once inline isn't enough.
struct HeapPayload<T: Copy> {
    len: u16,
    capacity: u16,
    slice: [MaybeUninit<T>],
}

// TODO
// impl<T: Copy, const N: usize> Drop for InlineVec<T, N> {
//     fn drop(&mut self) {
//         if !self.inline() {
//             warn!("Memory leaked in InlineVec");
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::util::shard_alloc::ShardAllocator;
    use core::panic;

    use super::*;

    #[test]
    fn push() {
        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();
        vec.push(1, &alloc);
        vec.push(2, &alloc);
        vec.push(3, &alloc);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
    }

    #[test]
    fn push_pop() {
        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();
        vec.push(1, &alloc);
        vec.push(2, &alloc);
        vec.push(3, &alloc);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn push_pop_push() {
        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();
        vec.push(1, &alloc);
        vec.push(2, &alloc);
        vec.push(3, &alloc);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.pop(), Some(2));
        vec.push(4, &alloc);
        assert_eq!(vec.pop(), Some(4));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn remove_range() {
        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();

        vec.push(1, &alloc);
        vec.push(2, &alloc);
        vec.push(3, &alloc);
        vec.push(4, &alloc);
        vec.push(5, &alloc);

        vec.remove_range(1..3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 4);
        assert_eq!(vec[2], 5);
    }

    #[test]
    fn retain_mut() {
        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();

        vec.push(1, &alloc);
        vec.push(2, &alloc);
        vec.push(3, &alloc);
        vec.push(4, &alloc);
        vec.push(5, &alloc);

        vec.retain_mut(|x| *x % 2 == 0);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 2);
        assert_eq!(vec[1], 4);
    }

    #[test]
    fn insert() {
        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();

        vec.push(1, &alloc);
        vec.push(2, &alloc);
        vec.push(3, &alloc);
        vec.push(4, &alloc);

        vec.insert(2, 5, &alloc);

        assert_eq!(vec.len(), 5);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 5);
        assert_eq!(vec[3], 3);
        assert_eq!(vec[4], 4);
    }

    #[test]
    fn doppelganger() {
        use rand::*;
        let ops = 100000;

        let alloc = ShardAllocator::new();
        let mut vec = ShardVec::<u32, 2>::new();
        let mut doppelganger = Vec::new();
        let mut rand = thread_rng();
        for _ in 0..ops {
            match rand.gen_range(0..10) {
                // Push
                0 | 1 | 2 | 3 | 4 | 5 => {
                    let val = rand.gen();
                    vec.push(val, &alloc);
                    doppelganger.push(val);
                }
                // Pop
                6 | 7 | 8 => {
                    assert_eq!(vec.pop(), doppelganger.pop());
                }
                // Remove
                9 if vec.len() > 0 => {
                    let index = rand.gen_range(0..vec.len());
                    assert_eq!(vec.swap_remove(index), doppelganger.swap_remove(index));
                }
                _ => (),
            }
            assert_eq!(vec.as_slice(), doppelganger.as_slice());
        }
    }
}
