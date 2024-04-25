mod universal;
mod universal_unsafe;

pub use universal::Slot;
pub use universal_unsafe::AnyUnsafeSlot;

pub type DynSlot<'a, R, T> = Slot<'a, R, T>;
pub type UnsafeSlot<'a, T> = AnyUnsafeSlot<'a, T>;
