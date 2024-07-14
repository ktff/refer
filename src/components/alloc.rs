use std::{
    alloc::{self, AllocError, Allocator, Layout},
    borrow::Borrow,
    ops::Deref,
    ptr::NonNull,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
};

use log::debug;
use parking_lot::Mutex;

/* Notes:
- Lock free is good enough.
- Happy paths.
- 4KB block alloc
- For layouts of equal or larger size delegate to global allocator.
- Use mutex since contention will be low so this will be fine.

   - Lets say Block is 4096 and aligned to 4096, min allocation is 16B and min alignment is then 16B.
   //// Then we need 256bits to track free/taken 16B slots.
   //// Lets use sorted tree of free blocks. With that, it's possible to coallece, have all blocks be in the same
   //// tree, and fast allocate exact size. Tree should be balanced according to left/right available memory.
   //// Such design will be fast for allocate but don't deallocate/ or at least rarely. While for case of lot
   //// of deallocation, it will prioritize coallecsing and thus be slower. This would also allow for realloc.

   //Let internal pointer be comprised of 2 parts, kinda like a fat pointer:
   //- 7B address to [Slot]
   //- 1B len of [Slot]

    It is fine to have max size of total allocated to something like 4GB.

   //Thus first appropirate block is used.

   //Allocate from the back/ or front?
   //// Front and shift internal pointers with memcpy. This will have better cache locality.
   //Back that will have better locality if otherwise will require to access before item in list to change pointer.

   //There are multiple lists.

   Let's keep it simple.

   Use block sizes of 16, 32, 64, 128, 256, 512, 1024, 2048, or even better make them configurable with a &'static [usize]. But that can come later.
   Keep a list with internal pointers for each size.
   Alloc and dealloc from the lists.
   Coallecsing can be done as a separate step during which completely coalesced blocks can be freed. But those can come later.



*/

const BLOCK_SIZE: usize = 4096;
const START_SIZE: usize = 512;
// This is safe since 4096 is not zero, is of power two, and will not overflow isize.
const PAGE_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(BLOCK_SIZE, BLOCK_SIZE) };

/// Concurrent allocator that assumes usually low contention and usually small size of few MB.
/// Lock free.
///
/// Uses global allocator for underlying allocations.
///
/// Can handle allocations from different MiniAllocator instances. Effectively moving memory between them.
#[repr(align(64))]
pub struct MiniAllocator {
    /// Lists to free blocks of exponentially increasing size.
    /// [0] -> [[u64;2];1]
    /// [1] -> [[u64;2];2]
    /// [2] -> [[u64;2];4]
    /// [3] -> [[u64;2];8]
    /// ...
    lists: [AtomicPtr<[u64; 2]>; 8],
    sub_blocks: Mutex<Vec<usize>>,

    allocated: std::sync::atomic::AtomicUsize,
}

impl MiniAllocator {
    pub const fn new() -> Self {
        Self {
            lists: [
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
                AtomicPtr::new(std::ptr::null_mut()),
            ],
            sub_blocks: Mutex::new(Vec::new()),
            allocated: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Free allocated memory
    /// Requires exclusive access.
    pub fn capacity(&mut self) -> usize {
        let mut capacity = 0;
        for (i, list) in self.lists.iter().enumerate() {
            let mut ptr = list.load(Ordering::Relaxed) as *const [u64; 2];
            let size = 16 << i;
            while !ptr.is_null() {
                capacity += size;
                // This is safe since list isn't changing since we have
                // exclusive access.
                ptr = unsafe { (&*ptr)[0] as usize as *const _ };
            }
        }
        capacity
    }

    pub fn allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Returns index of block to use for layout.
    #[inline(always)]
    const fn index(size: usize, align: usize) -> usize {
        let size = if size <= align { align } else { size };
        // Max pow2 len
        let i =
            std::mem::size_of::<usize>() * 8 - size.next_power_of_two().leading_zeros() as usize;
        // Divide by slot size, which is 16, or 4 in pow2. -1 to start from 0.
        i.saturating_sub(4 + 1)
    }

    /// We must have exclusive access to set and it must be properly allocated.
    #[inline(always)]
    unsafe fn add(&self, i: usize, mut block: NonNull<[u64; 2]>) {
        // We use relaxed ordering since we aren't reading anything besides the atomic field.
        // Add now to list at j
        let mut head = self.lists[i].load(Ordering::Relaxed);
        loop {
            // This is safe since we have exclusive access.
            block.as_mut()[0] = head as usize as u64;
            if block.as_ptr() as usize & ((16 << i) - 1) != 0 {
                panic!("Invalid alignment");
            }
            debug_assert_eq!(
                block.as_ptr() as usize & ((16 << i) - 1),
                0,
                "Not aligned {:x} to {}",
                block.as_ptr() as usize,
                16 << i
            );
            match self.lists[i].compare_exchange(
                head,
                // This is safe since we have exclusive access.
                block.as_mut(),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_head) => head = new_head,
            }
        }
    }

    /// Selects block for layout.
    /// Returns (block and it's level)
    #[inline(always)]
    fn select(&self, layout_level: usize) -> Option<(NonNull<[u64; 2]>, usize)> {
        // Find closest sized free block
        for (at, list) in self.lists.iter().enumerate().skip(layout_level) {
            let mut head_ptr = list.load(Ordering::Acquire);

            while let Some(head) = NonNull::new(head_ptr) {
                // This is safe since the memory was allocated.
                let next_ptr = unsafe { head.as_ref()[0] as usize as *mut [u64; 2] };
                match list.compare_exchange(
                    head_ptr,
                    next_ptr,
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    // Block taken
                    // Reading from head is valid since we have exclusive access.
                    Ok(_) => return Some((head, at)),
                    // Someone else allocated the block
                    Err(new) => head_ptr = new,
                }
            }

            // No free blocks of this size, try next size
        }

        None
    }

    /// Allocates layout in block and recycles excess memory.
    #[inline(always)]
    fn allocate_in(
        &self,
        layout_level: usize,
        block: NonNull<[u64; 2]>,
        block_level: usize,
    ) -> NonNull<[u8]> {
        let allocated = NonNull::slice_from_raw_parts(block.cast::<u8>(), 16 << layout_level);

        // Slice off not needed memory
        let mut sliced = NonNull::new(unsafe { block.as_ptr().add(1 << layout_level) })
            .expect("Shouldn't be zero");
        for j in layout_level..block_level {
            // Block of level j can be sliced of
            // This is safe since we have block of this or larger size.
            // Add sliced to list at j
            unsafe { self.add(j, sliced) };
            sliced =
                NonNull::new(unsafe { sliced.as_ptr().add(1 << j) }).expect("Shouldn't be zero");
        }

        //  We are done
        allocated
    }
}

unsafe impl Allocator for MiniAllocator {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let i = Self::index(layout.size(), layout.align());
        if i >= self.lists.len() {
            // This is safe since layout is not zero sized, and alloc guarantees that the memory
            // is allocated.
            self.allocated.fetch_add(layout.size(), Ordering::Relaxed);

            return NonNull::new(unsafe {
                std::ptr::slice_from_raw_parts_mut(alloc::alloc(layout), layout.size())
            })
            .ok_or(AllocError);
        }

        let (block, block_level) = match self.select(i) {
            Some((block, block_level)) => (block, block_level),
            None => {
                // Allocate new block
                let allocated = self.allocated.load(Ordering::Relaxed);
                let (level, layout, sub) = if allocated < BLOCK_SIZE {
                    // Allocate smaller block for now
                    let size = allocated.min(START_SIZE);
                    let j = Self::index(size, 1).max(i);
                    let size = BLOCK_SIZE >> (self.lists.len() - j);
                    (
                        j,
                        Layout::from_size_align(size, size).expect("Shouldn't fail"),
                        true,
                    )
                } else {
                    (self.lists.len(), PAGE_LAYOUT, false)
                };

                // This is safe since layout is not zero sized, and alloc guarantees that the memory
                // is allocated.
                let block = NonNull::new(unsafe { alloc::alloc(layout) })
                    .ok_or(AllocError)?
                    .cast::<[u64; 2]>();

                self.allocated.fetch_add(layout.size(), Ordering::Relaxed);

                if sub {
                    // We need to keep track of this block since it's smaller than the page.
                    self.sub_blocks.lock().push(block.as_ptr() as usize + level);
                }

                (block, level)
            }
        };

        Ok(self.allocate_in(i, block, block_level))
    }

    #[inline(always)]
    unsafe fn deallocate(&self, mut ptr: NonNull<u8>, layout: Layout) {
        let i = Self::index(layout.size(), layout.align());
        if i < self.lists.len() {
            let block = ptr.cast::<[u64; 2]>();
            // This is safe since caller guarantees that.
            self.add(i, block);
        } else {
            // This is safe since layout is not zero sized, while other constraints
            // are delegated through unsafe to the caller.
            self.allocated.fetch_sub(layout.size(), Ordering::Relaxed);
            alloc::dealloc(ptr.as_mut(), layout);
        }
    }

    // TODO:
    // fn allocate_zeroed(
    //     &self,
    //     layout: Layout
    // ) -> Result<NonNull<[u8]>, AllocError> { ... }
    // unsafe fn grow(
    //     &self,
    //     ptr: NonNull<u8>,
    //     old_layout: Layout,
    //     new_layout: Layout
    // ) -> Result<NonNull<[u8]>, AllocError> { ... }
    // unsafe fn grow_zeroed(
    //     &self,
    //     ptr: NonNull<u8>,
    //     old_layout: Layout,
    //     new_layout: Layout
    // ) -> Result<NonNull<[u8]>, AllocError> { ... }
    // unsafe fn shrink(
    //     &self,
    //     ptr: NonNull<u8>,
    //     old_layout: Layout,
    //     new_layout: Layout
    // ) -> Result<NonNull<[u8]>, AllocError> { ... }
}

// Impl Drop for MiniAllocator by reconstructing pages and then deallocating them. Logging any leaked unconstructed pages.
impl Drop for MiniAllocator {
    fn drop(&mut self) {
        // Deallocate all pages
        let mut level = 0;
        let mut blocks = Vec::new();
        let mut next_level_blocks = Vec::new();
        while let Some((block, block_level)) = self.select(0) {
            while level < block_level {
                blocks.sort();
                let mut maybe_later = None;
                while let Some(before) = blocks.pop() {
                    if let Some(later) = maybe_later.take() {
                        if before + (16 << level) == later
                            && before & ((16 << (level + 1)) - 1) == 0
                        {
                            // Coalesce
                            debug!(
                                "Coalesced block {before:0b} and {later:0b} of size {}",
                                16 << level
                            );
                            next_level_blocks.push(before);
                        } else {
                            // Search in sub blocks
                            let mut sub_blocks = self.sub_blocks.lock();
                            if let Some(at) =
                                sub_blocks.iter().position(|&sub| sub == later + level)
                            {
                                sub_blocks.remove(at);
                                let size = 16 << level;
                                // This is safe since we have exclusive access and we've reconstructed the sub block.
                                unsafe {
                                    alloc::dealloc(
                                        later as *mut u8,
                                        Layout::from_size_align(size, size)
                                            .expect("Shouldn't fail"),
                                    );
                                }
                            } else {
                                debug!("Leaked block {later:0b} of size {}", 16 << level);
                            }
                            maybe_later = Some(before);
                        }
                    } else {
                        maybe_later = Some(before);
                    }
                }

                if let Some(later) = maybe_later {
                    // Search in sub blocks
                    let mut sub_blocks = self.sub_blocks.lock();
                    if let Some(at) = sub_blocks.iter().position(|&sub| sub == later + level) {
                        sub_blocks.remove(at);
                        let size = 16 << level;
                        // This is safe since we have exclusive access and we've reconstructed the sub block.
                        unsafe {
                            alloc::dealloc(
                                later as *mut u8,
                                Layout::from_size_align(size, size).expect("Shouldn't fail"),
                            );
                        }
                    } else {
                        debug!("Leaked block {later:0b} of size {}", 16 << level);
                    }
                }

                std::mem::swap(&mut blocks, &mut next_level_blocks);
                level += 1;
            }

            blocks.push(block.as_ptr() as usize);
        }

        while level < 8 {
            blocks.sort();
            let mut maybe_later = None;
            while let Some(before) = blocks.pop() {
                if let Some(later) = maybe_later.take() {
                    if before + (16 << level) == later && before & ((16 << (level + 1)) - 1) == 0 {
                        // Coalesce
                        debug!(
                            "Coalesced block {before:0b} and {later:0b} of size {}",
                            16 << level
                        );
                        next_level_blocks.push(before);
                    } else {
                        // Search in sub blocks
                        let mut sub_blocks = self.sub_blocks.lock();
                        if let Some(at) = sub_blocks.iter().position(|&sub| sub == later + level) {
                            sub_blocks.remove(at);
                            let size = 16 << level;
                            // This is safe since we have exclusive access and we've reconstructed the sub block.
                            unsafe {
                                alloc::dealloc(
                                    later as *mut u8,
                                    Layout::from_size_align(size, size).expect("Shouldn't fail"),
                                );
                            }
                        } else {
                            debug!("Leaked block {later:0b} of size {}", 16 << level);
                        }
                        maybe_later = Some(before);
                    }
                } else {
                    maybe_later = Some(before);
                }
            }

            if let Some(later) = maybe_later {
                // Search in sub blocks
                let mut sub_blocks = self.sub_blocks.lock();
                if let Some(at) = sub_blocks.iter().position(|&sub| sub == later + level) {
                    sub_blocks.remove(at);
                    let size = 16 << level;
                    // This is safe since we have exclusive access and we've reconstructed the sub block.
                    unsafe {
                        alloc::dealloc(
                            later as *mut u8,
                            Layout::from_size_align(size, size).expect("Shouldn't fail"),
                        );
                    }
                } else {
                    debug!("Leaked block {later:0b} of size {}", 16 << level);
                }
            }

            std::mem::swap(&mut blocks, &mut next_level_blocks);
            level += 1;
        }

        // Free
        for block in blocks {
            debug!("Deallocating block {block:0b} of size {}", 16 << level);
            // This is safe since we have exclusive access and we've reconstructed the blocks.
            unsafe {
                alloc::dealloc(
                    block as *mut u8,
                    Layout::from_size_align(BLOCK_SIZE, BLOCK_SIZE).expect("Shouldn't fail"),
                );
            }
        }
    }
}

#[derive(Clone)]
pub struct SharedMiniAllocator(Arc<MiniAllocator>);

impl SharedMiniAllocator {
    pub fn new() -> Self {
        Self(Arc::new(MiniAllocator::new()))
    }
}

unsafe impl Allocator for SharedMiniAllocator {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.0.allocate(layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.0.deallocate(ptr, layout)
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.0.allocate_zeroed(layout)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.0.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.0.grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.0.shrink(ptr, old_layout, new_layout)
    }
}

impl Borrow<MiniAllocator> for SharedMiniAllocator {
    fn borrow(&self) -> &MiniAllocator {
        &self.0
    }
}

impl Deref for SharedMiniAllocator {
    type Target = MiniAllocator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn add_slices(allocator: &MiniAllocator, n: usize, start: u8) -> Vec<(NonNull<[u8]>, Layout)> {
        let mut slices = Vec::new();
        let mut sum = start;

        // Allocate and write data to slices
        for i in 0..n {
            let layout = Layout::from_size_align(i, 1).unwrap();
            let mut slice_ptr = allocator.allocate(layout).unwrap();
            let slice = unsafe { slice_ptr.as_mut() };
            for byte in slice {
                *byte = sum;
                sum = sum.wrapping_add(1);
            }
            slices.push((slice_ptr, layout));
        }

        slices
    }

    fn validate_slices(slices: &[(NonNull<[u8]>, Layout)], start: u8) {
        // Check that data is correct
        let mut sum = start;
        for (slice_ptr, _) in slices {
            let slice = unsafe { slice_ptr.as_ref() };
            for byte in slice {
                assert_eq!(*byte, sum);
                sum = sum.wrapping_add(1);
            }
        }
    }

    fn deallocate_slices(allocator: &MiniAllocator, slices: Vec<(NonNull<[u8]>, Layout)>) {
        // Deallocate slices
        for (slice_ptr, layout) in slices {
            unsafe { allocator.deallocate(slice_ptr.cast(), layout) };
        }
    }

    /// Expects that everything has been deallocated.
    fn check_for_memory_leaks(allocator: &mut MiniAllocator) {
        assert_eq!(allocator.allocated(), allocator.capacity());
    }

    #[test]
    fn test() {
        let mut allocator = MiniAllocator::new();
        let mut ptrs = Vec::new();
        for i in 0..100 {
            let size = i + 1;
            let layout = Layout::from_size_align(size, (size + 1).next_power_of_two() / 2).unwrap();
            let ptr = allocator.allocate(layout).unwrap();
            ptrs.push((ptr, layout));
        }

        deallocate_slices(&allocator, ptrs);

        check_for_memory_leaks(&mut allocator);
    }

    /// Allocate [u8], write to it different data, and repeat several times.
    /// Then check that all data is correct.
    #[test]
    fn test_overlap() {
        let mut allocator = MiniAllocator::new();

        let slices = add_slices(&allocator, 100, 0);
        validate_slices(&slices, 0);
        deallocate_slices(&allocator, slices);

        check_for_memory_leaks(&mut allocator);
    }

    #[test]
    fn multi_thread() {
        let threads = 16;
        let repetitions = 100;
        let size = 2_000_000;
        let allocator = Arc::new(MiniAllocator::new());

        let mut handles = Vec::new();
        for i in 0..threads {
            let allocator = allocator.clone();
            handles.push(std::thread::spawn(move || {
                for j in 0..repetitions {
                    let slices = add_slices(
                        &allocator,
                        ((size as f64).sqrt() / 2.0) as usize,
                        (i * j) as u8,
                    );
                    validate_slices(&slices, (i * j) as u8);
                    deallocate_slices(&allocator, slices);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut allocator = match Arc::try_unwrap(allocator) {
            Ok(alloc) => alloc,
            _ => panic!(),
        };
        check_for_memory_leaks(&mut allocator);
    }
}
