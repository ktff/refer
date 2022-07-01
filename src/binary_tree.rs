#![allow(type_alias_bounds)]
use std::{marker::PhantomData, sync::Arc};

use crate::{field::ReadField, storage::*};

pub type ANode<'a, Store: Storage<ANodeStructure>> = ReadStructure<'a, ANodeStructure, Store>;

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

impl Structure for ANodeStructure {
    type Data<S: Storage<Self> + ?Sized> = ANodeData<S>;
    type Fields = ANodeFields;

    fn fields<S: Storage<Self> + ?Sized>(_: &Self::Data<S>) -> Self::Fields {
        ANodeFields
    }
}

// ! Storage(Key -> Container) su usko povezani

fn example<'a, Store: Storage<ANodeStructure>>(node: ANode<'a, Store>) {
    let data: u32 = node.read_data(|_| ANodeFieldsData);

    let left_key: Option<Store::K> = node.read_data(|_| ANodeFieldsLess);

    let left: Option<ANode<'a, Store>> = node.read_store_optional(|_| ANodeFieldsLess);

    let vec: &Vec<String> = node.read_data(|_| ANodeFieldsVec);
}

// pub struct BNodeStructure;

// impl Structure for BNodeStructure {
//     type Store<'a> = PlainStore<'a, BNodeData>;
//     type Fields = BNodeFields;

//     fn fields<'a>(_: &Self::Store<'a>) -> Self::Fields {
//         ANodeFields
//     }
// }
