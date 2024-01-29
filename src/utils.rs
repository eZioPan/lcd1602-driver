//! Common tools

/// The state of a bit,
/// It's either [`BitState::Clear`] to represent a 0
/// or [`BitState::Set`] to represent a 1
#[derive(PartialEq)]
pub enum BitState {
    /// Bit is 0
    Clear,
    /// Bit is 1
    Set,
}

/// Simple bit ops
pub trait BitOps {
    #[allow(missing_docs)]
    fn set_bit(&mut self, pos: u8) -> Self;
    #[allow(missing_docs)]
    fn clear_bit(&mut self, pos: u8) -> Self;
    #[allow(missing_docs)]
    fn check_bit(&self, pos: u8) -> BitState;
}

impl BitOps for u8 {
    fn set_bit(&mut self, pos: u8) -> Self {
        assert!(pos <= 7, "bit offset larger than 7");
        *self |= 1u8 << pos;
        *self
    }

    fn clear_bit(&mut self, pos: u8) -> Self {
        assert!(pos <= 7, "bit offset larger than 7");
        *self &= !(1u8 << pos);
        *self
    }

    fn check_bit(&self, pos: u8) -> BitState {
        assert!(pos <= 7, "bit offset larger than 7");

        match self.checked_shr(pos as u32).unwrap() & 1 == 1 {
            true => BitState::Set,
            false => BitState::Clear,
        }
    }
}
