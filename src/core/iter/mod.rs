mod isolate;
mod order;
mod start;

use super::{
    permit::access::*,
    permit::{self, Permit},
    AnyContainer, Container, DynItem, Index, IndexBase, Key, Ref, Slot,
};
use isolate::{Isolate, RefIsolate};
use order::*;
use radix_heap::Radix;
use start::{Start, Subset};
use std::{collections::VecDeque, marker::PhantomData};

// TODO: Various DAG algorithms on top of this:
// TODO  * A*
// TODO  * Dijkstra
// TODO  * Tarjan (strongly connected components)

/// Iteration of directed acyclic graph
pub struct IterDag<
    'a,
    C: ?Sized,
    S = (),
    P = (),
    I = (),
    NI: 'a = (),
    NP: 'a = (),
    NO: 'a = (),
    EP: 'a = (),
    EI: 'a = (),
    O = (),
> {
    _container: PhantomData<&'a C>,
    start: S,
    permit: PhantomData<P>,
    isolate: PhantomData<I>,
    node_input: PhantomData<&'a NI>,
    node_processor: NP,
    node_output: PhantomData<&'a NO>,
    edge_processor: EP,
    edge_iterator: PhantomData<&'a EI>,
    order: O,
}

impl<'a, C: AnyContainer + ?Sized> IterDag<'a, C> {
    fn new() -> Self {
        IterDag {
            _container: PhantomData,
            start: (),
            permit: PhantomData,
            node_input: PhantomData,
            node_processor: (),
            node_output: PhantomData,
            edge_processor: (),
            edge_iterator: PhantomData,
            order: (),
            isolate: PhantomData,
        }
    }

    /// Star from multiple root nodes where each of them belongs to a group.
    /// Groups have isolated key space.
    /// That means each node can be expanded multiple times, once by each group.
    pub fn isolated<
        T: DynItem + ?Sized,
        TP: TypePermit + Permits<T>,
        Keys: KeyPermit + KeySet + Default,
    >(
        groups: impl IntoIterator<Item = Vec<Key<Ref<'a>, T>>>,
    ) -> IterDag<'a, C, start::Subset<'a, T, usize>, permit::Ref, RefIsolate<'a, C, TP, Keys>> {
        IterDag {
            start: start::Subset(
                groups
                    .into_iter()
                    .enumerate()
                    .flat_map(|(i, group)| group.into_iter().map(move |key| (i, key)))
                    .collect(),
            ),
            permit: PhantomData,
            isolate: PhantomData,
            ..Self::new()
        }
    }

    /// Star from multiple root nodes
    pub fn subset<
        T: DynItem + ?Sized,
        TP: TypePermit + Permits<T>,
        Keys: KeyPermit + KeySet + Default,
    >(
        start: Vec<Key<Ref<'a>, T>>,
    ) -> IterDag<'a, C, start::Subset<'a, T, ()>, permit::Ref, Access<'a, C, permit::Ref, TP, Keys>>
    {
        IterDag {
            start: start::Subset(start.into_iter().map(|key| ((), key)).collect()),
            permit: PhantomData,
            isolate: PhantomData,
            ..Self::new()
        }
    }

    /// Star from a single root node
    pub fn rooted<
        T: DynItem + ?Sized,
        TP: TypePermit + Permits<T>,
        Keys: KeyPermit + KeySet + Default,
    >(
        start: Key<Ref<'a>, T>,
    ) -> IterDag<'a, C, start::Root<T>, permit::Ref, Access<'a, C, permit::Ref, TP, Keys>> {
        IterDag {
            start: start::Root(Some(start)),
            permit: PhantomData,
            isolate: PhantomData,
            ..Self::new()
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, S: Start<'a>> IterDag<'a, C, S, permit::Ref> {
    /// Enable mutation.
    /// Also implies shared_space.
    pub fn mutate(self) -> IterDag<'a, C, S, permit::Mut> {
        IterDag {
            permit: PhantomData,
            ..self
        }
    }
}

impl<'a, C: ?Sized, S: Start<'a>, P: Permit, I: Isolate<'a, S::T, C = C, R = P> + 'a>
    IterDag<'a, C, S, P, I>
{
    /// Node processor.
    ///
    /// Filter nodes based on incoming connections and node itself.
    /// Returned value is passed to edge processor.
    ///
    /// Lowest same order inputs are passed to the processor.???
    pub fn node_map<NI: 'a, NP: FnMut(&[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a, NO: 'a>(
        self,
        node_processor: NP,
    ) -> IterDag<'a, C, S, P, I, NI, NP, NO> {
        IterDag {
            node_input: PhantomData,
            node_processor,
            node_output: PhantomData,
            ..self
        }
    }
}

impl<
        'a,
        C: ?Sized,
        S: Start<'a>,
        P: Permit,
        I: Isolate<'a, S::T, C = C, R = P> + 'a,
        NI,
        NP,
        NO,
    > IterDag<'a, C, S, P, I, NI, NP, NO>
{
    pub fn edges_map<
        EP: FnMut(&NO, &mut Slot<'a, P, S::T>) -> EI,
        // TODO: Enable borrowing from Slot for this iterator
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
    >(
        self,
        edge_processor: EP,
    ) -> IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI> {
        IterDag {
            edge_processor,
            edge_iterator: PhantomData,
            ..self
        }
    }
}

impl<
        'a,
        C: ?Sized,
        S: Start<'a>,
        P: Permit,
        I: Isolate<'a, S::T, C = C, R = P> + 'a,
        NI,
        NP,
        NO,
        EP,
        EI,
    > IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI>
{
    pub fn depth(self) -> IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI, order::Depth> {
        IterDag {
            order: order::Depth,
            ..self
        }
    }

    pub fn breadth(self) -> IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI, order::Breadth> {
        IterDag {
            order: order::Breadth,
            ..self
        }
    }

    /// Topological sort.
    /// Items with lower order are processed before items with higher order.
    /// Orders less than the order of node are out of order.
    /// Order values [TO] should be unique, else [NI] will get fragmented.
    pub fn topological<
        O: FnMut(&NI, SlotAccess<C, P, S::T>) -> Option<TO>,
        TO: Radix + Ord + Copy,
    >(
        self,
        order: O,
    ) -> IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI, order::Topological<O, TO>>
    where
        C: AnyContainer,
    {
        IterDag {
            order: order::Topological(order, PhantomData),
            ..self
        }
    }

    pub fn topological_by_key(
        self,
    ) -> IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI, order::TopologicalKey> {
        IterDag {
            order: order::TopologicalKey,
            ..self
        }
    }
}

impl<
        'a,
        C: AnyContainer + ?Sized,
        S: Start<'a>,
        P: Permit,
        I: Isolate<'a, S::T, C = C, R = P, Group = S::K> + 'a,
        NI,
        NP: FnMut(&[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, C, P, S::T, I::Group, Keys = I::Keys> + 'a,
    > IterDag<'a, C, S, P, I, NI, NP, NO, EP, EI, O>
{
    pub fn into_iter(
        self,
        access: Access<'a, C, P, I::TP, All>,
    ) -> impl Iterator<Item = IterNode<'a, P, S::T, NI, NO>> + 'a {
        DAGIterator {
            access: I::new(access, &self.start),
            config: self,
            queue: O::Queue::default(),
            buffer: VecDeque::new(),
        }
    }
}

pub enum IterNode<'a, P: Permit, T: DynItem + ?Sized, IN, OUT> {
    /// Was not expanded.
    Idle(Key<Ref<'a>, T>),

    Expanded(Slot<'a, P, T>, OUT),

    /// This node was called out of order.
    OutOfOrder(Vec<IN>, Key<Ref<'a>, T>),
    // OutOfCollection
}

pub struct DAGIterator<
    'a,
    S: Start<'a>,
    NI,
    NP,
    NO,
    EP,
    EI,
    O: Order<NI, I::C, I::R, S::T, I::Group>,
    I: Isolate<'a, S::T, Group = S::K>,
> {
    config: IterDag<'a, I::C, S, I::R, I, NI, NP, NO, EP, EI, O>,
    access: I,
    queue: O::Queue<(NI, Key<Ref<'a>, S::T>)>,
    buffer: VecDeque<IterNode<'a, I::R, S::T, NI, NO>>,
}

impl<
        'a,
        T: DynItem + ?Sized,
        NI,
        NP: FnMut(&[NI], &mut Slot<I::R, T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, I::R, T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, T>)>,
        O: Order<NI, I::C, I::R, T, I::Group>,
        I: Isolate<'a, T, Group = usize>,
    > DAGIterator<'a, Subset<'a, T, usize>, NI, NP, NO, EP, EI, O, I>
{
    /// Adds group of roots and returns their index.
    pub fn add_group(&mut self, group: impl IntoIterator<Item = Key<Ref<'a>, T>>) -> usize {
        let i = self.access.add_group();
        self.config
            .start
            .0
            .extend(group.into_iter().map(|key| (i, key)));
        i
    }

    /// Group must exist
    pub fn add_to_group(&mut self, group: usize, root: Key<Ref<'a>, T>) {
        self.config.start.0.push_back((group, root));
    }
}

impl<
        'a,
        T: DynItem + ?Sized,
        NI,
        NP: FnMut(&[NI], &mut Slot<I::R, T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, I::R, T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, T>)>,
        O: Order<NI, I::C, I::R, T, I::Group>,
        I: Isolate<'a, T, Group = ()>,
    > DAGIterator<'a, Subset<'a, T, ()>, NI, NP, NO, EP, EI, O, I>
{
    /// Group must exist
    pub fn add_root(&mut self, root: Key<Ref<'a>, T>) {
        self.config.start.0.push_back(((), root));
    }
}

impl<
        'a,
        S: Start<'a>,
        NI,
        NP: FnMut(&[NI], &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, I::R, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, I::C, I::R, S::T, I::Group>,
        I: Isolate<'a, S::T, Group = S::K>,
    > DAGIterator<'a, S, NI, NP, NO, EP, EI, O, I>
{
    /// Reorders queue according to new order.
    pub fn reorder(&mut self, order: O) {
        self.config.order = order;
        for ((_, group), (input, key)) in
            std::mem::replace(&mut self.queue, O::Queue::default()).into_iter()
        {
            if let Some(order) = self
                .access
                .borrow_key(group, key)
                .and_then(|access| self.config.order.ordering(&input, access.slot_access()))
            {
                self.queue.push((order, group), (input, key));
            } else {
                self.buffer
                    .push_back(IterNode::OutOfOrder(vec![input], key));
            }
        }
    }

    fn process(
        &mut self,
        group: I::Group,
        input: Vec<NI>,
        node: Key<Ref<'a>, S::T>,
    ) -> IterNode<'a, I::R, S::T, NI, NO> {
        if let Some(mut slot) = self
            .access
            .borrow_key(group, node)
            .and_then(|access| access.fetch_try())
        {
            if let Some(output) = (self.config.node_processor)(&input, &mut slot) {
                let mut slot = self
                    .access
                    .take_key(group, node)
                    .expect("Should be accessable")
                    .fetch();
                let expansion = (self.config.edge_processor)(&output, &mut slot);

                for (input, key) in expansion {
                    if let Some(order) = self
                        .access
                        .borrow_key(group, key)
                        .and_then(|access| self.config.order.ordering(&input, access.slot_access()))
                    {
                        self.queue.push((order, group), (input, key));
                    } else {
                        self.buffer
                            .push_back(IterNode::OutOfOrder(vec![input], key));
                    }
                }

                IterNode::Expanded(slot, output)
            } else {
                IterNode::Idle(node)
            }
        } else {
            IterNode::OutOfOrder(input, node)
        }
    }
}

impl<
        'a,
        S: Start<'a>,
        NI,
        NP: FnMut(&[NI], &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, I::R, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, I::C, I::R, S::T, I::Group>,
        I: Isolate<'a, S::T, Group = S::K>,
    > Iterator for DAGIterator<'a, S, NI, NP, NO, EP, EI, O, I>
{
    type Item = IterNode<'a, I::R, S::T, NI, NO>;

    fn next(&mut self) -> Option<Self::Item> {
        // NOTES:
        // - Only ever use get_try for getting slots to cover cases when a sub container is passed here or constrictive KeyPermit
        // - More generally this should be robust against malformed user input

        // Process buffered
        if let Some(node) = self.buffer.pop_front() {
            return Some(node);
        }

        // Process starting nodes
        if let Some((group, start)) = self.config.start.pop() {
            return Some(self.process(group, Vec::new(), start));
        }

        // Process queue
        if let Some((key, (input, next))) = self.queue.pop() {
            // Collect inputs
            let mut inputs = vec![input];
            while let Some((peek_key, peek)) = self.queue.peek() {
                if peek_key == &key && peek.1 == next {
                    let (_, (input, _)) = self.queue.pop().expect("Should be present");
                    inputs.push(input);
                } else {
                    break;
                }
            }

            return Some(self.process(key.1, inputs, next));
        }

        // Done
        None
    }
}

#[allow(dead_code)]
fn example<T>(access: Access<impl Container<()>, permit::Mut, All, All>) {
    for _work_slot in IterDag::rooted(unsafe { Key::<_, ()>::new_ref(Index::new(1).unwrap()) })
        // .mutate()
        .node_map(|_, _| Some(2))
        .edges_map(|_, _| Vec::<(bool, _)>::new().into_iter())
        // .depth()
        .topological(|_, _| Some(3))
        .into_iter(access.ty().as_ref())
    {}
}
