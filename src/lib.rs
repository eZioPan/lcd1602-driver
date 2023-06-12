/*!
# LCD 1602 Driver

Basic Usage:

1. Initialize pins:
   * Initialize three push-pull pins for **RS** / **RW** / **E** of LCD1602
   * Initialize 8 or 4 **open-drain** pins for DB0~DB7 or DB4~DB7 of LCD1602
   * Initialize a delay timer(which implement [embedded_hal::blocking::delay])
2. Use [8 pin Pins::new()] or [4 pin Pins::new()] to create a [Pins] struct containing all initialized pins
3. Use the [Builder::new()] to create a [Builder] struct with [Pins] and the delay timer
4. Use the functions provided by [Builder] to configure the initial state of the LCD1602
5. Use the [.build_and_init()] to convert the [Builder] to an [LCD] struct, and initialize the LCD1602
6. Use [LCD] struct:
   * [LCDBasic] trait provides functions close to LCD1602 instructions
   * [LCDExt] trait provides commonly used **non-animation** functions
   * [LCDAnimation] trait provides simple **animation** functions


[8 pin Pins::new()]: crate::pins::EightPinsAPI::new()
[4 pin Pins::new()]: crate::pins::FourPinsAPI::new()
[Builder::new()]: crate::builder::BuilderAPI::new()
[Builder]: crate::builder::Builder
[.build_and_init()]: crate::builder::BuilderAPI::build_and_init()
*/

#![no_std]

use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};
use enums::{
    animation::{FlipStyle, MoveStyle},
    basic_command::RAMType,
};

use self::{
    enums::basic_command::{Font, LineMode, MoveDirection, ShiftType, State},
    pins::Pins,
};

mod animation;
mod basic;
pub mod builder;
pub mod command_set;
pub mod enums;
mod ext;
mod full_command;
mod impls;
mod pin_interaction;
pub mod pins;
mod struct_api;
mod struct_utils;
pub mod utils;

/// The main struct for operating the LCD1602
pub struct LCD<ControlPin, DBPin, const PIN_CNT: usize, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    pins: Pins<ControlPin, DBPin, PIN_CNT>,
    delayer: Delayer,
    line: LineMode,
    font: Font,
    display_on: State,
    cursor_on: State,
    cursor_blink: State,
    direction: MoveDirection,
    shift_type: ShiftType,
    cursor_pos: (u8, u8),
    display_offset: u8,
    wait_interval_us: u32,
    ram_type: RAMType,
}
