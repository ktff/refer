use std::any::Any;

use super::{AnyEntity, AnyKey, AnyShell, Error};

pub trait AnyCollection: AnyKeyCollection {
    fn entity_any(&self, key: AnyKey) -> Result<Box<dyn AnyEntity<'_>>, Error>;
}

pub trait AnyItemCollection: AnyKeyCollection {
    fn item_any(&self, key: AnyKey) -> Result<&dyn Any, Error>;
}

pub trait AnyShellCollection: AnyKeyCollection {
    // /// How many lower bits of indices can be used for keys.
    // fn indices_bits(&self) -> usize;

    fn shell_any(&self, key: AnyKey) -> Result<Box<dyn AnyShell<'_>>, Error>;
}

pub trait AnyKeyCollection {
    fn first_any(&self) -> Option<AnyKey>;

    /// Returns following key after given with indices in ascending order.
    /// Order according to type is undefined.
    fn next_any(&self, key: AnyKey) -> Option<AnyKey>;
}
