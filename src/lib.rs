/*
! Main goal of this library is to make ArcRef<T> zero-cost abstraction.
!
! Other definition is to provide:
! - Circular references
! - Knowledge of what is referencing an item
!
! Additional goals are to provide:
! - Memory locality
! - Zero-cost memory overhead
! - Zero-cost access to items
! - Composability

use std::{
    any::Any,
    sync::{Arc, RwLock},
};


 Arc - 2 * usize
 RwLock - 12B
 usize - 8B
 Vec - 3 * usize
 ArcRef<Any> - 2 * usize
 Box - usize
 T - *
 ----------------
 2 * 8 + 12 + 8 + 3 * 8 + 8 = 68B + T + N * [ArcRef<Any>]

type ArcRef<T> = Arc<RwLock<(usize, Vec<ArcRef<Any>>, Box<T>)>>;

*/

#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

pub mod collection;
pub mod core;
pub mod item;
pub mod model;

/// Generic things
mod util;
