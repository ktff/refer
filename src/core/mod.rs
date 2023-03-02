#![allow(type_alias_bounds)]

#[macro_use]
pub mod container;
mod edge;
mod error;
mod locality;
#[macro_use]
mod item;
mod key;
pub mod permit;
mod slot;

pub use container::{AnyContainer, Container};
pub use edge::*;
pub use error::*;
pub use item::*;
pub use key::*;
pub use locality::*;
pub use permit::{AddPermit, AnyPermit, Permit, SlotSplitPermit, TypePermit, TypeSplitPermit};

pub use slot::*;

// *************************** Useful aliases *************************** //

pub type Result<T> = std::result::Result<T, ReferError>;
/*
NOTES
- Goal is to completely prevent memory errors, and to discourage logical errors.

- If a branch is not correct from the point of logic/expectations but the end result is the same then just log the
  the inconsistency and continue. And if the result is not the same return Option/Error. While for
  fatal/unrecoverable/inconsistent_states it should panic.

- Multi level containers must know/enforce levels on their children containers so to have an unique path for each key.

- Containers are not to be Items since that creates non trivial recursions on type and logic levels.
*/
