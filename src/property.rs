use crate::coders::{Decode, Encode};

/// Container of properties.
pub trait Container {
    type PropertiesIter<'a>: Iterator<Item = (PropertyId, &'a [u8])>
    where
        Self: 'a;

    fn properties<'a>(&'a self) -> Self::PropertiesIter<'a>;

    fn get<P: Property>(&self) -> Option<P> {
        self.get_ty(P::ID).map(P::decode)
    }

    fn get_ty<'a>(&'a self, ty: PropertyId) -> Option<&'a [u8]> {
        self.properties()
            .find(|(id, _)| *id == ty)
            .map(|(_, data)| data)
    }
}

pub trait Property: Decode {
    const ID: PropertyId;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyId(u32);

impl PropertyId {
    pub const fn new(id: u32) -> Self {
        PropertyId(id)
    }
}

impl Encode for PropertyId {
    const MAX_LEN: usize = <u32 as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for PropertyId {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}
