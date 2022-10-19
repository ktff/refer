use crate::core::*;
use bitvec::prelude::*;
use std::{
    alloc::{self, Layout},
    any::{Any, TypeId},
    cell::SyncUnsafeCell,
    collections::HashSet,
    marker::PhantomData,
    mem::MaybeUninit,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

const STARTING_SIZE: usize = 256;
const MAX_TABLE_SIZE: usize = 4096;

/// A simple table container of items of the same type.
/// Optimized to reduce memory overhead.
pub struct TableContainer<
    T: Send + Sync + 'static,
    S: Shell<T = T> + Default = super::item::SizedShell<T>,
    A: alloc::Allocator + Sync + Send + 'static = alloc::Global,
> where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    tables: Vec<TableBox<T, S>, A>,
    reserved: Vec<SubKey<T>, A>,
    /// Tables with possibly free slot.
    /// Descending order
    free_tables: Vec<usize, A>,
    key_len: u32,
    count: usize,
}

impl<T: Send + Sync + 'static, S: Shell<T = T> + Default> TableContainer<T, S, alloc::Global>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    pub fn new(key_len: u32) -> Self {
        // assert!(std::mem::size_of::<Table<T, S>>() <= MAX_TABLE_SIZE);

        Self {
            tables: Vec::new(),
            reserved: Vec::new(),
            free_tables: Vec::new(),
            key_len: key_len,
            count: 0,
        }
    }
}

impl<
        T: Send + Sync + 'static,
        S: Shell<T = T> + Default,
        A: alloc::Allocator + Sync + Send + Clone + 'static,
    > TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    pub fn new_in(key_len: u32, alloc: A) -> Self {
        // assert!(std::mem::size_of::<Table<T, S>>() <= MAX_TABLE_SIZE);

        Self {
            tables: Vec::new_in(alloc.clone()),
            reserved: Vec::new_in(alloc.clone()),
            free_tables: Vec::new_in(alloc),
            key_len: key_len,
            count: 0,
        }
    }

    /// Number items in this collection
    pub fn len(&self) -> usize {
        self.count
    }

    /// Memory used directly by this container.
    pub fn used_memory(&self) -> usize {
        self.tables.capacity() * std::mem::size_of::<TableBox<T, S>>()
            + self
                .tables
                .iter()
                .map(|t| std::mem::size_of_val::<Table<T, S>>(t))
                .sum::<usize>()
            + self.reserved.capacity() * std::mem::size_of::<Key<T>>()
            + self.free_tables.capacity() * std::mem::size_of::<usize>()
    }

    pub fn alloc(&self) -> &A {
        self.tables.allocator()
    }
}

impl<
        T: Send + Sync + 'static,
        S: Shell<T = T> + Default,
        A: alloc::Allocator + Sync + Send + 'static,
    > TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    /// Splits a key into table index and slot index.
    fn split_key(&self, key: SubKey<T>) -> (usize, usize) {
        let i = key.index(self.key_len).as_usize();
        let table_index = i / slots_len::<T, S>();
        let slot_index = i % slots_len::<T, S>();
        (table_index, slot_index)
    }

    fn join_key(&self, table_index: usize, slot_index: usize) -> SubKey<T> {
        let index = table_index * slots_len::<T, S>() + slot_index;
        SubKey::new(
            self.key_len,
            Index(NonZeroU64::new(index as u64).expect("Zero index is allocated")),
        )
    }

    fn add_free_hint(&mut self, table_index: usize) {
        if let Err(i) = self.free_tables.binary_search(&table_index) {
            self.free_tables.insert(i, table_index);
        }
    }
}

impl<
        T: Send + Sync + 'static,
        S: Shell<T = T> + Default,
        A: alloc::Allocator + Sync + Send + Clone + 'static,
    > Allocator<T> for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    type Alloc = A;

    type R = ();

    fn reserve(&mut self, _: Option<&T>, _: Self::R) -> Option<(ReservedKey<T>, &A)> {
        // Check free tables
        while let Some(&table_index) = self.free_tables.last() {
            let table = &self.tables[table_index];
            for slot_index in table.taken.iter_zeros() {
                if slot_index < table.slots.len() {
                    // Avoid allocating zero index
                    if table_index + slot_index > 0 {
                        let key = self.join_key(table_index, slot_index);
                        if self.split_key(key) == (table_index, slot_index) {
                            if !self.reserved.contains(&key) {
                                self.reserved.push(key);
                                return Some((ReservedKey::new(key), self.alloc()));
                            }
                        } else {
                            // We are out of keys in this table
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            // No free slots
            self.free_tables.pop();
        }

        // No free slots, allocate new table
        let table_index = self.tables.len();

        // Check key range
        let max_key = (table_index + 1) * slots_len::<T, S>();
        if max_key.checked_shr(self.key_len).unwrap_or(0) >= 1 {
            // Out of keys
            return None;
        }

        // New table
        let key = if let Some(table) = self.tables.get_mut(0) {
            let len = table.slots.len();
            if len < slots_len::<T, S>() {
                let new_len = (len * 2).min(slots_len::<T, S>());
                table.grow(self.free_tables.allocator(), new_len);
                debug_assert!(self.tables[0].slots.len() >= new_len);
                debug_assert!(len < new_len);
                self.join_key(0, len)
            } else {
                self.tables
                    .push(TableBox::new_in(self.alloc(), MAX_TABLE_SIZE));
                self.join_key(table_index, 0)
            }
        } else {
            self.tables
                .push(TableBox::new_in(self.alloc(), STARTING_SIZE));
            self.join_key(0, 1)
        };

        // New key
        self.reserved.push(key);
        Some((ReservedKey::new(key), self.alloc()))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let key = key.take();
        let i = self
            .reserved
            .iter()
            .position(|x| *x == key)
            .expect("Reservation doesn't exist");
        self.reserved.swap_remove(i);

        // Add free hint
        let (table_index, _) = self.split_key(key);
        self.add_free_hint(table_index);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        let key = key.take();
        self.cancel(ReservedKey::new(key));
        let (table_index, slot_index) = self.split_key(key);
        let table = &mut self.tables[table_index];
        assert!(!*table
            .taken
            .get(slot_index)
            .expect("Slot index out of bounds"));
        table.slots[slot_index].write((SyncUnsafeCell::new(item), Default::default()));
        table.taken.set(slot_index, true);

        self.count += 1;

        key
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized,
    {
        let (table_index, slot_index) = self.split_key(key);

        let table = &mut self.tables[table_index];
        if table.taken.get(slot_index).map(|bit| *bit) == Some(true) {
            table.taken.set(slot_index, false);
            // UNSAFE: We know that the slot is occupied.
            let (item, _) = unsafe { table.slots[slot_index].assume_init_read() };
            self.count -= 1;

            // Add free hint
            self.add_free_hint(table_index);

            Some((item.into_inner(), self.alloc()))
        } else {
            None
        }
    }
}

impl<
        T: AnyItem,
        S: Shell<T = T> + Default,
        A: alloc::Allocator + Sync + Send + Clone + 'static,
    > Container<T> for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    type GroupItem = ();

    type Shell = S;

    type SlotIter<'a> = impl Iterator<
        Item = (
            SubKey<T>,
            (&'a SyncUnsafeCell<T>, &'a ()),
            &'a SyncUnsafeCell<S>,
            &'a A,
        )> where Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<((&SyncUnsafeCell<T>, &()), &SyncUnsafeCell<Self::Shell>, &A)> {
        let (table_index, slot_index) = self.split_key(key);
        let table = self.tables.get(table_index)?;
        // Check that the slot is taken/initialized
        if table.taken[slot_index] {
            // This is safe since we've checked that this slot is taken
            let (item, shell) = unsafe { table.get(slot_index) };

            Some(((item, &()), shell, self.tables.allocator()))
        } else {
            None
        }
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        // This is safe since Vec::iter and slice iter guarantees that each element
        // is returned only once.
        Some(
            self.tables
                .iter()
                .enumerate()
                .filter(|(_, table)| table.taken.some())
                .flat_map(move |(table_index, table)| {
                    table.taken.iter_ones().map(move |slot_index| {
                        // This is safe since we've checked that this slot is taken
                        let (item, shell) = unsafe { table.get(slot_index) };
                        (
                            self.join_key(table_index, slot_index),
                            (item, &()),
                            shell,
                            self.tables.allocator(),
                        )
                    })
                }),
        )
    }
}

impl<
        T: AnyItem,
        S: Shell<T = T> + Default,
        A: alloc::Allocator + Sync + Send + Clone + 'static,
    > AnyContainer for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&SyncUnsafeCell<dyn AnyItem>, &dyn Any),
        &SyncUnsafeCell<dyn AnyShell>,
        &dyn std::alloc::Allocator,
    )> {
        let ((item, group_data), shell, alloc) = self.get_slot(key.downcast::<T>()?)?;
        Some((
            (item as &SyncUnsafeCell<dyn AnyItem>, group_data as &dyn Any),
            shell as &SyncUnsafeCell<dyn AnyShell>,
            alloc as &dyn std::alloc::Allocator,
        ))
    }

    fn unfill_any(&mut self, key: AnySubKey) {
        if let Some(key) = key.downcast() {
            self.unfill(key);
        }
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        if key == TypeId::of::<T>() {
            self.tables
                .iter()
                .enumerate()
                .find_map(|(table_index, table)| {
                    table
                        .first()
                        .map(|slot_index| self.join_key(table_index, slot_index).into())
                })
        } else {
            None
        }
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        if let Some(key) = key.downcast::<T>() {
            let (table_index, slot_index) = self.split_key(key);
            let table = self.tables.get(table_index)?;
            if let Some(slot_index) = table.next(slot_index) {
                Some(self.join_key(table_index, slot_index).into())
            } else {
                self.tables
                    .iter()
                    .enumerate()
                    .skip(table_index + 1)
                    .find_map(|(table_index, table)| {
                        table
                            .first()
                            .map(|slot_index| self.join_key(table_index, slot_index).into())
                    })
            }
        } else {
            None
        }
    }

    fn types(&self) -> HashSet<TypeId> {
        let mut set = HashSet::new();
        set.insert(TypeId::of::<T>());
        set
    }
}

// Drop for TableContainer
impl<
        T: Send + Sync + 'static,
        S: Shell<T = T> + Default,
        A: alloc::Allocator + Sync + Send + 'static,
    > Drop for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn drop(&mut self) {
        // Drop all items
        for table in self.tables.drain(..) {
            table.drop(self.free_tables.allocator());
        }
    }
}

struct TableBox<T: 'static, S: Shell<T = T> + Default>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    pointer: NonNull<Table<T, S>>,
    _marker: PhantomData<(T, S)>,
}

unsafe impl<T: 'static, S: Shell<T = T> + Default> Sync for TableBox<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
}

unsafe impl<T: 'static, S: Shell<T = T> + Default> Send for TableBox<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
}

impl<T: 'static, S: Shell<T = T> + Default> TableBox<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn new_in<A: std::alloc::Allocator>(allocator: &A, size: usize) -> Self {
        let len = if size < MAX_TABLE_SIZE {
            size / std::mem::size_of::<MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>>()
        } else {
            slots_len::<T, S>()
        };
        let table = unsafe {
            // This is safe since we are constructing a Unsized with a slice.
            let new_layout = {
                let ptr: *const Table<T, S> = std::ptr::from_raw_parts(std::ptr::null(), len);
                Layout::for_value_raw(ptr)
            };
            let ptr = allocator.allocate(new_layout).expect("Failed to allocate");

            // Init table
            let extra_capacity = (ptr.len() - new_layout.size())
                / std::mem::size_of::<MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>>();
            let final_len = len + extra_capacity;

            // Safe since Table is freshly allocated
            // NOTE: This should be wrapped in MaybeUninit but it doesn't support unsized types.
            //        But having uninit bit array should be fine.
            let table: *mut Table<T, S> =
                std::ptr::from_raw_parts_mut(ptr.as_ptr() as *mut (), final_len);
            (&mut *table).taken = BitArray::ZERO;
            // Initializing the `slots` field is not necessary since they are MaybeUninit

            table
        };
        Self {
            pointer: NonNull::new(table).expect("Null pointer allocated"),
            _marker: PhantomData,
        }
    }

    fn grow<A: std::alloc::Allocator>(&mut self, allocator: &A, len: usize) {
        unsafe {
            // This is safe since we are constructing a Unsized with a slice.
            let new_layout = {
                let ptr: *const Table<T, S> = std::ptr::from_raw_parts(std::ptr::null(), len);
                Layout::for_value_raw(ptr)
            };
            let layout = Layout::for_value::<Table<T, S>>(self.pointer.as_ref());
            let ptr = allocator
                .grow(
                    NonNull::new_unchecked(self.pointer.as_ptr() as *mut u8),
                    layout,
                    new_layout,
                )
                .expect("Failed to grow");
            // Init table
            let extra_capacity = (ptr.len() - new_layout.size())
                / std::mem::size_of::<MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>>();
            let final_len = len + extra_capacity;

            // Safe since Table is freshly allocated
            // NOTE: This should be wrapped in MaybeUninit but it doesn't support unsized types.
            //        But having uninit bit array should be fine.
            let table: *mut Table<T, S> =
                std::ptr::from_raw_parts_mut(ptr.as_ptr() as *mut (), final_len);

            *self = Self {
                pointer: NonNull::new(table).expect("Null pointer allocated"),
                _marker: PhantomData,
            };
        }
    }

    fn drop<A: std::alloc::Allocator>(mut self, allocator: &A) {
        let table: &mut Table<T, S> = &mut self;
        for i in table.taken.iter_ones() {
            // SAFETY: We've checked that this slot is taken
            unsafe {
                table.slots.get_mut(i).map(|slot| slot.assume_init_drop());
            }
        }

        let ptr: NonNull<u8> = self.pointer.cast();
        let layout = Layout::for_value::<Table<T, S>>(&self);
        unsafe { allocator.deallocate(ptr, layout) }
    }
}

impl<T: 'static, S: Shell<T = T> + Default> Deref for TableBox<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    type Target = Table<T, S>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T: 'static, S: Shell<T = T> + Default> DerefMut for TableBox<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.pointer.as_mut() }
    }
}

struct Table<T: 'static, S: Shell<T = T> + Default>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    // Bitfield
    taken: BitArray<[u64; taken_len::<T, S>()], Lsb0>,
    // Slots
    slots: [MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>],
}

impl<T: 'static, S: Shell<T = T> + Default> Table<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    /// UNSAFETY: The caller must ensure that the slots are initialized
    unsafe fn get(&self, index: usize) -> (&SyncUnsafeCell<T>, &SyncUnsafeCell<S>) {
        let (ref item, ref shell) = *self.slots[index].assume_init_ref();
        (item, shell)
    }

    /// First taken slot
    fn first(&self) -> Option<usize> {
        self.taken.iter_ones().next()
    }

    /// Following taken slot
    fn next(&self, index: usize) -> Option<usize> {
        self.taken.as_bitslice()[index + 1..]
            .iter_ones()
            .next()
            .map(|i| i + index + 1)
    }
}

impl<T: 'static, S: Shell<T = T> + Default> Drop for Table<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn drop(&mut self) {
        for i in self.taken.iter_ones() {
            // SAFETY: We've checked that this slot is taken
            unsafe {
                self.slots.get_mut(i).map(|slot| slot.assume_init_drop());
            }
        }
    }
}

pub const fn slots_len<T, S>() -> usize {
    let slot_size = std::mem::size_of::<MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>>();

    // const fn taken_len<T, S>() -> usize {
    // TODO: This can be computed accurately
    // Assume worst case
    let max_slots = MAX_TABLE_SIZE / slot_size;
    let taken_len = max_slots.div_ceil(64);
    // }
    let taken_size = std::mem::size_of::<u64>() * taken_len;
    // TODO: This can be computed accurately
    // Assume worst case
    let max_padding_size =
        std::mem::align_of::<MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>>()
            .saturating_sub(std::mem::align_of::<u64>());
    // //Box is storing allocator on heap.
    let allocator_size = 0; //std::mem::size_of::<usize>();
    let max_slots = (MAX_TABLE_SIZE - taken_size - max_padding_size - allocator_size) / slot_size;
    max_slots
}

pub const fn taken_len<T, S>() -> usize {
    // TODO: This can be computed accurately
    // Assume worst case
    let slot_size = std::mem::size_of::<MaybeUninit<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>>();
    let max_slots = MAX_TABLE_SIZE / slot_size;
    max_slots.div_ceil(64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::owned::Owned;
    use std::any::Any;

    #[test]
    fn add_items() {
        let n = 20;
        let mut container = Owned::new(TableContainer::<usize>::new(64));

        let keys = (0..n)
            .map(|i| container.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        for (i, key) in keys.iter().enumerate() {
            assert_eq!(container.get(*key).unwrap().0, (&i, &()));
        }
    }

    #[test]
    fn reserve_cancel() {
        let mut container = Owned::new(TableContainer::<usize>::new(1));

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        assert!(container.reserve(Some(&item), ()).is_none());

        container.cancel(key);
        assert!(container.reserve(Some(&item), ()).is_some());
    }

    #[test]
    fn add_unfill() {
        let mut container = Owned::new(TableContainer::<usize>::new(10));

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        assert_eq!(container.items().get(key).unwrap().0, &item);
        assert_eq!(container.unfill(key.into()).unwrap().0, item);
        assert!(container.items().get(key).is_none());
    }

    #[test]
    fn iter() {
        let n = 20;
        let mut container = Owned::new(TableContainer::<usize>::new(10));

        let mut keys = (0..n)
            .map(|i| (container.add_with(i, ()).unwrap(), i))
            .collect::<Vec<_>>();

        keys.sort();

        assert_eq!(
            keys,
            container
                .items()
                .iter()
                .map(|(key, (&item, _))| (key, item))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn get_any() {
        let mut container = Owned::new(TableContainer::<usize>::new(10));

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        assert_eq!(
            (container.items_mut().get_any(key.into()).unwrap().0 as &dyn Any)
                .downcast_ref::<usize>(),
            Some(&item)
        );
    }

    #[test]
    fn unfill_any() {
        let mut container = TableContainer::<usize>::new(10);

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        let key = container.fulfill(key, item);

        container.unfill_any(key.into());
        assert!(container.get_slot(key.into()).is_none());
    }

    #[test]
    fn iter_keys() {
        let n = 20;
        let mut container = Owned::new(TableContainer::<usize>::new(8));

        let mut keys = (0..n)
            .map(|i| container.add_with(i, ()).unwrap().into())
            .collect::<Vec<AnyKey>>();

        keys.sort();

        let any_keys = std::iter::successors(container.first(keys[0].type_id()), |key| {
            container.next(*key)
        })
        .take(30)
        .collect::<Vec<_>>();

        assert_eq!(keys, any_keys);
    }
}
