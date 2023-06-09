use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::full_command::{Bits, FullCommand, FullCommandAPI, ReadWriteOp, RegisterSelection};

use super::{Pins, PinsCrateLevelAPI, PinsInternalAPI};

impl<ControlPin, DBPin, const PIN_CNT: usize> PinsCrateLevelAPI for Pins<ControlPin, DBPin, PIN_CNT>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
{
    fn send(&mut self, command: impl Into<FullCommand>) -> Option<u8> {
        assert!(
            PIN_CNT == 4 || PIN_CNT == 8,
            "Pins other than 4 or 8 are not supported"
        );

        self.en_pin.set_low().ok().unwrap();

        let command = command.into();

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
