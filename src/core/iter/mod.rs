mod order;
mod start;

use super::{
    permit::access::*,
    permit::{self, Permit},
    AnyContainer, Container, DynItem, Index, IndexBase, Key, Ref, Slot,
};
use order::*;
use radix_heap::Radix;
use start::Start;
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
        }
    }

    pub fn subset<T: DynItem + ?Sized>(
        start: Vec<Key<Ref<'a>, T>>,
    ) -> IterDag<'a, C, start::Subset<'a, T>, permit::Ref> {
        IterDag {
            start: start::Subset(start.into()),
            permit: PhantomData,
            ..Self::new()
        }
    }

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

impl<'a, C: ?Sized, S: Start<'a>> IterDag<'a, C, S, permit::Ref> {
    pub fn mutate(self) -> IterDag<'a, C, S, permit::Mut> {
        IterDag {
            permit: PhantomData,
            ..self
        }
    }
}

impl<'a, C: ?Sized, S: Start<'a>, P: Permit> IterDag<'a, C, S, P> {
    /// Lowest same order inputs are passed to the processor.
    pub fn filter_map<NI: 'a, NP: FnMut(&[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a, NO: 'a>(
        self,
        node_processor: NP,
    ) -> IterDag<'a, C, S, P, NI, NP, NO> {
        IterDag {
            node_input: PhantomData,
            node_processor,
            node_output: PhantomData,
            ..self
        }
    }
}

impl<'a, C: ?Sized, S: Start<'a>, P: Permit, NI, NP, NO> IterDag<'a, C, S, P, NI, NP, NO> {
    pub fn edges_map<
        EP: FnMut(&NO, &mut Slot<'a, P, S::T>) -> EI,
        // TODO: Enable borrowing from Slot for this iterator
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
    >(
        self,
        edge_processor: EP,
    ) -> IterDag<'a, C, S, P, NI, NP, NO, EP, EI> {
        IterDag {
            edge_processor,
            edge_iterator: PhantomData,
            ..self
        }
    }
}

impl<'a, C: ?Sized, S: Start<'a>, P: Permit, NI, NP, NO, EP, EI>
    IterDag<'a, C, S, P, NI, NP, NO, EP, EI>
{
    pub fn depth(self) -> IterDag<'a, C, S, P, NI, NP, NO, EP, EI, order::Depth> {
        IterDag {
            order: order::Depth,
            ..self
        }
    }

    pub fn breadth(self) -> IterDag<'a, C, S, P, NI, NP, NO, EP, EI, order::Breadth> {
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
    ) -> IterDag<'a, C, S, P, NI, NP, NO, EP, EI, order::Topological<O, TO>>
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
    ) -> IterDag<'a, C, S, P, NI, NP, NO, EP, EI, order::TopologicalKey> {
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
        NI,
        NP: FnMut(&[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, C, P, S::T> + 'a,
    > IterDag<'a, C, S, P, NI, NP, NO, EP, EI, O>
{
    pub fn into_iter(
        self,
        access: Access<'a, C, P, impl Permits<S::T> + 'a, All>,
    ) -> impl Iterator<Item = IterNode<'a, P, S::T, NI, NO>> + 'a {
        DAGIterator {
            config: self,
            access: access.keys_split_with(O::Keys::default()),
            queue: O::Queue::default(),
            buffer: VecDeque::new(),
        }
    }

    // TODO: Version that stores NI & NO into graph.
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
    C: AnyContainer + ?Sized,
    S: Start<'a>,
    P: Permit,
    NI,
    NP,
    NO,
    EP,
    EI,
    O: Order<NI, C, P, S::T>,
    TP: Permits<S::T>,
> {
    config: IterDag<'a, C, S, P, NI, NP, NO, EP, EI, O>,
    // Expanded keys are taken from access.
    access: Access<'a, C, P, TP, O::Keys>,
    queue: O::Queue<(NI, Key<Ref<'a>, S::T>)>,
    buffer: VecDeque<IterNode<'a, P, S::T, NI, NO>>,
}

impl<
        'a,
        C: AnyContainer + ?Sized,
        S: Start<'a>,
        P: Permit,
        NI,
        NP: FnMut(&[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, C, P, S::T>,
        TP: Permits<S::T>,
    > DAGIterator<'a, C, S, P, NI, NP, NO, EP, EI, O, TP>
{
    fn process(
        &mut self,
        input: Vec<NI>,
        node: Key<Ref<'a>, S::T>,
    ) -> IterNode<'a, P, S::T, NI, NO> {
        if let Some(mut slot) = self
            .access
            .borrow_key(node)
            .and_then(|access| access.get_try())
        {
            if let Some(output) = (self.config.node_processor)(&input, &mut slot) {
                let mut slot = self
                    .access
                    .take_key(node)
                    .expect("Should be accessable")
                    .get();
                let expansion = (self.config.edge_processor)(&output, &mut slot);

                for (input, key) in expansion {
                    if let Some(order) = self
                        .access
                        .borrow_key(key)
                        .and_then(|access| self.config.order.ordering(&input, access.slot_access()))
                    {
                        self.queue.push(order, (input, key));
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
        C: AnyContainer + ?Sized,
        S: Start<'a>,
        P: Permit,
        NI,
        NP: FnMut(&[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<'a, P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, C, P, S::T>,
        TP: Permits<S::T>,
    > Iterator for DAGIterator<'a, C, S, P, NI, NP, NO, EP, EI, O, TP>
{
    type Item = IterNode<'a, P, S::T, NI, NO>;

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
            return Some(self.process(Vec::new(), start));
        }

        // Process queue
        if let Some((_, (input, next))) = self.queue.pop() {
            // Collect inputs
            let mut inputs = vec![input];
            while let Some(peek) = self.queue.peek() {
                if peek.1 == next {
                    let (_, (input, _)) = self.queue.pop().expect("Should be present");
                    inputs.push(input);
                } else {
                    break;
                }
            }

            return Some(self.process(inputs, next));
        }

        // Done
        None
    }
}

#[allow(dead_code)]
fn example<T>(access: Access<impl Container<()>, permit::Mut, All, All>) {
    for _work_slot in IterDag::rooted(unsafe { Key::<_, ()>::new_ref(Index::new(1).unwrap()) })
        // .mutate()
        .filter_map(|_, _| Some(2))
        .edges_map(|_, _| Vec::<(bool, _)>::new().into_iter())
        // .depth()
        .topological(|_, _| Some(3))
        .into_iter(access.ty().as_ref())
    {}
}
