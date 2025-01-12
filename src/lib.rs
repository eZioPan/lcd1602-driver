/*!
# LCD 1602 Driver

Basic Usage:

1. Initialize a "sender" <br/>
    This crate include 2 driver:
    * 4-pin/8-pin parallel driver [`sender::ParallelSender`]
    * I2C driver with a separate adapter board [`sender::I2cSender`]

    You can choose either of it, or you can use any driver implemented [`sender::SendCommand`].

2. Use [`lcd::Lcd::new()`] to create a [`lcd::Lcd`], and initialize LCD1602 hardware

3. use any methods provide by [`lcd::Lcd`] to control LCD1602
*/

#![no_std]
#![warn(missing_docs)]

pub mod command;
pub mod lcd;
pub mod sender;
mod state;
pub mod utils;
