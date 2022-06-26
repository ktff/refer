use crate::coders::{Decode, Encode};

/// Container of properties.
pub trait Container {
    type PropertiesIter<'a>: Iterator<Item = (PropertyId, &'a [u8])>
    where
        Self: 'a;

    /// Will be inserted at some point.
    fn set(&mut self, ty: PropertyId, data: &[u8]);

    fn properties<'a>(&'a self) -> Self::PropertiesIter<'a>;

    fn get<'a>(&'a self, ty: PropertyId) -> Option<&'a [u8]> {
        self.properties()
            .find(|(id, _)| *id == ty)
            .map(|(_, data)| data)
    }

    fn contains<'a>(&'a self, ty: PropertyId) -> bool {
        self.properties().any(|(id, _)| id == ty)
    }

    /// Will be removed at some point
    fn remove(&mut self, ty: PropertyId);

    fn update<P: Property>(&mut self, property: Option<P>)
    where
        [u8; P::MAX_LEN]: Sized,
    {
        if let Some(property) = property {
            let mut buffer = [0; P::MAX_LEN];
            let len = property.encode(&mut buffer);
            let data = &buffer[..len];

            if self.get(P::ID) != Some(data) {
                self.set(P::ID, data);
            }
        } else if self.contains(P::ID) {
            self.remove(P::ID)
        }
    }
}

// TODO: Family of properties?s

pub trait Property: Decode + Sized {
    const ID: PropertyId;

    fn set(&self, con: &mut impl Container)
    where
        [u8; Self::MAX_LEN]: Sized,
    {
        let mut buffer = [0; Self::MAX_LEN];
        let len = self.encode(&mut buffer);
        con.set(Self::ID, &buffer[..len]);
    }

    fn of(con: &impl Container) -> Option<Self> {
        con.get(Self::ID).map(Self::decode)
    }

    fn remove(con: &mut impl Container) {
        con.remove(Self::ID);
    }
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
