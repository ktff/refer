mod order;
mod start;

use std::{
    any::Any,
    borrow::Borrow,
    collections::{binary_heap::Iter, HashSet},
    marker::PhantomData,
};

use super::{
    permit::access::*,
    permit::{self, Permit},
    AnyContainer, Container, DynContainer, DynItem, Index, Item, Key, Ref, Slot,
};
use order::{Order, OrderJoin};
use start::Start;

fn iter<T>(access: Access<impl Container<()>, permit::Mut, All, All>) {
    for _work_slot in IterDag::rooted(unsafe { Key::<_, ()>::new_ref(Index::new(1).unwrap()) })
        // .mutate()
        .filter_map(|input, node| Some(2))
        .edges_map(|input, node| Vec::<(bool, _)>::new().into_iter())
        .depth()
        .into_iter(access.ty().as_ref())
    {}
}

// ? NOTE: Zahtijevati usmjerenost edge-ova?
// ?        Odnosno da se ukloni eq edge ili mu se barem da smjer?
// !        Nije nužno.

// Starting nodes:
// b) Subset
// c) One

//       X

// Order of iteration:
// a) Depth first
// b) Breadth first
// c) Topological sort X custom + key

//       X

// Node processing + filter + map to data(passed to edge processing) X Any + Typed

//       X

// Edge processing + filter + map to data(passed to child node processing) X Any + Typed

//       X

// Corner cases passing = edge going to already processed node + dropped not covered node type

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
        start: HashSet<Key<Ref<'a>, T>>,
    ) -> IterDag<'a, C, start::Subset<'a, T>, permit::Ref> {
        IterDag {
            start: start::Subset(start),
            permit: PhantomData,
            ..Self::new()
        }
    }

    pub fn rooted<T: DynItem + ?Sized>(
        start: Key<Ref<'a>, T>,
    ) -> IterDag<'a, C, start::Root<T>, permit::Ref> {
        IterDag {
            start: start::Root(start),
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
    // TODO: Option to enable for not expanded nodes to be called again with expanded [NI].

    pub fn filter_map<
        NI: 'a,
        NP: FnMut(&[NI], &mut Slot<'a, P, S::T>) -> Option<NO> + 'a,
        NO: 'a,
    >(
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
    pub fn topological<O: FnMut(&NI, SlotAccess<C, P, S::T>) -> Option<TO>, TO: Ord + Copy>(
        self,
        order: O,
        join: OrderJoin<TO>,
    ) -> IterDag<'a, C, S, P, NI, NP, NO, EP, EI, order::Topological<O, TO>>
    where
        C: AnyContainer,
    {
        IterDag {
            order: order::Topological(order, join),
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

impl<'a, C: AnyContainer + ?Sized, S: Start<'a>, P: Permit, NI, NP, NO, EP, EI, O: Order>
    IterDag<'a, C, S, P, NI, NP, NO, EP, EI, O>
{
    pub fn into_iter(
        self,
        access: Access<'a, C, P, impl Permits<S::T>, impl KeyPermit>,
    ) -> impl Iterator<Item = IterNode<'a, P, S::T, NI, NO>> + 'a {
        // NOTES:
        // - Only ever use get_try for getting slots to cover cases when a sub container is passed here or constrictive KeyPermit
        // - More generally this should be robust against malformed user input
        // TODO
        std::iter::empty()
    }
}

pub enum IterNode<'a, P: Permit, T: DynItem + ?Sized, IN, OUT> {
    /// Was not expanded.
    Idle(Key<Ref<'a>, T>),

    Expanded(Slot<'a, P, T>, OUT),

    /// This node was called out of order.
    OutOfOrder(IN, Key<Ref<'a>, T>),
}
