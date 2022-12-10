mod index;
mod key;
mod prefix;

use std::{
    any::{self, Any, TypeId},
    fmt::{self},
    hash::{Hash, Hasher},
    marker::{PhantomData, Unsize},
    ops::CoerceUnsized,
    ptr::{DynMetadata, Pointee},
};

pub use index::*;
pub use key::{AnyKey, Key};
pub use prefix::*;

use crate::core::{AnyContainer, AnyPermit, AnySlot};

// TODO: Test this whole stack

// TODO: Revisit, refactor, Any X {Key,Prefix,SubKey} and Index concepts, interaction, etc.
// // TODO: BottomKey ?

// ? Index - underlying storage - operacije se nebi trebale moci izvoditi na njemu vec samo kada mu se pridjele uloge, Key/Top
// ? Key - 1 to 1 mapping to Slot

// ! Izgradnje:
// ? ∅ -> ContainerKey -> ContainerKey[ContainerKey] -> ... -> Context
// ? ... -> Top -> Top -> ∅ => (Top Xor ContainerKey) >> bottom_offset, if (Top And ContainerKey.mask) == 0
// // ? ... -> Bottom[Bottom] -> Bottom -> NonZeroUsize => (Key Xor Top) >> bottom_offset
// ? Key - ContainerKey -> NonZeroUsize => (Key Xor ContainerKey) >> ContainerKey.bottom_offset
// ? ContainerKey + NonZeroUsize -> Key => ContainerKey | NonZeroUsize
// // ? Key -> Bottom
// ? Key -> Top
// ? Top can have zero set bits, bit bellow lowest set is set to 1, may at minimal represent two slots.
