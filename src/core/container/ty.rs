use super::*;

// TODO: Explore if this can be a region that splits on type. Then AnyKey can be just an index.

pub trait TypeContainer<T: Item> {
    //: MultiContainer {
    type Sub: Container<T>;

    fn path(&self) -> Path;

    /// Implementations should have #[inline(always)]
    fn get(&self) -> Option<&Self::Sub>;

    fn get_mut(&mut self) -> Option<&mut Self::Sub>;

    fn fill(&mut self) -> &mut Self::Sub;
}

// pub trait MultiContainer {

// }

// *************************** Blankets ***************************

// impl<T: Item, C: TypeContainer<T> + AnyContainer> Container<T> for C {
//     type Shell = <C::Sub as Container<T>>::Shell;

//     type SlotIter<'a> = <C::Sub as Container<T>>::SlotIter<'a>;

//     #[inline(always)]
//     fn get_slot(&self, key: Key<T>) -> Option<UnsafeSlot<T, Self::Shell>> {
//         self.get().and_then(|container| container.get_slot(key))
//     }

//     fn get_context(&self, key: T::LocalityKey) -> Option<SlotContext<T>> {
//         self.get().and_then(|container| container.get_context(key))
//     }

//     fn iter_slot(&self, path: KeyPath<T>) -> Option<Self::SlotIter<'_>> {
//         self.get().and_then(|container| container.iter_slot(path))
//     }

//     fn fill_slot(&mut self, key: T::LocalityKey, item: T) -> Result<Key<T>, T> {
//         self.fill().fill_slot(key, item)
//     }

//     fn fill_context(&mut self, key: T::LocalityKey) {
//         self.fill().fill_context(key)
//     }

//     fn unfill_slot(&mut self, key: Key<T>) -> Option<(T, Self::Shell, SlotContext<T>)> {
//         self.get_mut()
//             .and_then(|container| container.unfill_slot(key))
//     }
// }
