use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::utils::{BitOps, BitState};

use super::{Pins, PinsInternalAPI};

impl<ControlPin, DBPin, const PIN_CNT: usize> PinsInternalAPI for Pins<ControlPin, DBPin, PIN_CNT>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
{
    fn push_bits(&mut self, raw_bits: u8) {
        self.db_pins
            .iter_mut()
            .enumerate()
            .for_each(|(index, pin)| match raw_bits.check_bit(index as u8) {
                BitState::Set => {
                    pin.set_high().ok().unwrap();
                }
                BitState::Clear => {
                    pin.set_low().ok().unwrap();
                }
            });
    }

    fn fetch_bits(&mut self) -> u8 {
        self.db_pins
            .iter_mut()
            .enumerate()
            // use .fold() to change same value in different iteration
            .fold(0u8, |mut acc, (index, pin)| {
                // in open drain mode, set pin high to release control
                pin.set_high().ok().unwrap();
                // it's incorrect to use .get_state() here, which return what we want to put pin in, rather what pin real state
                match pin.is_low() {
                    Ok(val) => match val {
                        false => acc.set_bit(index as u8),
                        true => acc.clear_bit(index as u8),
                    },
                    Err(_) => panic!("Something wrong when read from pin"),
                }
                acc
            })
    }
}
