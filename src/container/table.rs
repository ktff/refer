use crate::core::*;
use bitvec::prelude::*;
use std::{
    alloc,
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
    mem::MaybeUninit,
    num::NonZeroU64,
    ptr::addr_of_mut,
};

use super::item::{SizedShell, Slot};

const MAX_TABLE_SIZE: usize = 4096;

/// A simple table container of items of the same type.
/// Optimized to reduce memory overhead.
pub struct TableContainer<
    T: 'static,
    S: Shell<T = T> + Default = super::item::SizedShell<T>,
    A: alloc::Allocator + 'static = alloc::Global,
> where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    tables: Vec<Box<Table<T, S>, A>, A>,
    reserved: Vec<SubKey<T>, A>,
    /// Tables with possibly free slot.
    /// Descending order
    free_tables: Vec<usize, A>,
    key_len: u32,
    count: usize,
}

impl<T: 'static, S: Shell<T = T> + Default> TableContainer<T, S, alloc::Global>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    pub fn new(key_len: u32) -> Self {
        assert!(std::mem::size_of::<Table<T, S>>() <= MAX_TABLE_SIZE);

        Self {
            tables: Vec::new(),
            reserved: Vec::new(),
            free_tables: Vec::new(),
            key_len: key_len,
            count: 0,
        }
    }
}

impl<T: 'static, S: Shell<T = T> + Default, A: alloc::Allocator + Clone + 'static>
    TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    pub fn new_in(key_len: u32, alloc: A) -> Self {
        assert!(std::mem::size_of::<Table<T, S>>() <= MAX_TABLE_SIZE);
        // println!(
        //     "Table size: {}, slot size: {}, slot count: {}, bitarray size: {}",
        //     std::mem::size_of::<Table<T, S>>(),
        //     std::mem::size_of::<MaybeUninit<(UnsafeCell<T>, UnsafeCell<S>)>>(),
        //     slots_len::<T, S>(),
        //     std::mem::size_of::<BitArray<[u64; taken_len::<T, S>()], Lsb0>>()
        // );

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
        self.tables.capacity() * std::mem::size_of::<Box<Table<T, S>, A>>()
            + self.tables.len() * std::mem::size_of::<Table<T, S>>()
            + self.reserved.capacity() * std::mem::size_of::<Key<T>>()
            + self.free_tables.capacity() * std::mem::size_of::<usize>()
    }

    pub fn alloc(&self) -> &A {
        self.tables.allocator()
    }
}

impl<T: 'static, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> TableContainer<T, S, A>
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

impl<T: 'static, S: Shell<T = T> + Default, A: alloc::Allocator + Clone + 'static> Allocator<T>
    for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    type Alloc = A;

    type R = ();

    fn reserve(&mut self, _: Option<&T>, _: Self::R) -> Option<(ReservedKey<T>, &A)> {
        // Check free tables
        while let Some(&table_index) = self.free_tables.last() {
            for slot_index in self.tables[table_index].taken.iter_zeros() {
                if slot_index < slots_len::<T, S>() {
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
        let max_key = table_index * slots_len::<T, S>();
        if (table_index * slots_len::<T, S>())
            .checked_shr(self.key_len)
            .unwrap_or(0)
            >= 1
        {
            // Out of keys
            return None;
        }

        // New table
        let table = Table::new_in(self.alloc().clone());
        self.tables.push(table);

        // Check for zero index
        let slot_index = if table_index == 0 { 1 } else { 0 };

        // New key
        let key = self.join_key(table_index, slot_index);
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
        table.slots[slot_index].write((UnsafeCell::new(item), Default::default()));
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

impl<T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> !Sync
    for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
}

impl<T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + Clone + 'static> Container<T>
    for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    type GroupItem = ();

    type Shell = S;

    type SlotIter<'a> = impl Iterator<
        Item = (
            SubKey<T>,
            (&'a UnsafeCell<T>, &'a ()),
            &'a UnsafeCell<S>,
            &'a A,
        )> where Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<((&UnsafeCell<T>, &()), &UnsafeCell<Self::Shell>, &A)> {
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
                .filter(|(i, table)| table.taken.some())
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

impl<T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + Clone + 'static> AnyContainer
    for TableContainer<T, S, A>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&UnsafeCell<dyn AnyItem>, &dyn Any),
        &UnsafeCell<dyn AnyShell>,
        &dyn std::alloc::Allocator,
    )> {
        let ((item, group_data), shell, alloc) = self.get_slot(key.downcast::<T>()?)?;
        Some((
            (item as &UnsafeCell<dyn AnyItem>, group_data as &dyn Any),
            shell as &UnsafeCell<dyn AnyShell>,
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

struct Table<T: 'static, S: Shell<T = T> + Default>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    // Bitfield
    taken: BitArray<[u64; taken_len::<T, S>()], Lsb0>,
    // Slots
    slots: [MaybeUninit<(UnsafeCell<T>, UnsafeCell<S>)>; slots_len::<T, S>()],
}

impl<T: 'static, S: Shell<T = T> + Default> Table<T, S>
where
    [(); taken_len::<T, S>()]: Sized,
    [(); slots_len::<T, S>()]: Sized,
{
    fn new() -> Self {
        Self {
            taken: BitArray::ZERO,
            slots: MaybeUninit::uninit_array(),
        }
    }

    fn new_in<A: std::alloc::Allocator>(alloc: A) -> Box<Self, A> {
        unsafe {
            let mut table = Box::<Self, A>::new_uninit_in(alloc);
            let ptr = table.as_mut_ptr();
            // Initializing the `taken` field
            addr_of_mut!((*ptr).taken).write(BitArray::ZERO);

            // Initializing the `slots` field is not necessary since they are MaybeUninit

            // All the fields are initialized, so we call `assume_init`
            table.assume_init()
        }
    }

    /// UNSAFETY: The caller must ensure that the slots are initialized
    unsafe fn get(&self, index: usize) -> (&UnsafeCell<T>, &UnsafeCell<S>) {
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
    let slot_size = std::mem::size_of::<MaybeUninit<(UnsafeCell<T>, UnsafeCell<S>)>>();

    // const fn taken_len<T, S>() -> usize {
    // TODO: This can be computed accurately
    // Assume worst case
    let max_slots = MAX_TABLE_SIZE / slot_size;
    let taken_len = max_slots.div_ceil(64);
    // }
    let taken_size = std::mem::size_of::<u64>() * taken_len;
    // TODO: This can be computed accurately
    // Assume worst case
    let max_padding_size = std::mem::align_of::<MaybeUninit<(UnsafeCell<T>, UnsafeCell<S>)>>()
        .saturating_sub(std::mem::align_of::<u64>());
    // Box is storing allocator on heap.
    let allocator_size = std::mem::size_of::<usize>();
    let max_slots = (MAX_TABLE_SIZE - taken_size - max_padding_size - allocator_size) / slot_size;
    max_slots
}

pub const fn taken_len<T, S>() -> usize {
    // TODO: This can be computed accurately
    // Assume worst case
    let slot_size = std::mem::size_of::<MaybeUninit<(UnsafeCell<T>, UnsafeCell<S>)>>();
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
