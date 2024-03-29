//! 4-pin/8-pin parallel interface driver

use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin, StatefulOutputPin},
};

use crate::{
    command::{Bits, Command, ReadWriteOp, RegisterSelection, State},
    utils::{BitOps, BitState},
};

use super::SendCommand;

/// [`ParallelSender`] is the parallel interface to drive LCD1602
pub struct ParallelSender<ControlPin, DBPin, BLPin, const PIN_CNT: usize>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    BLPin: StatefulOutputPin,
{
    rs_pin: ControlPin,
    rw_pin: ControlPin,
    en_pin: ControlPin,
    db_pins: [DBPin; PIN_CNT],
    bl_pin: Option<BLPin>,
}

impl<ControlPin, DBPin, BLPin> ParallelSender<ControlPin, DBPin, BLPin, 4>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    BLPin: StatefulOutputPin,
{
    /// Create 4-pin parallel driver, will need other 3 pins to control LCD,  
    /// and a optional pin to control backlight (better connect the pin to a transistor)
    #[allow(clippy::too_many_arguments)]
    pub fn new_4pin(
        rs: ControlPin,
        rw: ControlPin,
        en: ControlPin,
        db4: DBPin,
        db5: DBPin,
        db6: DBPin,
        db7: DBPin,
        bl: Option<BLPin>,
    ) -> Self {
        Self {
            rs_pin: rs,
            rw_pin: rw,
            en_pin: en,
            db_pins: [db4, db5, db6, db7],
            bl_pin: bl,
        }
    }
}

impl<ControlPin, DBPin, BLPin> ParallelSender<ControlPin, DBPin, BLPin, 8>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    BLPin: StatefulOutputPin,
{
    /// Create 8-pin parallel driver, will need other 3 pins to control LCD,  
    /// and a optional pin to control backlight (better connect the pin to a transistor)
    #[allow(clippy::too_many_arguments)]
    pub fn new_8pin(
        rs: ControlPin,
        rw: ControlPin,
        en: ControlPin,
        db0: DBPin,
        db1: DBPin,
        db2: DBPin,
        db3: DBPin,
        db4: DBPin,
        db5: DBPin,
        db6: DBPin,
        db7: DBPin,
        bl: Option<BLPin>,
    ) -> Self {
        Self {
            rs_pin: rs,
            rw_pin: rw,
            en_pin: en,
            db_pins: [db0, db1, db2, db3, db4, db5, db6, db7],
            bl_pin: bl,
        }
    }
}

impl<ControlPin, DBPin, BLPin, const PIN_CNT: usize>
    ParallelSender<ControlPin, DBPin, BLPin, PIN_CNT>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    BLPin: StatefulOutputPin,
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
                };
                acc
            })
    }
}

impl<ControlPin, DBPin, BLPin, const PIN_CNT: usize, Delayer> SendCommand<Delayer>
    for ParallelSender<ControlPin, DBPin, BLPin, PIN_CNT>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    BLPin: StatefulOutputPin,
    Delayer: DelayNs,
{
    fn get_backlight(&mut self) -> State {
        match self.bl_pin.as_mut() {
            Some(bl_pin) => match bl_pin.is_set_high().unwrap() {
                true => State::On,
                false => State::Off,
            },
            None => Default::default(),
        }
    }

    fn set_backlight(&mut self, backlight: State) {
        if let Some(bl_pin) = self.bl_pin.as_mut() {
            match backlight {
                State::Off => bl_pin.set_low().unwrap(),
                State::On => bl_pin.set_high().unwrap(),
            }
        }
    }

    fn send(&mut self, command: Command) -> Option<u8> {
        assert!(
            PIN_CNT == 4 || PIN_CNT == 8,
            "Pins other than 4 or 8 are not supported"
        );

        self.en_pin.set_low().ok().unwrap();

        match command.get_register_selection() {
            RegisterSelection::Command => {
                self.rs_pin.set_low().ok().unwrap();
            }
            RegisterSelection::Data => {
                self.rs_pin.set_high().ok().unwrap();
            }
        }

        match command.get_read_write_op() {
            ReadWriteOp::Write => {
                self.rw_pin.set_low().ok().unwrap();
            }
            ReadWriteOp::Read => {
                self.rw_pin.set_high().ok().unwrap();
            }
        }

        match command.get_read_write_op() {
            ReadWriteOp::Write => {
                let bits = command
                    .get_data()
                    .expect("Write command but no data provide");
                match PIN_CNT {
                    4 => match bits {
                        Bits::Bit4(raw_bits) => {
                            assert!(raw_bits < 2u8.pow(4), "data is greater than 4 bits");
                            self.push_bits(raw_bits);
                            self.en_pin.set_high().ok().unwrap();
                            self.en_pin.set_low().ok().unwrap();
                        }
                        Bits::Bit8(raw_bits) => {
                            self.push_bits(raw_bits >> 4);
                            self.en_pin.set_high().ok().unwrap();
                            self.en_pin.set_low().ok().unwrap();
                            self.push_bits(raw_bits & 0b1111);
                            self.en_pin.set_high().ok().unwrap();
                            self.en_pin.set_low().ok().unwrap();
                        }
                    },

                    8 => {
                        if let Bits::Bit8(raw_bits) = bits {
                            self.push_bits(raw_bits);
                            self.en_pin.set_high().ok().unwrap();
                            self.en_pin.set_low().ok().unwrap();
                        } else {
                            panic!("in 8 pin mode, data should always be 8 bit")
                        }
                    }

                    _ => unreachable!(),
                }

                None
            }
            ReadWriteOp::Read => match PIN_CNT {
                4 => {
                    self.en_pin.set_high().ok().unwrap();
                    let high_4_bits = self.fetch_bits().checked_shl(4).unwrap();
                    self.en_pin.set_low().ok().unwrap();
                    self.en_pin.set_high().ok().unwrap();
                    let low_4_bits = self.fetch_bits();
                    self.en_pin.set_low().ok().unwrap();
                    Some(high_4_bits + low_4_bits)
                }

                8 => {
                    self.en_pin.set_high().ok().unwrap();
                    let bits = self.fetch_bits();
                    self.en_pin.set_low().ok().unwrap();
                    Some(bits)
                }

                _ => unreachable!(),
            },
        }
    }
}
