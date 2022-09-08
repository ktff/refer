mod collection;
mod container;
mod item;
mod key;
mod reference;
mod shell;

pub use collection::*;
pub use container::*;
pub use item::*;
pub use key::*;
pub use reference::*;
pub use shell::*;

/*
NOTES

- Goal is to completely prevent memory errors, and to discourage logical errors.

- If a branch is not correct from the point of logic/expectations but the end result is the same then just log the
  the inconsistency and continue. And if the result is not the same return Option/Error. While for
  fatal/unrecoverable/inconsistent_states it should panic.

- Multi level containers must know/enforce levels on their children containers so to have an unique path for each key.

- Containers are not to be Items since that creates non trivial recursions on type and logic levels.


TODO:

   * LocalBox<T>
   * Split Item Access, zahtjeva da se dropa potpora za locking, Polly ItemCollection can split &mut self to multiple &mut views each with set of types that don't overlap.
   * Finish DeltaKey

*/
