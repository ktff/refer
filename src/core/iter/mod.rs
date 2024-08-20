mod isolate;
mod order;
mod start;

use super::{
    permit::access::*,
    permit::{self, Permit},
    AnyContainer, Container, DynItem, Index, IndexBase, Key, Ref, Slot,
};
use isolate::Isolate;
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

    node_input: PhantomData<&'a NI>,
    node_processor: NP,
    node_output: PhantomData<&'a NO>,
    edge_processor: EP,
    edge_iterator: PhantomData<&'a EI>,
    order: O,
    isolate: PhantomData<I>,
}

impl<'a, C: ?Sized> IterDag<'a, C> {
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

    /// Star from multiple root nodes
    pub fn subset<T: DynItem + ?Sized>(
        start: Vec<Key<Ref<'a>, T>>,
    ) -> IterDag<'a, C, start::Subset<'a, T>, permit::Ref> {
        IterDag {
            start: start::Subset(start.into()),
            permit: PhantomData,
            ..Self::new()
        }
    }

    /// Star from a single root node
    pub fn rooted<T: DynItem + ?Sized>(
        start: Key<Ref<'a>, T>,
    ) -> IterDag<'a, C, start::Root<T>, permit::Ref> {
        IterDag {
            start: start::Root(Some(start)),
            permit: PhantomData,
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

    /// Roots have isolated key space.
    /// That means each node can be expanded multiple times, once by each root.
    pub fn isolated_space<TP: TypePermit + Permits<S::T>, Keys: KeyPermit + KeySet + Default>(
        self,
    ) -> IterDag<'a, C, S, permit::Ref, Access<'a, C, permit::Ref, TP, Keys>> {
        IterDag {
            isolate: PhantomData,
            ..self
        }
    }

    /// All roots share same access key space.
    /// That means each node can only be expanded once.
    pub fn shared_space<TP: TypePermit + Permits<S::T>, Keys: KeyPermit + KeySet + Default>(
        self,
    ) -> IterDag<'a, C, S, permit::Ref, Access<'a, C, permit::Ref, TP, Keys>> {
        IterDag {
            isolate: PhantomData,
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
        I: Isolate<'a, S::T, C = C, R = P> + 'a,
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
            config: self,
            access: I::new(access),
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
    I: Isolate<'a, S::T>,
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
        I: Isolate<'a, T>,
    > DAGIterator<'a, Subset<'a, T>, NI, NP, NO, EP, EI, O, I>
{
    /// Adds additional start to the back of start list.
    pub fn push_start(&mut self, start: Key<Ref<'a>, T>) {
        self.config.start.0.push_back(start);
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
        I: Isolate<'a, S::T>,
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
        I: Isolate<'a, S::T>,
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
        if let Some(start) = self.config.start.pop() {
            let group = self.access.add_root(start);
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
        .isolated_space()
        // .shared_space()
        .node_map(|_, _| Some(2))
        .edges_map(|_, _| Vec::<(bool, _)>::new().into_iter())
        // .depth()
        .topological(|_, _| Some(3))
        .into_iter(access.ty().as_ref())
    {}
}
