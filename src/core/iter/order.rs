use super::{IndexBase, KeyPermit, KeySet, Keys, SlotAccess, TopKey};
use crate::core::{AnyContainer, DynItem, Item, Permit};
use radix_heap::{Radix, RadixHeapMap};
use std::{cmp::Reverse, collections::VecDeque, marker::PhantomData};

pub struct Depth;

pub struct Breadth;

pub struct Topological<F, T>(pub F, pub PhantomData<T>);

pub struct TopologicalKey;

pub trait Order<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized, SK> {
    type Keys: KeyPermit + KeySet + Default;
    type Key: Ord;
    type Queue<T>: Queue<(Self::Key, SK), T> + Default;

    fn ordering(&mut self, input: &NI, slot: SlotAccess<C, P, I>) -> Option<Self::Key>;
}

impl<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized, SK> Order<NI, C, P, I, SK>
    for Depth
{
    type Keys = Keys;
    type Key = ();
    type Queue<T> = LifoQueue<(Self::Key, SK), T>;

    fn ordering(&mut self, _: &NI, _: SlotAccess<C, P, I>) -> Option<Self::Key> {
        Some(())
    }
}

impl<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized, SK> Order<NI, C, P, I, SK>
    for Breadth
{
    type Keys = Keys;
    type Key = ();
    type Queue<T> = FifoQueue<(Self::Key, SK), T>;

    fn ordering(&mut self, _: &NI, _: SlotAccess<C, P, I>) -> Option<Self::Key> {
        Some(())
    }
}

impl<
        F: FnMut(&NI, SlotAccess<C, P, I>) -> Option<K>,
        K: Radix + Ord + Copy,
        NI,
        C: AnyContainer + ?Sized,
        P: Permit,
        I: DynItem + ?Sized,
        SK: Radix + Ord + Copy,
    > Order<NI, C, P, I, SK> for Topological<F, K>
{
    type Keys = Keys;
    type Key = K;
    /// Min queue
    type Queue<T> = RadixHeapMap<Reverse<(K, SK)>, T>;

    fn ordering(&mut self, input: &NI, slot: SlotAccess<C, P, I>) -> Option<K> {
        (self.0)(input, slot)
    }
}

impl<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized, SK: Radix + Ord + Copy>
    Order<NI, C, P, I, SK> for TopologicalKey
{
    type Keys = TopKey;
    type Key = IndexBase;
    /// Min queue
    type Queue<T> = RadixHeapMap<Reverse<(IndexBase, SK)>, T>;

    fn ordering(&mut self, _: &NI, slot: SlotAccess<C, P, I>) -> Option<Self::Key> {
        Some(slot.key().index().get())
    }
}

pub trait Queue<K, T> {
    /// false if not pushed because key is out of order.
    fn push(&mut self, key: K, item: T) -> bool;

    fn peek(&mut self) -> Option<(&K, &T)>;

    fn pop(&mut self) -> Option<(K, T)>;

    fn into_iter(self) -> impl Iterator<Item = (K, T)>;
}

pub struct LifoQueue<K, T> {
    queue: VecDeque<(K, T)>,
}

impl<K, T> Queue<K, T> for LifoQueue<K, T> {
    fn push(&mut self, key: K, item: T) -> bool {
        self.queue.push_front((key, item));
        true
    }

    fn peek(&mut self) -> Option<(&K, &T)> {
        self.queue.front().map(|(key, item)| (key, item))
    }

    fn pop(&mut self) -> Option<(K, T)> {
        self.queue.pop_front()
    }

    fn into_iter(self) -> impl Iterator<Item = (K, T)> {
        self.queue.into_iter()
    }
}

impl<K, T> Default for LifoQueue<K, T> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

pub struct FifoQueue<K, T> {
    queue: VecDeque<(K, T)>,
}

impl<K, T> Queue<K, T> for FifoQueue<K, T> {
    fn push(&mut self, key: K, item: T) -> bool {
        self.queue.push_back((key, item));
        true
    }

    fn peek(&mut self) -> Option<(&K, &T)> {
        self.queue.front().map(|(key, item)| (key, item))
    }

    fn pop(&mut self) -> Option<(K, T)> {
        self.queue.pop_front()
    }

    fn into_iter(self) -> impl Iterator<Item = (K, T)> {
        self.queue.into_iter()
    }
}

impl<K, T> Default for FifoQueue<K, T> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl<K: Radix + Ord + Copy, T> Queue<K, T> for RadixHeapMap<Reverse<K>, T> {
    fn push(&mut self, key: K, item: T) -> bool {
        let key = Reverse(key);
        if self.top().map(|top| key < top).unwrap_or(true) {
            self.push(key, item);
            true
        } else {
            false
        }
    }

    fn peek(&mut self) -> Option<(&K, &T)> {
        RadixHeapMap::peek(self).map(|(key, item)| (&key.0, item))
    }

    fn pop(&mut self) -> Option<(K, T)> {
        self.pop().map(|(key, data)| (key.0, data))
    }

    fn into_iter(self) -> impl Iterator<Item = (K, T)> {
        IntoIterator::into_iter(self).map(|(key, data)| (key.0, data))
    }
}
