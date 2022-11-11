mod any_unsafe;
mod mut_any;
mod mut_any_item;
mod mut_any_shell;
mod mut_item;
mod mut_shell;
mod mut_slot;
mod ref_any;
mod ref_any_item;
mod ref_any_shell;
mod ref_item;
mod ref_shell;
mod ref_slot;
mod unsafe_slot;

pub use any_unsafe::AnyUnsafeSlot;
pub use mut_any::MutAnySlot;
pub use mut_any_item::MutAnyItemSlot;
pub use mut_any_shell::MutAnyShellSlot;
pub use mut_item::MutItemSlot;
pub use mut_shell::MutShellSlot;
pub use mut_slot::MutSlot;
pub use ref_any::RefAnySlot;
pub use ref_any_item::RefAnyItemSlot;
pub use ref_any_shell::RefAnyShellSlot;
pub use ref_item::RefItemSlot;
pub use ref_shell::RefShellSlot;
pub use ref_slot::RefSlot;
pub use unsafe_slot::UnsafeSlot;

// Transition lists:
//
// UnsafeSlot
//  -> RefSlot
//      -> RefItemSlot
//      -> RefShellSlot
//      -> RefAnySlot
//  -> RefItemSlot
//      -> RefAnyItemSlot
//  -> RefShellSlot
//      -> RefAnyShellSlot
//  -> MutSlot
//      -> (MutItemSlot, MutShellSlot)
//      -> MutAnySlot
//  -> MutItemSlot
//      -> MutAnyItemSlot
//  -> MutShellSlot
//      -> MutAnyShellSlot
//
// AnyUnsafeSlot
//  -> RefAnySlot
//      -> RefAnyItemSlot
//      -> RefAnyShellSlot
//      -> RefSlot
//  -> RefAnyItemSlot
//      -> RefItemSlot
//  -> RefAnyShellSlot
//      -> RefShellSlot
//  -> MutAnySlot
//      -> (MutAnyItemSlot, MutAnyShellSlot)
//      -> MutSlot
//  -> MutAnyItemSlot
//      -> MutItemSlot
//  -> MutAnyShellSlot
//      -> MutShellSlot
//

// Conventions:
//
// fn into_[slot|item|shell][|_mut]() -> [Ref|Mut][|Any][Slot|Item|Shell]Slot
// fn split(self) -> (_ItemSlot,_ShellSlot)
// fn upcast(self) -> AnySelf
// fn downcast<T>(AnySelf) -> Option<Self<T>>
//
// fn field_name(&self) -> &T
// fn field_name_mut(&mut self) -> &mut T
