#![allow(type_alias_bounds)]
use std::{marker::PhantomData, sync::Arc};

use crate::{field::ReadField, storage::*};

// *------------------------------------- A -------------------------------------* //

pub type ANode<'a, Store: Storage<ANodeStructure>> = ReadStructure<'a, ANodeStructure, Store>;

impl Structure for ANodeStructure {
    type Data<K: Copy> = ANodeData<K>;
    type Fields = ANodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S::K>) -> Self::Fields {
        ANodeFields
    }
}

pub struct ANodeStructure;

pub struct ANodeFields;

pub struct ANodeFieldsData;

pub struct ANodeFieldsLess;

pub struct ANodeFieldsVec;

pub struct ANodeData<K: Copy> {
    data: u32,
    vec: Vec<String>,
    parent: Option<K>,
    less: Option<K>,
    greater: Option<K>,
    next: Option<K>,
}

impl<K: Copy> ReadField<ANodeData<K>> for ANodeFieldsData {
    type To<'a> = u32 where K:'a;
    fn read<'a>(&self, from: &'a ANodeData<K>) -> Self::To<'a> {
        from.data
    }
}

impl<K: Copy> ReadField<ANodeData<K>> for ANodeFieldsLess {
    type To<'a> = Option<K> where K:'a;
    fn read<'a>(&self, from: &'a ANodeData<K>) -> Self::To<'a> {
        from.less
    }
}

impl<K: Copy> ReadField<ANodeData<K>> for ANodeFieldsVec {
    type To<'a> = &'a Vec<String> where K:'a;
    fn read<'a>(&self, from: &'a ANodeData<K>) -> Self::To<'a> {
        &from.vec
    }
}

fn example_a<'a, Store: Storage<ANodeStructure>>(node: ANode<'a, Store>) {
    let data: u32 = node.read_data(|_| ANodeFieldsData);

    let left_key: Option<Store::K> = node.read_data(|_| ANodeFieldsLess);

    let left: Option<ANode<'a, Store>> = node.read_store_optional(|_| ANodeFieldsLess);

    let vec: &Vec<String> = node.read_data(|_| ANodeFieldsVec);
}

fn example_a_instance<'a>(node: ANode<'a, PlainStorage<ANodeStructure>>) {}

// *------------------------------------- B -------------------------------------* //

pub type BNode<'a, Store: Storage<BNodeStructure>> = ReadStructure<'a, BNodeStructure, Store>;

pub struct Raw<K: Copy>(PhantomData<K>, [u8]);

impl Structure for BNodeStructure {
    type Data<K: Copy> = Raw<K>;
    type Fields = BNodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S::K>) -> Self::Fields {
        BNodeFields
    }
}

pub struct BNodeStructure;

pub struct BNodeFields;

pub struct BNodeFieldsData;

pub struct BNodeFieldsLess;

pub struct BNodeData<K: Copy> {
    data: u32,
    less: Option<K>,
}

impl<K: Copy> ReadField<Raw<K>> for BNodeFieldsData {
    type To<'a> = u32 where K:'a;
    fn read<'a>(&self, from: &'a Raw<K>) -> Self::To<'a> {
        unimplemented!()
    }
}

impl<K: Copy> ReadField<Raw<K>> for BNodeFieldsLess {
    type To<'a> = Option<K> where K:'a;
    fn read<'a>(&self, from: &'a Raw<K>) -> Self::To<'a> {
        unimplemented!()
    }
}

fn example_b<'a, Store: Storage<BNodeStructure>>(node: BNode<'a, Store>) {
    let data: u32 = node.read_data(|_| BNodeFieldsData);

    let left_key: Option<Store::K> = node.read_data(|_| BNodeFieldsLess);

    let left: Option<BNode<'a, Store>> = node.read_store_optional(|_| BNodeFieldsLess);
}

fn example_b_instance<'a>(node: BNode<'a, RawStorage<BNodeStructure>>) {}
