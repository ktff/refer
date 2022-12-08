use crate::core::{Allocator, AnyItem, Container, ReservedKey, Shell, SubKey, UnsafeSlot};
use std::{any::Any, ops::Range};

// TODO: Revisit this

// pub trait Layer<T: AnyItem> {
//     type Alloc: std::alloc::Allocator + 'static;

//     type GroupItem: Any;

//     type Shell: Shell<T = T>;

//     type R: Copy;

//     type Inner: Container<T>;

//     type IterInner<'a>: Iterator<Item = (Option<usize>, &'a Self::Inner)>
//     where
//         Self: 'a;

//     fn get_inner(&self, index: Option<usize>) -> Option<&Self::Inner>;

//     fn get_inner_mut(&mut self, index: Option<usize>) -> Option<&mut Self::Inner>;

//     fn iter_inner(&self) -> Option<Self::IterInner<'_>>;

//     fn range_inner(&self) -> Option<Range<usize>>;

//     fn key_len(&self) -> Option<u32>;

//     fn assign(
//         &mut self,
//         item: Option<&T>,
//         r: Self::R,
//     ) -> Option<(Option<usize>, <Self::Inner as Allocator>::R)>;

//     fn transform(
//         &self,
//         key: SubKey<T>,
//         slot: UnsafeSlot<
//             T,
//             <Self::Inner as Container<T>>::GroupItem,
//             <Self::Inner as Container<T>>::Shell,
//             <Self::Inner as Allocator<T>>::Alloc,
//         >,
//     ) -> Option<UnsafeSlot<T, Self::GroupItem, Self::Shell, Self::Alloc>>;
// }

pub trait TransformLayer<T: AnyItem> {
    type Alloc: std::alloc::Allocator + 'static;

    type GroupItem: Any;

    type Shell: Shell<T = T>;

    type Inner: Container<T>;

    fn get_inner(&self) -> &Self::Inner;

    fn get_inner_mut(&mut self) -> &mut Self::Inner;

    fn transform(
        &self,
        key: SubKey<T>,
        slot: UnsafeSlot<
            T,
            <Self::Inner as Container<T>>::GroupItem,
            <Self::Inner as Container<T>>::Shell,
            <Self::Inner as Allocator<T>>::Alloc,
        >,
    ) -> Option<UnsafeSlot<T, Self::GroupItem, Self::Shell, Self::Alloc>>;
}

pub trait DisperseLayer<T: AnyItem>: Send + Sync {
    type R: Copy;

    type Inner: Container<T>;

    type IterInner<'a>: Iterator<Item = (usize, &'a Self::Inner)>
    where
        Self: 'a;

    type IterInnerMut<'a>: Iterator<Item = (usize, &'a mut Self::Inner)>
    where
        Self: 'a;

    fn get_inner(&self, index: usize) -> Option<&Self::Inner>;

    fn get_inner_mut(&mut self, index: usize) -> Option<&mut Self::Inner>;

    fn iter_inner(&self) -> Self::IterInner<'_>;

    fn iter_inner_mut(&mut self) -> Self::IterInnerMut<'_>;

    fn range_inner(&self) -> Range<usize>;

    fn key_len(&self) -> u32;

    fn assign(
        &mut self,
        item: Option<&T>,
        r: Self::R,
    ) -> Option<(usize, <Self::Inner as Allocator<T>>::R)>;
}

// *********************************** IMPLs *********************************** //
// This can be default impl once specialization is implemented properly.

// #[macro_export]
// macro_rules!  {
//     () => {

//     };
// }

// default impl<T: AnyItem, L: DisperseLayer<T>> Allocator<T> for L {
//     type Alloc = <<L as DisperseLayer<T>>::Inner as Allocator<T>>::Alloc;

//     type R = <L as DisperseLayer<T>>::R;

//     fn reserve(&mut self, item: Option<&T>, r: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)> {
//         let (index, r) = self.assign(item, r)?;
//         let (sub_key, alloc) = self
//             .get_inner_mut(index)
//             .expect("Invalid assign index")
//             .reserve(item, r)?;
//         Some((sub_key.push(self.key_len(), index), alloc))
//     }

//     fn cancel(&mut self, key: ReservedKey<T>) {
//         let (index, sub_key) = key.pop(self.key_len());
//         self.get_inner_mut(index)
//             .expect("Invalid index")
//             .cancel(sub_key);
//     }

//     fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
//     where
//         T: Sized,
//     {
//         let (index, sub_key) = key.pop(self.key_len());
//         self.get_inner_mut(index)
//             .expect("Invalid index")
//             .fulfill(sub_key, item)
//             .push(self.key_len(), index)
//     }

//     fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
//     where
//         T: Sized,
//     {
//         let (index, sub_key) = key.pop(self.key_len());
//         self.get_inner_mut(index)
//             .expect("Invalid index")
//             .unfill(sub_key)
//     }
// }
