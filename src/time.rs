type TimeTy = u64;

/// Elapsed time in measure of virtual ticks.
/// Virtual in a sense that a single real tick
/// can move time by multiple virtual ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Time(pub TimeTy);

// TODO: make macro for this

impl Property for Time {
    const ID: PropertyId = PropertyId::new(4);
}

impl Encode for Time {
    const MAX_LEN: usize = <TimeTy as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for Time {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}

/// A real tick valued with given amount of virtual ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Tick(pub TimeTy);

// TODO: make macro for this

impl Property for Tick {
    const ID: PropertyId = PropertyId::new(5);
}

impl Encode for Tick {
    const MAX_LEN: usize = <TimeTy as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for Tick {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}
