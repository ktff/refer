#![allow(type_alias_bounds)]
use std::{marker::PhantomData, sync::Arc};

use crate::{field::ReadField, storage::*};

// *********************** Storage ***************************** //
/*
Ideja je da je storage dio strukture stoga ima smisla ju explicitno definirati.
To također znaci da nema generic parametara za to.
*/

type NodeStorage = PlainStorage<usize, Node>;
type NodeKey = <NodeStorage as Storage<Node>>::K;
type NodeRead<'a> = ReadStructure<'a, Node, NodeStorage>;

// *********************** Node ******************************* //

pub struct Node {
    data: u32,
    vec: Vec<String>,
    parent: Option<NodeKey>,
    less: Option<NodeKey>,
    greater: Option<NodeKey>,
    next: Option<NodeKey>,
}

impl Instance<NodeKey> for Node {
    fn for_each_ref(&self, call: impl FnMut(Relation, NodeKey)) {
        unimplemented!()
    }

    fn remove_ref(&mut self, key: NodeKey) -> bool {
        unimplemented!()
    }
}

fn example<'a>(node: NodeRead<'a>) {
    let data: u32 = node.data;

    let left_key: Option<NodeKey> = node.less;

    let left: Option<NodeRead<'a>> = node.less.map(|less| node.read(less));

    let vec: &Vec<String> = &node.vec;
}
