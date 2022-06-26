pub trait Encode {
    const MAX_LEN: usize;

    /// Return len of used slice.
    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize;
}

pub trait Decode: Encode {
    fn decode(buffer: &[u8]) -> Self;
}

// ******************* impl *********************** //

impl Encode for u32 {
    const MAX_LEN: usize = 4;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for u32 {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}

impl Encode for u64 {
    const MAX_LEN: usize = 8;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for u64 {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}
