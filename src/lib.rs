#![feature(type_alias_impl_trait)]
#![feature(const_option)]
#![feature(negative_impls)]
// Currently this is not implemented safely for *dyn Trait.
#![feature(trait_upcasting)]
#![feature(allocator_api)]
#![feature(generic_const_exprs)]
#![feature(int_roundings)]
#![feature(maybe_uninit_uninit_array)]
#![feature(new_uninit)]
#![feature(ptr_metadata)]
#![feature(maybe_uninit_slice)]
#![feature(layout_for_ptr)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_uninit)]
#![feature(impl_trait_in_assoc_type)]

//! # Goal
//! The main goal of this library is to provide foundation for programs
//! that are modeling graph like problems.
//!
//! Primary attribute of the library is composability for achieving code reuse,
//! flexibility, and zero-cost abstractions or at least zero-overhead principle.
//!
//! Secondary attribute is performance of memory and computation. This is achieved
//! by enabling such optimizations, and then optionally providing implementations
//! that exploit them.
//!
//! # Features
//! - Reference management, through Item and Shell family of traits.
//! - Memory management, through Container family of traits.
//! - Access management, through Collection family of traits.
//!
//! # Architecture
//! There are several pieces that interact/are composable with one another:
//! - Model - the what's being build using the library.
//! - Items - the building blocks of the model.
//! - Shells - associated to each item and used to record references to its item.
//! - Collections - provides access to contained items and shells of a model.
//! - Containers - stores/contains items and shells.
//! - Reference - a reference to an (item,shell) that is supposed to be tracked and valid.
//! - Ids - provides identity management which ties all of the other pieces together.
//!
//! Ids and references are concrete types that are not intended to be extended and are provided
//! by the library.
//!
//! Collections, containers, and shells, are trait families that are intended to be implemented
//! for the model if some of the provided generic implementations are not sufficient.
//!
//! Items are trait families that are intended to be implemented for the model.
//!
//! Models aren't represented in any way by the library besides providing some examples/generic implementations.
//!
//! # Examples
//! TODO

pub mod collection;
#[macro_use]
pub mod container;
pub mod core;
#[macro_use]
pub mod item;
pub mod components;
pub mod model;
pub mod shell;

pub use crate::core::*;

// Generic things
mod util;
