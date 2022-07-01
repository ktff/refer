use std::marker::PhantomData;

pub trait ReadField<From: ?Sized> {
    type To<'a>
    where
        From: 'a;
    fn read<'a>(&self, from: &'a From) -> Self::To<'a>;
}

pub struct CopyReadField<To: Copy>(PhantomData<To>);

impl<To: Copy + 'static> ReadField<[u8]> for CopyReadField<To> {
    type To<'a> = To;
    fn read<'a>(&self, from: &'a [u8]) -> Self::To<'a> {
        assert!(from.len() >= std::mem::size_of::<To>());
        unsafe { std::ptr::read_unaligned(from.as_ptr() as *const To) }
    }
}

impl<To: Copy + 'static> ReadField<[u8; std::mem::size_of::<To>()]> for CopyReadField<To> {
    type To<'a> = To;
    fn read<'a>(&self, from: &'a [u8; std::mem::size_of::<To>()]) -> Self::To<'a> {
        unsafe { std::ptr::read_unaligned(from.as_ptr() as *const To) }
    }
}

impl<To: Copy + 'static> ReadField<To> for CopyReadField<To> {
    type To<'a> = To;
    fn read<'a>(&self, from: &'a To) -> Self::To<'a> {
        *from
    }
}

// pub trait FindField<const N: usize> {
//     type From;
//     type To;

//     fn find<'a>(&self, from: &'a Self::From) -> &'a Self::To;
// }

// pub struct FieldStack<const N: usize, Field, Stack> {

// }

// impl<N> FindField<N> for FieldStack<N>{

// }
