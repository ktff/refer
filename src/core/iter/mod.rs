pub mod isolate;
pub mod iterator;
pub mod order;
pub mod process;
pub mod start;

pub use ordered_float::NotNan;
pub use process::{ProcessDAG, ProcessIsolatedDAG};

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
use std::{collections::VecDeque, marker::PhantomData};

// TODO: Various DAG algorithms on top of this:
// TODO  * A*
// TODO  * Dijkstra
// TODO  * Tarjan (strongly connected components)

pub type GroupId = u32;

/// Iteration of directed acyclic graph
pub struct VisitDAG<
    'a,
    C: ?Sized = (),
    S = (),
    P = (),
    I = (),
    NI: 'a = (),
    NP: 'a = (),
    NO: 'a = (),
    EP: 'a = (),
    EI: 'a = (),
    O = (),
    TP = (),
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
    type_permit: PhantomData<TP>,
}

impl<'a> VisitDAG<'a> {
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
            type_permit: PhantomData,
        }
    }

    /// Star from multiple root nodes where each of them belongs to a group.
    /// Groups have isolated key space.
    /// That means each node can be expanded multiple times, once by each group.
    pub fn isolated_space<
        C: AnyContainer + ?Sized,
        T: DynItem + ?Sized,
        Keys: KeyPermit + KeySet + Default,
    >(
        groups: impl IntoIterator<Item = Vec<Key<Ref<'a>, T>>>,
    ) -> VisitDAG<'a, C, start::Subset<'a, T, GroupId>, permit::Ref, Isolated<T, C, Keys>> {
        VisitDAG {
            start: start::Subset(
                groups
                    .into_iter()
                    .enumerate()
                    .flat_map(|(i, group)| group.into_iter().map(move |key| (i as GroupId, key)))
                    .collect(),
            ),
            permit: PhantomData,
            isolate: PhantomData,
            _container: PhantomData,
            ..Self::new()
        }
    }

    /// Star from multiple root nodes
    pub fn subset<C: AnyContainer + ?Sized, T: DynItem + ?Sized>(
        start: Vec<Key<Ref<'a>, T>>,
    ) -> VisitDAG<'a, C, start::Subset<'a, T, ()>, permit::Ref> {
        VisitDAG {
            start: start::Subset(start.into_iter().map(|key| ((), key)).collect()),
            permit: PhantomData,
            _container: PhantomData,
            ..Self::new()
        }
    }

    /// Start from a single root node
    pub fn rooted<C: AnyContainer + ?Sized, T: DynItem + ?Sized>(
        start: Key<Ref<'a>, T>,
    ) -> VisitDAG<'a, C, start::Root<'a, T>, permit::Ref> {
        VisitDAG {
            start: start::Root(Some(start)),
            permit: PhantomData,
            _container: PhantomData,
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

    /// Breath first but they are topologically sorted by key.
    /// [NI] will be grouped.
    pub fn forward(self) -> VisitDAG<'a, C, S, P, I, (), (), (), (), (), order::Forward> {
        VisitDAG {
            order: order::Forward,
            ..self
        }
    }

    /// Topological sort.
    /// Items with lower order are processed before items with higher order.
    /// Order values [TO] should be unique, else [NI] will get fragmented.
    pub fn topological<
        NI,
        O: FnMut(S::K, &mut NI, SlotAccess<C, P, S::T>) -> Option<TO>,
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
    pub fn reach<
        NI: 'a,
        NP: FnMut((Option<O::Key>, S::K), Vec<NI>, &mut Slot<P, S::T>) -> Option<NO> + 'a,
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
    pub fn expand<
        EP: FnMut(
            &NO,
            &mut Slot<P, S::T>,
            &mut dyn FnMut(NI, Key<Ref<'_>, S::T>),
            &Access<'_, C, P, TP, I::Keys>,
        ),
        TP: 'a + TypePermit + Permits<S::T>,
    >(
        self,
        edge_processor: EP,
    ) -> VisitDAG<'a, C, S, P, I, NI, NP, NO, EP, (), O, TP> {
        VisitDAG {
            edge_processor,
            edge_iterator: PhantomData,
            type_permit: PhantomData,
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
        NP: FnMut((Option<O::Key>, S::K), Vec<NI>, &mut Slot<P, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(
            &NO,
            &mut Slot<P, S::T>,
            &mut dyn FnMut(NI, Key<Ref<'_>, S::T>),
            &Access<'_, C, P, S::T, I::Keys>,
        ),
        O: Order<NI, C, P, S::T, S::K, Keys = I::Keys> + 'a,
        TP: 'a + TypePermit + Permits<S::T>,
    > VisitDAG<'a, C, S, P, I, NI, NP, NO, EP, (), O, TP>
{
    pub fn into_iter<'b>(
        self,
        access: Access<'b, C, P, TP, All>,
    ) -> DAGIterator<'a, 'b, C, S, P, NI, NP, NO, EP, (), O, I, I::B<'b, TP>, TP>
    where
        'a: 'b,
    {
        DAGIterator::new(
            I::new(access, &self.start),
            DAGCore {
                config: self,
                queue: O::Queue::default(),
                buffer: VecDeque::new(),
                internal: None,
            },
        )
    }

    pub fn into_process(self) -> DAGProcess<'a, C, S, P, NI, NP, NO, EP, (), O, I, TP> {
        DAGProcess::new(
            I::paused(&self.start),
            DAGCore {
                config: self,
                queue: O::Queue::default(),
                buffer: VecDeque::new(),
                internal: None,
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
    TP,
> {
    config: VisitDAG<'a, C, S, P, I, NI, NP, NO, EP, EI, O, TP>,
    queue: O::Queue<(NI, Key<Ref<'a>, S::T>)>,
    buffer: VecDeque<(S::K, NI, Key<Ref<'a>, S::T>)>,
    internal: Option<InternalEvent>,
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
        TP,
    > DAGCore<'a, I::C, Subset<'a, T, ()>, I::R, NI, NP, NO, EP, EI, O, I, TP>
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
        I: IsolateTemplate<T, Group = GroupId>,
        TP,
    > DAGCore<'a, I::C, Subset<'a, T, GroupId>, I::R, NI, NP, NO, EP, EI, O, I, TP>
{
    /// Adds group of roots and returns their index.
    fn add_group(&mut self, index: GroupId, group: impl IntoIterator<Item = Key<Ref<'a>, T>>) {
        self.config
            .start
            .0
            .extend(group.into_iter().map(|key| (index, key)));
    }

    /// Group must exist
    pub fn add_root_to_group(&mut self, group: GroupId, root: Key<Ref<'a>, T>) {
        self.config.start.0.push_back((group, root));
    }

    pub fn add_input_to_group(
        &mut self,
        group: GroupId,
        order: O::Key,
        input: NI,
        to: Key<Ref<'a>, T>,
    ) {
        self.queue.push((order, group), (input, to));
    }

    /// Removes group.
    /// It's index may be reused.
    fn purge(&mut self, group: GroupId) {
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
        NP: FnMut((Option<O::Key>, I::Group), Vec<NI>, &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(
            &NO,
            &mut Slot<I::R, S::T>,
            &mut dyn FnMut(NI, Key<Ref<'_>, S::T>),
            &Access<'_, I::C, I::R, TP, I::Keys>,
        ),
        O: Order<NI, I::C, I::R, S::T, S::K>,
        I: IsolateTemplate<S::T, Group = S::K>,
        TP: 'a + TypePermit + Permits<S::T>,
    > DAGCore<'a, I::C, S, I::R, NI, NP, NO, EP, (), O, I, TP>
{
    /// Sets order for further processing.
    pub fn set_order(&mut self, order: O) {
        self.config.order = order;
    }

    /// Reorders queue according to new order.
    fn reorder<'b>(&mut self, order: O, access: &mut I::B<'b, TP>)
    where
        'a: 'b,
    {
        self.config.order = order;
        self.recompute_order(access);
    }

    fn recompute_order<'b>(&mut self, access: &mut I::B<'b, TP>)
    where
        'a: 'b,
    {
        for ((_, group), (mut input, key)) in
            std::mem::replace(&mut self.queue, O::Queue::default()).into_iter()
        {
            if let Some(access) = access.borrow_key(group, key) {
                if let Some(order) =
                    self.config
                        .order
                        .ordering(group, &mut input, access.slot_access())
                {
                    self.queue.push((order, group), (input, key));
                }
            } else {
                self.buffer.push_back((group, input, key));
            }
        }
    }

    fn process<'b>(
        &mut self,
        order: Option<O::Key>,
        group: I::Group,
        input: Vec<NI>,
        node: Key<Ref<'a>, S::T>,
        access: &mut Access<'b, I::C, I::R, TP, I::Keys>,
    ) -> IterNode<'a, 'b, I::R, S::T, NI, NO>
    where
        'a: 'b,
    {
        if let Some(mut slot) = access
            .borrow_key(node)
            .and_then(|access| access.fetch_try())
        {
            if let Some(output) = (self.config.node_processor)((order, group), input, &mut slot) {
                let mut slot = access.take_key(node).expect("Should be accessable").fetch();
                (self.config.edge_processor)(
                    &output,
                    &mut slot,
                    &mut |mut input, key| {
                        let key = key.promise().fulfill(node);
                        if let Some(access) = access.borrow_key(key) {
                            if let Some(order) =
                                self.config
                                    .order
                                    .ordering(group, &mut input, access.slot_access())
                            {
                                self.queue.push((order, group), (input, key));
                            }
                        } else {
                            self.buffer.push_back((group, input, key));
                        }
                    },
                    access,
                );

                IterNode::Expanded(slot, output)
            } else {
                IterNode::NotExpanded(node)
            }
        } else {
            IterNode::AlreadyExpanded(input, node)
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

        // Internal
        if let Some(internal) = self.internal.take() {
            return Some(Ok(IterNode::Internal(internal)));
        }

        // Process buffered
        if let Some((_, input, key)) = self.buffer.pop_front() {
            return Some(Ok(IterNode::AlreadyExpanded(vec![input], key)));
        }

        // Process starting nodes
        if let Some((group, start)) = self.config.start.pop() {
            if self.config.start.is_empty() {
                self.internal = Some(InternalEvent::StartProcessed);
            }
            return Some(Err((None, group, Vec::new(), start)));
        }

        // Process queue
        if let Some((key, (input, next))) = self.queue.pop() {
            // Collect inputs
            let mut inputs = vec![input];

            loop {
                if let Some((peek_key, peek)) = self.queue.peek() {
                    if peek_key == &key && peek.1 == next {
                        let (_, (input, _)) = self.queue.pop().expect("Should be present");
                        inputs.push(input);
                        continue;
                    }
                } else {
                    self.internal = Some(InternalEvent::LevelProcessed);
                }
                break;
            }

            return Some(Err((Some(key.0), key.1, inputs, next)));
        }

        // Done
        None
    }
}

pub enum IterNode<'a: 'b, 'b, P: Permit, T: DynItem + ?Sized, IN, OUT> {
    /// Was not expanded.
    NotExpanded(Key<Ref<'a>, T>),

    Expanded(Slot<'b, P, T>, OUT),

    /// This node was called already expanded.
    AlreadyExpanded(Vec<IN>, Key<Ref<'a>, T>),

    /// A notification of some internal event.
    /// Can be safely ignored.
    Internal(InternalEvent),
}

/// Event that happened in internals
pub enum InternalEvent {
    /// All starts have been processed and next call will start processing
    /// expanded, if any.
    StartProcessed,
    /// A level has been processed.
    /// For breath ordering, level corresponds to depth and next call will
    /// start processing expanded from previous level.
    /// For other orderings, level corresponds to end of processing and next call
    /// will finish.
    LevelProcessed,
}

#[allow(dead_code)]
fn example<T>(mut access: Access<impl Container<()>, permit::Mut, All, All>) {
    // Pause-resume
    let iter = VisitDAG::isolated_space(vec![vec![unsafe {
        Key::<_, ()>::new_ref(Index::new(1).unwrap())
    }]])
    .topological(|_, _, _| Some(3))
    .reach::<(), _, _>(|_, _, _| Some(2))
    .expand(|_, _, _sink, _| ())
    .into_iter(access.as_ref().ty());
    let paused_work = iter.pause();

    // Other access
    for _work_slot in VisitDAG::rooted(unsafe { Key::<_, ()>::new_ref(Index::new(1).unwrap()) })
        .mutate()
        .shared_space()
        .topological(|_, _, _| NotNan::new(3.0f64).ok())
        .reach::<(), _, _>(|_, _, _| Some(2))
        .expand(|_, _, _, _| ())
        .into_iter(access.borrow_mut().ty())
    {}

    let mut process = VisitDAG::isolated_space(vec![vec![unsafe {
        Key::<_, ()>::new_ref(Index::new(1).unwrap())
    }]])
    .depth()
    .reach::<(), _, _>(|_, _, _| Some(2))
    .expand(|_, _, _, _| ())
    .into_process();

    for _work_slot in paused_work.resume(access.as_ref().ty()) {}

    let _ = process.step(access.as_ref().ty());

    let mut process = Box::new(process) as Box<dyn ProcessDAG<_, _, _, _, _, _>>;

    let _ = process.step(access.as_ref().ty());
}
