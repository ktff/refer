#![allow(type_alias_bounds)]
use std::{marker::PhantomData, sync::Arc};

use crate::{field::ReadField, storage::*};

pub struct NodeData<K: Copy> {
    data: u32,
    vec: Vec<String>,
    parent: Option<K>,
    less: Option<K>,
    greater: Option<K>,
    next: Option<K>,
}

impl<K: Copy> KeyStore<K> for NodeData<K> {
    fn iter(&self, call: impl FnMut(Relation, K)) {
        unimplemented!()
    }

    /// May panic if this owns the key.
    fn remove(&self, key: K) -> bool {
        unimplemented!()
    }
}

impl<K: Copy> KeyStore<K> for Raw<NodeData<K>> {
    fn iter(&self, call: impl FnMut(Relation, K)) {
        unimplemented!()
    }

    /// May panic if this owns the key.
    fn remove(&self, key: K) -> bool {
        unimplemented!()
    }
}

impl<K: Copy> Into<Box<Raw<NodeData<K>>>> for NodeData<K> {
    fn into(self) -> Box<Raw<NodeData<K>>> {
        unimplemented!()
    }
}

// *------------------------------------- A -------------------------------------* //

pub struct ANodeStructure;
pub struct ANodeFields;

impl Structure for ANodeStructure {
    type T<K: Copy> = NodeData<K>;
    type Data<K: Copy> = NodeData<K>;
    type Fields = ANodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S::K>) -> Self::Fields {
        ANodeFields
    }
}

pub type ANode<'a, Store: Storage<ANodeStructure>> = ReadStructure<'a, ANodeStructure, Store>;

impl<'a, Store: Storage<ANodeStructure>> ANode<'a, Store> {
    pub fn data(&self) -> u32 {
        self.read_data(|_, data| data.data)
    }

    pub fn vec(&self) -> &Vec<String> {
        self.read_data(|_, data| &data.vec)
    }

    pub fn parent_key(&self) -> Option<Store::K> {
        self.read_data(|_, data| data.parent)
    }

    pub fn less_key(&self) -> Option<Store::K> {
        self.read_data(|_, data| data.less)
    }

    pub fn less(&self) -> Option<ANode<'a, Store>> {
        self.read_store_optional(|_, data| data.less)
    }
}

fn example_a<'a, Store: Storage<ANodeStructure>>(node: ANode<'a, Store>) {
    let data: u32 = node.data();

    let left_key: Option<Store::K> = node.less_key();

    let left: Option<ANode<'a, Store>> = node.less();

    let vec: &Vec<String> = node.vec();
}

fn example_a_instance<'a>(node: ANode<'a, PlainStorage<ANodeStructure>>) {}

// *------------------------------------- B -------------------------------------* //

pub struct Raw<T>(PhantomData<T>, [u8]);

pub struct BNodeFields;

pub struct BNodeStructure;

impl Structure for BNodeStructure {
    type T<K: Copy> = NodeData<K>;
    type Data<K: Copy> = Raw<NodeData<K>>;
    type Fields = BNodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S::K>) -> Self::Fields {
        BNodeFields
    }
}

pub type BNode<'a, Store: Storage<BNodeStructure>> = ReadStructure<'a, BNodeStructure, Store>;

impl<'a, Store: Storage<BNodeStructure>> BNode<'a, Store> {
    pub fn data(&self) -> u32 {
        self.read_data(|_, data| unimplemented!())
    }

    pub fn vec(&self) -> &Vec<String> {
        self.read_data(|_, data| unimplemented!())
    }

    pub fn parent_key(&self) -> Option<Store::K> {
        self.read_data(|_, data| unimplemented!())
    }

    pub fn less_key(&self) -> Option<Store::K> {
        self.read_data(|_, data| unimplemented!())
    }

    pub fn less(&self) -> Option<BNode<'a, Store>> {
        self.read_store_optional(|_, data| unimplemented!())
    }
}

fn example_b<'a, Store: Storage<BNodeStructure>>(node: BNode<'a, Store>) {
    let data: u32 = node.data();

    let less_key: Option<Store::K> = node.less_key();

    let less: Option<BNode<'a, Store>> = node.less();
}

fn example_b_instance<'a>(node: BNode<'a, RawStorage<BNodeStructure>>) {}
