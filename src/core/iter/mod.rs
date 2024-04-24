mod order;
mod start;

use std::{
    any::Any,
    collections::{binary_heap::Iter, HashSet},
    marker::PhantomData,
};

use super::{
    permit::access::*,
    permit::{self, Permit},
    AnyContainer, Container, DynItem, DynSlot, Index, Item, Key, Ref, Slot,
};
use order::Order;
use start::Start;

pub type DataIterator<T> = impl Iterator<Item = T>;

fn iter<T>(access: Access<impl Container<()>, permit::Mut, All, All>) -> DataIterator<T> {
    IterDag::rooted(unsafe { Key::<_, ()>::new_ref(Index::new(1).unwrap()) })
        .as_mut()
        .depth()
        .filter_map(|input, node| Some(2))
        .edges_map_dyn(|input, node| Vec::<(bool, _)>::new().into_iter())
        .run(access.ty());

    Vec::<T>::new().into_iter()
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
    S = (),
    P = (),
    O = (),
    NI: 'a = (),
    NP: 'a = (),
    NO: 'a = (),
    EP: 'a = (),
    EI: 'a = (),
> {
    _ref: PhantomData<&'a ()>,
    start: S,
    permit: PhantomData<P>,
    order: O,
    node_input: PhantomData<&'a NI>,
    node_processor: NP,
    node_output: PhantomData<&'a NO>,
    edge_processor: EP,
    edge_iterator: PhantomData<&'a EI>,
}

impl<'a> IterDag<'a> {
    fn new() -> Self {
        IterDag {
            _ref: PhantomData,
            start: (),
            permit: PhantomData,
            order: (),
            node_input: PhantomData,
            node_processor: (),
            node_output: PhantomData,
            edge_processor: (),
            edge_iterator: PhantomData,
        }
    }

    pub fn subset<T: DynItem + ?Sized>(
        start: HashSet<Key<Ref<'a>, T>>,
    ) -> IterDag<'a, start::Subset<'a, T>> {
        IterDag {
            start: start::Subset(start),
            ..Self::new()
        }
    }

    pub fn rooted<T: DynItem + ?Sized>(start: Key<Ref<'a>, T>) -> IterDag<'a, start::Root<T>> {
        IterDag {
            start: start::Root(start),
            ..Self::new()
        }
    }
}

impl<'a, S: Start<'a>> IterDag<'a, S> {
    pub fn as_ref(self) -> IterDag<'a, S, permit::Ref> {
        IterDag {
            permit: PhantomData,
            ..self
        }
    }
    pub fn as_mut(self) -> IterDag<'a, S, permit::Mut> {
        IterDag {
            permit: PhantomData,
            ..self
        }
    }
}

impl<'a, S: Start<'a>, P: Permit> IterDag<'a, S, P> {
    pub fn depth(self) -> IterDag<'a, S, P, order::Depth> {
        IterDag {
            order: order::Depth,
            ..self
        }
    }

    pub fn breadth(self) -> IterDag<'a, S, P, order::Breadth> {
        IterDag {
            order: order::Breadth,
            ..self
        }
    }

    pub fn topological<F: FnMut(&mut Slot<'a, P, S::T>) -> Option<usize>>(
        self,
        order: F,
    ) -> IterDag<'a, S, P, order::Topological<F>>
    where
        S::T: Item,
    {
        IterDag {
            order: order::Topological(order),
            ..self
        }
    }

    pub fn topological_dyn<F: FnMut(&mut DynSlot<'a, P, S::T>) -> Option<usize>>(
        self,
        order: F,
    ) -> IterDag<'a, S, P, order::Topological<F>> {
        IterDag {
            order: order::Topological(order),
            ..self
        }
    }

    pub fn topological_by_key(self) -> IterDag<'a, S, P, order::TopologicalKey> {
        IterDag {
            order: order::TopologicalKey,
            ..self
        }
    }
}

impl<'a, S: Start<'a>, P: Permit, O: Order> IterDag<'a, S, P, O> {
    pub fn filter_map<
        NI: 'a,
        NP: FnMut(DataIterator<NI>, &mut Slot<'a, P, S::T>) -> Option<NO> + 'a,
        NO: 'a,
    >(
        self,
        node_processor: NP,
    ) -> IterDag<'a, S, P, O, NI, NP, NO>
    where
        S::T: Item,
    {
        IterDag {
            node_input: PhantomData,
            node_processor,
            node_output: PhantomData,
            ..self
        }
    }

    pub fn filter_map_dyn<
        NI: 'a,
        NP: FnMut(DataIterator<NI>, &mut DynSlot<'a, P, S::T>) -> Option<NO> + 'a,
        NO: 'a,
    >(
        self,
        node_processor: NP,
    ) -> IterDag<'a, S, P, O, NI, NP, NO>
    where
        S::T: Item,
    {
        IterDag {
            node_input: PhantomData,
            node_processor,
            node_output: PhantomData,
            ..self
        }
    }
}

impl<'a, S: Start<'a>, P: Permit, O: Order, NI, NP, NO> IterDag<'a, S, P, O, NI, NP, NO> {
    pub fn edges_map<
        EP: FnMut(NO, &mut Slot<'a, P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
    >(
        self,
        edge_processor: EP,
    ) -> IterDag<'a, S, P, O, NI, NP, NO, EP, EI>
    where
        S::T: Item,
    {
        IterDag {
            edge_processor,
            edge_iterator: PhantomData,
            ..self
        }
    }

    pub fn edges_map_dyn<
        EP: FnMut(NO, &mut DynSlot<'a, P, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
    >(
        self,
        edge_processor: EP,
    ) -> IterDag<'a, S, P, O, NI, NP, NO, EP, EI> {
        IterDag {
            edge_processor,
            edge_iterator: PhantomData,
            ..self
        }
    }
}

impl<'a, S: Start<'a>, P: Permit, O: Order, NI, NP, NO, EP, EI>
    IterDag<'a, S, P, O, NI, NP, NO, EP, EI>
{
    pub fn run(self, access: Access<'a, impl Container<S::T>, P, S::T, impl KeyPermit>)
    where
        S::T: Item,
    {
        unimplemented!()
    }

    pub fn run_dyn(self, access: Access<'a, impl AnyContainer, P, All, impl KeyPermit>) {
        unimplemented!()
    }
}
