//! # I2C adapter board driver
//!
//! This adapter board has a I2C interface to MCU,  
//! and it's 8 bi-directional pin P7 to P0 is attach to following pin of LCD1602
//!
//! P7 -> P0  
//! DB7/DB6/DB5/DB4/BL/CS(EN)/RW/RS
//!
//! Since there are only 4 pin for DB pin, so it only support 4 bit data width command.
//!
//! LCD Command is first splitted into 4-bit length command(s), then appending BL/CS(EN)/RW/RS bits to [`I2cCommand`].  
//! [`I2cCommand`] containers a one or two u8 data, depending on original LCD Command length.  
//! But [`I2cCommand`] is still not suitable for controlling LCD, one u8 data should be send as ENABLE pin "disable" - "enable" - "disable" sequence,  
//! to make sure LCD read our command precisely.  
//! Thus [`I2cCommand`] will further convert to a three u8 data sequence or six u8 data sequence [`I2cRawSeq`] .

use embedded_hal_async::i2c::{AddressMode, I2c};
use embedded_hal_async::delay::DelayNs;

use crate::{
    command::{Bits, Command, ReadWriteOp, RegisterSelection, State},
    utils::{BitOps, BitState},
};

use super::SendCommand;

/// [`I2cSender`] is the I2C interface with an adapter board to drive LCD1602
pub struct I2cSender<'a, I2cLcd, A>
where
    I2cLcd: I2c<A>,
    A: AddressMode + Clone,
{
    i2c: &'a mut I2cLcd,
    addr: A,
    first_command: bool,
    backlight_state: State,
}

impl<'a, I2cLcd, A> I2cSender<'a, I2cLcd, A>
where
    I2cLcd: I2c<A>,
    A: AddressMode + Clone,
{
    /// Create a [`I2cSender`] driver
    pub fn new(i2c: &'a mut I2cLcd, addr: A) -> Self {
        Self {
            i2c,
            addr,
            first_command: true,
            backlight_state: State::Off,
        }
    }
}

impl<'a, I2cLcd, A, Delayer> SendCommand<Delayer, true> for I2cSender<'a, I2cLcd, A>
where
    I2cLcd: I2c<A>,
    A: AddressMode + Clone,
    Delayer: DelayNs,
{
    // set backlight should ALWAYS keep ENABLE pin at low voltage state
    async fn set_actual_backlight(&mut self, state: State) {
        // PCF8574T use weak pull up as high voltage,
        // thus only ENABLE pin should be strictly set to low voltage.
        let mut i2c_raw_seq: u8 = 0b1111_1011;

        match state {
            State::On => i2c_raw_seq.set_bit(3),
            State::Off => i2c_raw_seq.clear_bit(3),
        };

        self.i2c
            .write(self.addr.clone(), &[i2c_raw_seq])
            .await
            .expect("Failed to write backlight state to I2C");
        <Self as SendCommand<Delayer, true>>::yield_now(self).await;
        self.backlight_state = state;
    }

    async fn get_actual_backlight(&mut self) -> State {
        let mut buf = [0u8];
        // just a read is sufficient get backlight state
        self.i2c
            .read(self.addr.clone(), &mut buf)
            .await
            .expect("Failed to read backlight state from I2C");
        <Self as SendCommand<Delayer, true>>::yield_now(self).await;
        match buf[0].check_bit(3) {
            BitState::Clear => State::Off,
            BitState::Set => State::On,
        }
    }

    async fn send(&mut self, lcd_command: Command) -> Option<u8> {
        if self.first_command {
            assert!(
                lcd_command.get_data().is_some(),
                "first command should has some data to write"
            );

            match lcd_command.get_data().expect("First command data missing") {
                Bits::Bit8(_) => panic!("first command should be 4 bit"),

                Bits::Bit4(_) => {
                    let i2c_command = I2cCommand::gen_i2c_cmd(lcd_command, self.backlight_state);

                    assert!(
                        i2c_command.0 == 0b0010_0000 || i2c_command.0 == 0b0010_1000,
                        "first command should be Function set, and should set to 4 bit mode"
                    );

                    let I2cRawSeq(len, raw_seq) = i2c_command.into();

                    self.i2c
                        .write(self.addr.clone(), &raw_seq[0..len as usize])
                        .await
                        .expect("Failed to write first command to I2C");
                    <Self as SendCommand<Delayer, true>>::yield_now(self).await;
                }
            }

            self.first_command = false;
        } else {
            // if not first command, then all command should have 8 bit length
            // though we send it as 4 bit per group
            if let Some(Bits::Bit4(_)) = lcd_command.get_data() {
                panic!("Only first command is 4 bit long, other command should be 8 bit long")
            }

            match lcd_command.get_read_write_op() {
                ReadWriteOp::Write => {
                    assert!(
                        lcd_command.get_data().is_some(),
                        "first command should has some data to write"
                    );

                    if lcd_command.get_register_selection() == RegisterSelection::Command {
                        match lcd_command.get_data().expect("Command data missing") {
                            Bits::Bit8(lcd_command_data) => {
                                if (lcd_command_data >> 4) == 0b0011 {
                                    panic!("This I2C driver doesn't support 8 bit Data Width Mode")
                                }
                            }
                            _ => unreachable!(),
                        }
                    }

                    let i2c_command = I2cCommand::gen_i2c_cmd(lcd_command, self.backlight_state);
                    let I2cRawSeq(len, raw_seq) = i2c_command.into();
                    self.i2c
                        .write(self.addr.clone(), &raw_seq[0..len as usize])
                        .await
                        .expect("Failed to write command to I2C");
                    <Self as SendCommand<Delayer, true>>::yield_now(self).await;
                }

                ReadWriteOp::Read => {
                    let mut concat_buf = [0u8; 2];
                    let mut buf = [0u8];

                    let i2c_command = I2cCommand::gen_i2c_cmd(lcd_command, self.backlight_state);
                    let I2cRawSeq(len, raw_seq) = i2c_command.into();

                    assert_eq!(
                        len, 6,
                        "Read command will always need to send 6 I2C sequence"
                    );

                    // The following read-write step is
                    //
                    // Send the command ("disable" - "enable", and keep "enable" state), but not sending ending "disable", read the first 4-bit data from I2C.
                    // Send the ending "disable" of the command, then start to sending the command again, to fetch last 4-bit data from I2C.
                    // And finally send the "disable" command to end entire seq.

                    self.i2c
                        .write_read(self.addr.clone(), &raw_seq[0..2], &mut buf)
                        .await
                        .expect("Failed to write_read I2C (first half)");
                    <Self as SendCommand<Delayer, true>>::yield_now(self).await;
                    concat_buf[0] = buf[0];

                    self.i2c
                        .write_read(self.addr.clone(), &raw_seq[2..5], &mut buf)
                        .await
                        .expect("Failed to write_read I2C (second half)");
                    <Self as SendCommand<Delayer, true>>::yield_now(self).await;

                    self.i2c
                        .write(self.addr.clone(), &raw_seq[5..6])
                        .await
                        .expect("Failed to write I2C (final step)");
                    <Self as SendCommand<Delayer, true>>::yield_now(self).await;
                    concat_buf[1] = buf[0];

                    // Combine 2 halves of data into a whole one.
                    return Some((concat_buf[0] & 0b1111_0000) | (concat_buf[1] >> 4));
                }
            };
        }

        None
    }
}

// I2cCommand is the command that convert from LCD1602 Command to I2C adapter board command.
//
// The first I2C data is always used.(thus a u8)
// In most time the second I2C data is used, only the first command sent to LCD doesn't have the second part.(thus a Option<u8>)
struct I2cCommand(u8, Option<u8>);

impl Default for I2cCommand {
    fn default() -> Self {
        Self(0, Some(0))
    }
}

impl I2cCommand {
    fn gen_i2c_cmd(lcd_command: Command, backlight_state: State) -> Self {
        let mut i2c_raw_data = I2cCommand::default();
        let i2c_raw_data_1_inner = i2c_raw_data
            .1
            .as_mut()
            .expect("I2C command second byte missing");

        if lcd_command.get_register_selection() == RegisterSelection::Data {
            i2c_raw_data.0.set_bit(0);
            i2c_raw_data_1_inner.set_bit(0);
        }

        // will need to set bit of backlight controlling
        match backlight_state {
            State::Off => {
                //i2c_raw_data.0 &= 0b1111_0111;
                //*i2c_raw_data_1_inner &= 0b1111_0111;
                i2c_raw_data.0.clear_bit(3);
                i2c_raw_data_1_inner.clear_bit(3);
            }
            State::On => {
                //i2c_raw_data.0 |= 0b0000_1000;
                //*i2c_raw_data_1_inner &= 0b0000_1000;
                i2c_raw_data.0.set_bit(3);
                i2c_raw_data_1_inner.set_bit(3);
            }
        }

        match lcd_command.get_read_write_op() {
            ReadWriteOp::Write => match lcd_command.get_data() {
                None => panic!("Write command should have some data to be send"),
                Some(command_data) => match command_data {
                    Bits::Bit4(raw_data) => {
                        assert!(raw_data < (1 << 4), "data is overflow 4 bit");
                        i2c_raw_data.0 |= raw_data << 4;
                        i2c_raw_data.1 = None;
                    }
                    Bits::Bit8(raw_data) => {
                        i2c_raw_data.0 |= raw_data & 0b1111_0000;
                        *i2c_raw_data_1_inner |= (raw_data & 0b0000_1111) << 4
                    }
                },
            },
            ReadWriteOp::Read => {
                // A Read will need to send same LCD command twice
                i2c_raw_data.0.set_bit(1);
                i2c_raw_data.0 |= 0b1111 << 4; // This actually makes P4~P7 of PCF8574 weak pull up, to read data in

                i2c_raw_data_1_inner.set_bit(1);
                *i2c_raw_data_1_inner |= 0b1111 << 4; // This actually makes P4~P7 of PCF8574 weak pull up, to read data in
            }
        }

        i2c_raw_data
    }
}

// This is the actual I2C sequence will be sending by us.
struct I2cRawSeq(u8, [u8; 6]);

impl From<I2cCommand> for I2cRawSeq {
    // split each part of I2cCommand into "disable" - "enable" - "disable" seq
    fn from(i2c_command: I2cCommand) -> Self {
        let mut seq = [0u8; 6];
        let mut len = 3;

        let mut disable_0 = i2c_command.0;
        disable_0.clear_bit(2);
        let mut enable_0 = disable_0;
        enable_0.set_bit(2);

        seq[0] = disable_0;
        seq[1] = enable_0;
        seq[2] = disable_0;

        if let Some(disable_1_val) = i2c_command.1 {
            let mut disable_1 = disable_1_val;
            disable_1.clear_bit(2);
            let mut enable_1 = disable_1;
            enable_1.set_bit(2);

            seq[3] = disable_1;
            seq[4] = enable_1;
            seq[5] = disable_1;

            len = 6;
        }

        Self(len, seq)
    }
}
