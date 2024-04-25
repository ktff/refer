use super::{IndexBase, KeyPermit, KeySet, Keys, SlotAccess, TopKey};
use crate::core::{AnyContainer, DynItem, Permit};
use radix_heap::{Radix, RadixHeapMap};
use std::{cmp::Reverse, collections::VecDeque, marker::PhantomData};

pub struct Depth;

pub struct Breadth;

pub struct Topological<F, T>(pub F, pub PhantomData<T>);

pub struct TopologicalKey;

pub trait Order<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized> {
    type Keys: KeyPermit + KeySet + Default;
    type Key;
    type Queue<T>: Queue<Self::Key, T> + Default;

    fn ordering(&mut self, input: &NI, slot: SlotAccess<C, P, I>) -> Option<Self::Key>;
}

impl<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized> Order<NI, C, P, I> for Depth {
    type Keys = Keys;
    type Key = ();
    type Queue<T> = LifoQueue<T>;

    fn ordering(&mut self, _: &NI, _: SlotAccess<C, P, I>) -> Option<Self::Key> {
        Some(())
    }
}

impl<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized> Order<NI, C, P, I> for Breadth {
    type Keys = Keys;
    type Key = ();
    type Queue<T> = FifoQueue<T>;

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
    > Order<NI, C, P, I> for Topological<F, K>
{
    type Keys = Keys;
    type Key = K;
    /// Min queue
    type Queue<T> = RadixHeapMap<Reverse<K>, T>;

    fn ordering(&mut self, input: &NI, slot: SlotAccess<C, P, I>) -> Option<K> {
        (self.0)(input, slot)
    }
}

impl<NI, C: AnyContainer + ?Sized, P: Permit, I: DynItem + ?Sized> Order<NI, C, P, I>
    for TopologicalKey
{
    type Keys = TopKey;
    type Key = IndexBase;
    /// Min queue
    type Queue<T> = RadixHeapMap<Reverse<IndexBase>, T>;

    fn ordering(&mut self, _: &NI, slot: SlotAccess<C, P, I>) -> Option<Self::Key> {
        Some(slot.key().index().get())
    }
}

pub trait Queue<K, T> {
    /// false if not pushed because key is out of order.
    fn push(&mut self, key: K, item: T) -> bool;

    fn peek(&mut self) -> Option<&T>;

    fn pop(&mut self) -> Option<(K, T)>;
}

pub struct LifoQueue<T> {
    queue: VecDeque<T>,
}

impl<T> Queue<(), T> for LifoQueue<T> {
    fn push(&mut self, _: (), item: T) -> bool {
        self.queue.push_front(item);
        true
    }

    fn peek(&mut self) -> Option<&T> {
        self.queue.front()
    }

    fn pop(&mut self) -> Option<((), T)> {
        self.queue.pop_front().map(|item| ((), item))
    }
}

impl<T> Default for LifoQueue<T> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

pub struct FifoQueue<T> {
    queue: VecDeque<T>,
}

impl<T> Queue<(), T> for FifoQueue<T> {
    fn push(&mut self, _: (), item: T) -> bool {
        self.queue.push_back(item);
        true
    }

    fn peek(&mut self) -> Option<&T> {
        self.queue.front()
    }

    fn pop(&mut self) -> Option<((), T)> {
        self.queue.pop_front().map(|item| ((), item))
    }
}

impl<T> Default for FifoQueue<T> {
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

    fn peek(&mut self) -> Option<&T> {
        RadixHeapMap::peek(self).map(|(_, item)| item)
    }

    fn pop(&mut self) -> Option<(K, T)> {
        self.pop().map(|(key, data)| (key.0, data))
    }
}
