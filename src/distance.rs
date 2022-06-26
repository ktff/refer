use crate::{coders::*, graph::*, property::*};

type DistanceTy = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Distance(pub DistanceTy);

// TODO: make macro for this

impl Property for Distance {
    const ID: PropertyId = PropertyId::new(0);
}

impl Encode for Distance {
    const MAX_LEN: usize = <DistanceTy as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for Distance {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}

type VelocityTy = i64;

/// Velocity per tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Velocity(pub VelocityTy);

impl Velocity {
    pub fn none_if_zero(self) -> Option<Self> {
        if self.0 == 0 {
            None
        } else {
            Some(self)
        }
    }
}

// TODO: make macro for this

impl Property for Velocity {
    const ID: PropertyId = PropertyId::new(3);
}

impl Encode for Velocity {
    const MAX_LEN: usize = <VelocityTy as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for Velocity {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}
