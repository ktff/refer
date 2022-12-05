use std::any::TypeId;

use crate::core::*;

pub trait Dispatcher {
    /// Calls F for T that corresponds to given type.
    /// None if C doesn't implement Container<T>.
    fn dispatch_mut<F: MutFunction>(&self, ty: TypeId, f: F) -> Option<F::Output>;

    // /// Calls F for T that corresponds to given type.
    // /// None if C doesn't implement Container<T>.
    // fn dispatch<F: RefFunction<Self::C>>(
    //     &self,
    //     ty: TypeId,
    //     container: &Self::C,
    //     f: F,
    // ) -> Option<F::Output>;
}

pub trait MutFunction {
    type Output;
    fn call<T: Item, C: Container<T>>(self, container: &mut C) -> Self::Output;
}

// pub trait RefFunction<C> {
//     type Output;
//     fn call<T: AnyItem>(self, container: &C) -> Self::Output
//     where
//         C: Container<T>;
// }
