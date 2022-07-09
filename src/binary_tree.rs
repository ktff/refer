#![allow(type_alias_bounds)]
use std::{marker::PhantomData, sync::Arc};

use crate::{field::ReadField, storage::*};

pub struct NodeFamily;

impl Family for NodeFamily {
    type I<K: Key> = Node<K>;
}

pub struct Node<K: Key> {
    data: u32,
    vec: Vec<String>,
    parent: Option<K>,
    less: Option<K>,
    greater: Option<K>,
    next: Option<K>,
}

impl<K: Key> Instance<K> for Node<K> {
    fn iter(&self, call: impl FnMut(Relation, K)) {
        unimplemented!()
    }

    fn remove_ref(&mut self, key: K) -> bool {
        unimplemented!()
    }
}

type NodeRead<'a, Store: Storage<NodeFamily>> = ReadStructure<'a, NodeFamily, Store>;

fn example<'a, Store: Storage<NodeFamily>>(node: NodeRead<'a, Store>) {
    let data: u32 = node.data;

    let left_key: Option<Store::K> = node.less;

    let left: Option<NodeRead<'a, Store>> = node.less.map(|less| node.read(less));

    let vec: &Vec<String> = &node.vec;
}

fn example_instance<'a>(_: NodeRead<'a, PlainStorage<NodeFamily>>) {}
