use embedded_hal::i2c::{AddressMode, I2c};

use crate::{
    command::{Bits, Command, ReadWriteOp, RegisterSelection, SendCommand, State},
    utils::{BitOps, BitState},
};

// I2C to parallel:
// P7 -> P0
// DB7/DB6/DB5/DB4/BL/CS/RW/RS

pub struct I2cSender<'a, I2cLcd: I2c<A>, A: AddressMode + Clone> {
    i2c: &'a mut I2cLcd,
    addr: A,
    first_command: bool,
}

impl<'a, I2cLcd: I2c<A>, A: AddressMode + Clone> I2cSender<'a, I2cLcd, A> {
    pub fn new(i2c: &'a mut I2cLcd, addr: A) -> Self {
        Self {
            i2c,
            addr,
            first_command: true,
        }
    }
}

impl<'a, I2cLcd: I2c<A>, A: AddressMode + Clone> SendCommand for I2cSender<'a, I2cLcd, A> {
    fn set_backlight(&mut self, state: State) {
        let mut disabled_command: u8 = 0b1111_0010;

        if state == State::On {
            disabled_command.set_bit(3);
        }

        let mut enabled_command = disabled_command;
        enabled_command.set_bit(2);

        let seq = [disabled_command, enabled_command, disabled_command];

        self.i2c.write(self.addr.clone(), &seq).unwrap();
    }

    fn get_backlight(&mut self) -> State {
        let mut buf = [0u8];
        // just a read is sufficient get backlight state
        self.i2c.read(self.addr.clone(), &mut buf).unwrap();
        match buf[0].check_bit(3) {
            BitState::Clear => State::Off,
            BitState::Set => State::On,
        }
    }

    fn send(&mut self, command_set: impl Into<Command>) -> Option<u8> {
        let command: Command = command_set.into();

        if self.first_command {
            assert!(
                command.get_data().is_some(),
                "first command should has some data to write"
            );

            match command.get_data().unwrap() {
                Bits::Bit8(_) => panic!("first command should be 4 bit"),

                Bits::Bit4(_) => {
                    let i2c_data = I2cRawData::from(command);

                    // TODO: make backlight check more sensible
                    assert!(
                        *i2c_data.0.as_ref().unwrap() == 0b0010_0000
                            || *i2c_data.0.as_ref().unwrap() == 0b0010_1000,
                        "first command should be Function set, and should set to 4 bit mode"
                    );

                    let I2cSeq(_, seq) = i2c_data.into();

                    self.i2c.write(self.addr.clone(), &seq[0..3]).unwrap();
                }
            }

            self.first_command = false;
        } else {
            // if not first command, then all command should have 8 bit length
            // though we send it as 4 bit per group
            if let Some(Bits::Bit4(_)) = command.get_data() {
                panic!("Only first command is 4 bit long, other command should be 8 bit long")
            }

            match command.get_read_write_op() {
                ReadWriteOp::Write => {
                    assert!(
                        command.get_data().is_some(),
                        "first command should has some data to write"
                    );

                    if command.get_register_selection() == RegisterSelection::Command {
                        match command.get_data().unwrap() {
                            Bits::Bit8(command_data) => {
                                if (command_data >> 4) == 0b0011 {
                                    panic!("This I2C driver doesn't support 8 bit Data Width Mode")
                                }
                            }
                            _ => unreachable!(),
                        }
                    }

                    let i2c_data = I2cRawData::from(command);
                    let I2cSeq(_, seq) = i2c_data.into();
                    self.i2c.write(self.addr.clone(), &seq).unwrap();
                }

                ReadWriteOp::Read => {
                    let mut concat_buf = [0u8; 2];
                    let mut buf = [0u8];

                    let i2c_data = I2cRawData::from(command);
                    let I2cSeq(_, seq) = i2c_data.into();

                    self.i2c
                        .write_read(self.addr.clone(), &seq[0..2], &mut buf)
                        .unwrap();
                    concat_buf[0] = buf[0];
                    self.i2c
                        .write_read(self.addr.clone(), &seq[2..5], &mut buf)
                        .unwrap();
                    self.i2c.write(self.addr.clone(), &seq[5..6]).unwrap();
                    concat_buf[1] = buf[0];

                    return Some((concat_buf[0] & 0b1111_0000) | (concat_buf[1] >> 4));
                }
            };
        }

        None
    }
}

struct I2cRawData(Option<u8>, Option<u8>);

// all I2cRawData is at disable mode
impl From<Command> for I2cRawData {
    fn from(command: Command) -> Self {
        // always "disable" data
        // TODO: make backlight turn on/off
        let mut data = [Some(0b0000_1000u8), Some(0b0000_1000u8)];

        data.iter_mut().for_each(|v| {
            if command.get_register_selection() == RegisterSelection::Data {
                *v.as_mut().unwrap() |= 1;
            }
        });

        match command.get_read_write_op() {
            ReadWriteOp::Write => match command.get_data() {
                None => panic!("Write command should have some data to be send"),
                Some(command_data) => match command_data {
                    Bits::Bit4(raw_data) => {
                        assert!(raw_data < (1 << 4), "data is overflow 4 bit");
                        *data[0].as_mut().unwrap() |= raw_data << 4;
                        data[1] = None;
                    }
                    Bits::Bit8(raw_data) => {
                        *data[0].as_mut().unwrap() |= raw_data & 0b1111_0000;
                        *data[1].as_mut().unwrap() |= (raw_data & 0b0000_1111) << 4
                    }
                },
            },
            ReadWriteOp::Read => {
                data.iter_mut().for_each(|v| {
                    *v.as_mut().unwrap() |= 1 << 1;
                    *v.as_mut().unwrap() |= 0b1111 << 4; // make PCF8574 use weak pull up, to read data in
                });
            }
        }

        I2cRawData(data[0], data[1])
    }
}

struct I2cSeq(u8, [u8; 6]);

impl From<I2cRawData> for I2cSeq {
    fn from(raw_data: I2cRawData) -> Self {
        let mut seq = [0u8; 6];
        let mut len = 3;

        let mut disable_0 = raw_data.0.unwrap();
        disable_0.clear_bit(2);
        let mut enable_0 = disable_0;
        enable_0.set_bit(2);

        seq[0] = disable_0;
        seq[1] = enable_0;
        seq[2] = disable_0;

        if raw_data.1.is_some() {
            let mut disable_1 = raw_data.1.unwrap();
            disable_1.clear_bit(2);
            let mut enable_1 = disable_1;
            enable_1.set_bit(2);

            seq[3] = disable_1;
            seq[4] = enable_1;
            seq[5] = disable_1;

            len = 6;
        }

        I2cSeq(len, seq)
    }
}
