#![allow(type_alias_bounds)]
use std::{marker::PhantomData, sync::Arc};

use crate::{field::ReadField, storage::*};

// *------------------------------------- A -------------------------------------* //

pub type ANode<'a, Store: Storage<ANodeStructure>> = ReadStructure<'a, ANodeStructure, Store>;

impl Structure for ANodeStructure {
    type Data<S: Storage<Self> + ?Sized> = ANodeData<S>;
    type Fields = ANodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S>) -> Self::Fields {
        ANodeFields
    }
}

pub struct ANodeStructure;

pub struct ANodeFields;

pub struct ANodeFieldsData;

pub struct ANodeFieldsLess;

pub struct ANodeFieldsVec;

pub struct ANodeData<S: Storage<ANodeStructure> + ?Sized> {
    data: u32,
    vec: Vec<String>,
    parent: Option<S::K>,
    less: Option<S::K>,
    greater: Option<S::K>,
    next: Option<S::K>,
}

impl<S: Storage<ANodeStructure>> ReadField<ANodeData<S>> for ANodeFieldsData {
    type To<'a> = u32 where
    S: 'a;
    fn read<'a>(&self, from: &'a ANodeData<S>) -> Self::To<'a> {
        from.data
    }
}

impl<S: Storage<ANodeStructure>> ReadField<ANodeData<S>> for ANodeFieldsLess {
    type To<'a> = Option<S::K> where
    S: 'a;
    fn read<'a>(&self, from: &'a ANodeData<S>) -> Self::To<'a> {
        from.less
    }
}

impl<S: Storage<ANodeStructure>> ReadField<ANodeData<S>> for ANodeFieldsVec {
    type To<'a> = &'a Vec<String> where
    S: 'a;
    fn read<'a>(&self, from: &'a ANodeData<S>) -> Self::To<'a> {
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

pub struct Raw<S: Storage<BNodeStructure> + ?Sized>(PhantomData<S::K>, [u8]);

impl Structure for BNodeStructure {
    type Data<S: Storage<Self> + ?Sized> = Raw<S>;
    type Fields = BNodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S>) -> Self::Fields {
        BNodeFields
    }
}

pub struct BNodeStructure;

pub struct BNodeFields;

pub struct BNodeFieldsData;

pub struct BNodeFieldsLess;

pub struct BNodeData<S: Storage<BNodeStructure> + ?Sized> {
    data: u32,
    less: Option<S::K>,
}

impl<S: Storage<BNodeStructure>> ReadField<Raw<S>> for BNodeFieldsData {
    type To<'a> = u32 where
    S: 'a;
    fn read<'a>(&self, from: &'a Raw<S>) -> Self::To<'a> {
        unimplemented!()
    }
}

impl<S: Storage<BNodeStructure>> ReadField<Raw<S>> for BNodeFieldsLess {
    type To<'a> = Option<S::K> where
    S: 'a;
    fn read<'a>(&self, from: &'a Raw<S>) -> Self::To<'a> {
        unimplemented!()
    }
}

fn example_b<'a, Store: Storage<BNodeStructure>>(node: BNode<'a, Store>) {
    let data: u32 = node.read_data(|_| BNodeFieldsData);

    let left_key: Option<Store::K> = node.read_data(|_| BNodeFieldsLess);

    let left: Option<BNode<'a, Store>> = node.read_store_optional(|_| BNodeFieldsLess);
}

fn example_b_instance<'a>(node: BNode<'a, RawStorage<BNodeStructure>>) {}
