pub mod isolate;
pub mod iterator;
pub mod order;
pub mod process;
pub mod start;

pub use process::{ProcessDAG, ProcessDAGGrouped};

use super::{
    permit::{self, access::*, Permit},
    AnyContainer, Container, DynItem, Index, IndexBase, Key, Ref, Slot,
};
use isolate::{Isolate, IsolateTemplate, Isolated, Shared};
use iterator::DAGIterator;
use order::*;
use process::DAGProcess;
use radix_heap::Radix;
use start::{Start, Subset};
use std::{borrow::Borrow, collections::VecDeque, marker::PhantomData};

// TODO: Various DAG algorithms on top of this:
// TODO  * A*
// TODO  * Dijkstra
// TODO  * Tarjan (strongly connected components)

/// Iteration of directed acyclic graph
pub struct VisitDAG<
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

impl<'a, C: AnyContainer + ?Sized> VisitDAG<'a, C> {
    fn new() -> Self {
        VisitDAG {
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
    pub fn isolated_space<T: DynItem + ?Sized, Keys: KeyPermit + KeySet + Default>(
        groups: impl IntoIterator<Item = Vec<Key<Ref<'a>, T>>>,
    ) -> VisitDAG<'a, C, start::Subset<'a, T, usize>, permit::Ref, Isolated<T, C, Keys>> {
        VisitDAG {
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
    pub fn subset<T: DynItem + ?Sized>(
        start: Vec<Key<Ref<'a>, T>>,
    ) -> VisitDAG<'a, C, start::Subset<'a, T, ()>, permit::Ref> {
        VisitDAG {
            start: start::Subset(start.into_iter().map(|key| ((), key)).collect()),
            permit: PhantomData,
            ..Self::new()
        }
    }

    /// Star from a single root node
    pub fn rooted<T: DynItem + ?Sized>(
        start: Key<Ref<'a>, T>,
    ) -> VisitDAG<'a, C, start::Root<T>, permit::Ref> {
        VisitDAG {
            start: start::Root(Some(start)),
            permit: PhantomData,
            ..Self::new()
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, S: Start<'a>> VisitDAG<'a, C, S, permit::Ref> {
    /// Enable mutation.
    pub fn mutate(self) -> VisitDAG<'a, C, S, permit::Mut> {
        VisitDAG {
            permit: PhantomData,
            ..self
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, S: Start<'a>, P: Permit> VisitDAG<'a, C, S, P> {
    /// Roots share key space.
    /// That means each node can be expanded only once.
    pub fn shared_space<Keys: KeyPermit + KeySet + Default>(
        self,
    ) -> VisitDAG<'a, C, S, P, Shared<S::T, C, P, Keys>> {
        VisitDAG {
            isolate: PhantomData,
            ..self
        }
    }
}

impl<'a, C: ?Sized, S: Start<'a>, P: Permit, I: IsolateTemplate<S::T, C = C, R = P> + 'a>
    VisitDAG<'a, C, S, P, I, (), (), (), (), ()>
{
    pub fn depth(self) -> VisitDAG<'a, C, S, P, I, (), (), (), (), (), order::Depth> {
        VisitDAG {
            order: order::Depth,
            ..self
        }
    }

    pub fn breadth(self) -> VisitDAG<'a, C, S, P, I, (), (), (), (), (), order::Breadth> {
        VisitDAG {
            order: order::Breadth,
            ..self
        }
    }

    /// Topological sort.
    /// Items with lower order are processed before items with higher order.
    /// Orders less than the order of node are out of order.
    /// Order values [TO] should be unique, else [NI] will get fragmented.
    pub fn topological<
        NI,
        O: FnMut(S::K, &NI, SlotAccess<C, P, S::T>) -> Option<TO>,
        TO: Radix + Ord + Copy,
    >(
        self,
        order: O,
    ) -> VisitDAG<'a, C, S, P, I, (), (), (), (), (), order::Topological<O, TO>>
    where
        C: AnyContainer,
    {
        VisitDAG {
            order: order::Topological(order, PhantomData),
            ..self
        }
    }

    pub fn topological_by_key(
        self,
    ) -> VisitDAG<'a, C, S, P, I, (), (), (), (), (), order::TopologicalKey> {
        VisitDAG {
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
        I: IsolateTemplate<S::T, C = C, R = P> + 'a,
        O,
    > VisitDAG<'a, C, S, P, I, (), (), (), (), (), O>
{
    /// Node processor.
    ///
    /// Filter nodes based on incoming connections and node itself.
    /// Returned value is passed to edge processor.
    ///
    /// Lowest same order inputs are passed to the processor.???
    pub fn node_map<
        NI: 'a,
        NP: FnMut((Option<O::Key>, S::K), &[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO: 'a,
    >(
        self,
        node_processor: NP,
    ) -> VisitDAG<'a, C, S, P, I, NI, NP, NO, (), (), O>
    where
        O: Order<NI, C, P, S::T, S::K, Keys = I::Keys>,
    {
        VisitDAG {
            node_input: PhantomData,
            node_processor,
            node_output: PhantomData,
            ..self
        }
    }
}

impl<
        'a,
        C: AnyContainer + ?Sized,
        S: Start<'a>,
        P: Permit,
        I: IsolateTemplate<S::T, C = C, R = P> + 'a,
        NI,
        NP,
        NO,
        O: Order<NI, C, P, S::T, I::Group, Keys = I::Keys>,
    > VisitDAG<'a, C, S, P, I, NI, NP, NO, (), (), O>
{
    pub fn edges_map<
        EP: FnMut(&NO, &mut Slot<P, S::T>) -> EI,
        // TODO: Enable borrowing from Slot for this iterator
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
    >(
        self,
        edge_processor: EP,
    ) -> VisitDAG<'a, C, S, P, I, NI, NP, NO, EP, EI, O> {
        VisitDAG {
            edge_processor,
            edge_iterator: PhantomData,
            ..self
        }
    }
}

impl<
        'a,
        C: AnyContainer + ?Sized,
        S: Start<'a>,
        P: Permit,
        I: IsolateTemplate<S::T, C = C, R = P, Group = S::K>,
        NI,
        NP: FnMut((Option<O::Key>, S::K), &[NI], &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, C, P, S::T, S::K, Keys = I::Keys> + 'a,
    > VisitDAG<'a, C, S, P, I, NI, NP, NO, EP, EI, O>
{
    pub fn into_iter<'b, TP: 'b + TypePermit + Permits<S::T>>(
        self,
        access: Access<'b, C, P, TP, All>,
    ) -> DAGIterator<'a, 'b, C, S, P, NI, NP, NO, EP, EI, O, I, I::B<'b, TP>, TP>
    where
        'a: 'b,
    {
        DAGIterator::new(
            I::new(access, &self.start),
            DAGCore {
                config: self,
                queue: O::Queue::default(),
                buffer: VecDeque::new(),
            },
        )
    }

    pub fn into_process(self) -> DAGProcess<'a, C, S, P, NI, NP, NO, EP, EI, O, I> {
        DAGProcess::new(
            I::paused(&self.start),
            DAGCore {
                config: self,
                queue: O::Queue::default(),
                buffer: VecDeque::new(),
            },
        )
    }
}

pub struct DAGCore<
    'a,
    C: AnyContainer + ?Sized,
    S: Start<'a>,
    P: Permit,
    NI,
    NP,
    NO,
    EP,
    EI,
    O: Order<NI, C, P, S::T, S::K>,
    I: IsolateTemplate<S::T>,
> {
    config: VisitDAG<'a, C, S, P, I, NI, NP, NO, EP, EI, O>,
    queue: O::Queue<(NI, Key<Ref<'a>, S::T>)>,
    buffer: VecDeque<(S::K, NI, Key<Ref<'a>, S::T>)>,
}

impl<
        'a,
        T: DynItem + ?Sized,
        NI,
        NP,
        NO,
        EP,
        EI,
        O: Order<NI, I::C, I::R, T, I::Group>,
        I: IsolateTemplate<T, Group = ()>,
    > DAGCore<'a, I::C, Subset<'a, T, ()>, I::R, NI, NP, NO, EP, EI, O, I>
{
    /// Group must exist
    pub fn add_root(&mut self, root: Key<Ref<'a>, T>) {
        self.config.start.0.push_back(((), root));
    }
}

impl<
        'a,
        T: DynItem + ?Sized,
        NI,
        NP,
        NO,
        EP,
        EI,
        O: Order<NI, I::C, I::R, T, I::Group>,
        I: IsolateTemplate<T, Group = usize>,
    > DAGCore<'a, I::C, Subset<'a, T, usize>, I::R, NI, NP, NO, EP, EI, O, I>
{
    /// Adds group of roots and returns their index.
    fn add_group(&mut self, index: usize, group: impl IntoIterator<Item = Key<Ref<'a>, T>>) {
        self.config
            .start
            .0
            .extend(group.into_iter().map(|key| (index, key)));
    }

    /// Group must exist
    pub fn add_to_group(&mut self, group: usize, root: Key<Ref<'a>, T>) {
        self.config.start.0.push_back((group, root));
    }

    /// Removes group.
    /// It's index may be reused.
    fn purge(&mut self, group: usize) {
        self.config.start.0.retain(|(g, _)| g != &group);
        self.buffer.retain(|(g, _, _)| g != &group);
        for (key, value) in std::mem::replace(&mut self.queue, O::Queue::default()).into_iter() {
            if key.1 != group {
                self.queue.push(key, value);
            }
        }
    }
}

impl<
        'a,
        S: Start<'a>,
        NI,
        NP: FnMut((Option<O::Key>, I::Group), &[NI], &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<I::R, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, I::C, I::R, S::T, S::K>,
        I: IsolateTemplate<S::T, Group = S::K>,
    > DAGCore<'a, I::C, S, I::R, NI, NP, NO, EP, EI, O, I>
{
    /// Sets order for further processing.
    pub fn set_order(&mut self, order: O) {
        self.config.order = order;
    }

    /// Reorders queue according to new order.
    fn reorder<'b, TP: 'b + TypePermit + Permits<S::T>>(
        &mut self,
        order: O,
        access: &mut I::B<'b, TP>,
    ) where
        'a: 'b,
    {
        self.config.order = order;
        self.recompute_order(access);
    }

    fn recompute_order<'b, TP: 'b + TypePermit + Permits<S::T>>(
        &mut self,
        access: &mut I::B<'b, TP>,
    ) where
        'a: 'b,
    {
        for ((_, group), (input, key)) in
            std::mem::replace(&mut self.queue, O::Queue::default()).into_iter()
        {
            if let Some(access) = access.borrow_key(group, key) {
                if let Some(order) = self
                    .config
                    .order
                    .ordering(group, &input, access.slot_access())
                {
                    self.queue.push((order, group), (input, key));
                }
            } else {
                self.buffer.push_back((group, input, key));
            }
        }
    }

    fn process<'b, TP: 'b + TypePermit + Permits<S::T>, Keys: KeyPermit + KeySet>(
        &mut self,
        order: Option<O::Key>,
        group: I::Group,
        input: Vec<NI>,
        node: Key<Ref<'a>, S::T>,
        access: &mut Access<'b, I::C, I::R, TP, Keys>,
    ) -> IterNode<'a, 'b, I::R, S::T, NI, NO>
    where
        'a: 'b,
    {
        if let Some(mut slot) = access
            .borrow_key(node)
            .and_then(|access| access.fetch_try())
        {
            if let Some(output) = (self.config.node_processor)((order, group), &input, &mut slot) {
                let mut slot = access.take_key(node).expect("Should be accessable").fetch();
                let expansion = (self.config.edge_processor)(&output, &mut slot);

                for (input, key) in expansion {
                    if let Some(access) = access.borrow_key(key) {
                        if let Some(order) =
                            self.config
                                .order
                                .ordering(group, &input, access.slot_access())
                        {
                            self.queue.push((order, group), (input, key));
                        }
                    } else {
                        self.buffer.push_back((group, input, key));
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

    fn process_next<'b>(
        &mut self,
    ) -> Option<
        Result<
            IterNode<'a, 'b, I::R, S::T, NI, NO>,
            (Option<O::Key>, I::Group, Vec<NI>, Key<Ref<'a>, S::T>),
        >,
    >
    where
        'a: 'b,
    {
        // NOTES:
        // - Only ever use get_try for getting slots to cover cases when a sub container is passed here or constrictive KeyPermit
        // - More generally this should be robust against malformed user input

        // Process buffered
        if let Some((_, input, key)) = self.buffer.pop_front() {
            return Some(Ok(IterNode::OutOfOrder(vec![input], key)));
        }

        // Process starting nodes
        if let Some((group, start)) = self.config.start.pop() {
            return Some(Err((None, group, Vec::new(), start)));
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

            return Some(Err((Some(key.0), key.1, inputs, next)));
        }

        // Done
        None
    }
}

pub enum IterNode<'a: 'b, 'b, P: Permit, T: DynItem + ?Sized, IN, OUT> {
    /// Was not expanded.
    Idle(Key<Ref<'a>, T>),

    Expanded(Slot<'b, P, T>, OUT),

    /// This node was called out of order.
    OutOfOrder(Vec<IN>, Key<Ref<'a>, T>),
    // OutOfCollection
}

#[allow(dead_code)]
fn example<T>(mut access: Access<impl Container<()>, permit::Mut, All, All>) {
    // Pause-resume
    let iter = VisitDAG::isolated_space(vec![vec![unsafe {
        Key::<_, ()>::new_ref(Index::new(1).unwrap())
    }]])
    .topological(|_, _, _| Some(3))
    .node_map(|_, _, _| Some(2))
    .edges_map(|_, _| Vec::<(bool, _)>::new().into_iter())
    .into_iter(access.as_ref().ty());
    let paused_work = iter.pause();

    // Other access
    for _work_slot in VisitDAG::rooted(unsafe { Key::<_, ()>::new_ref(Index::new(1).unwrap()) })
        .mutate()
        .shared_space()
        .topological(|_, _, _| Some(3))
        .node_map(|_, _, _| Some(2))
        .edges_map(|_, _| Vec::<(bool, _)>::new().into_iter())
        .into_iter(access.borrow_mut().ty())
    {}

    let mut process = VisitDAG::isolated_space(vec![vec![unsafe {
        Key::<_, ()>::new_ref(Index::new(1).unwrap())
    }]])
    .depth()
    .node_map(|_, _, _| Some(2))
    .edges_map(|_, _| Vec::<(bool, _)>::new().into_iter())
    .into_process();

    for _work_slot in paused_work.resume(access.as_ref().ty()) {}

    let _ = process.step(access.as_ref().ty());

    let mut process = Box::new(process) as Box<dyn ProcessDAG<_, _, _, _, _, _>>;

    let _ = process.step(access.as_ref().ty());
}
