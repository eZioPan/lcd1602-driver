/*!
# LCD 1602 Driver

Basic Usage:

1. Initialize pins:
   * Initialize three push-pull pins for **RS** / **RW** / **E** of LCD1602
   * Initialize 8 or 4 open-drain pins for DB0~DB7 or DB4~DB7 of LCD1602
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

use self::{
    command_set::{Font, LineMode, MoveDirection, ShiftType, State},
    full_command::FullCommand,
    pins::Pins,
};

pub mod builder;
pub mod command_set;
mod full_command;
mod impl_animation;
mod impl_ext;
mod impl_lcd_api;
mod impl_pin_interaction;
mod impl_struct_api;
pub mod pins;
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

#[derive(Clone, Copy, PartialEq)]
pub enum RAMType {
    DDRAM,
    CGRAM,
}

pub enum MoveType {
    ForceMoveLeft,
    ForceMoveRight,
    NoCrossBoundary,
    Shortest,
}

pub enum FlipType {
    Sequential,
    Simultaneous,
}

/// The [LCDAnimation] trait provides methods for animating the display
pub trait LCDAnimation {
    /// Make the entire screen blink
    ///
    /// # Arguments
    ///
    /// * `count` - the number of times to blink the screen. If the value is `0`, the screen will blink endless.
    /// * `interval_us` - The interval (in microseconds) at which the screen state changes
    fn full_display_blink(&mut self, count: u32, interval_us: u32);

    /// Typewriter-style display
    ///
    /// # Arguments
    ///
    /// * `str` - string to display
    /// * `delay_us` - The interval (in microseconds) of each character show up
    fn typewriter_write(&mut self, str: &str, delay_us: u32);

    /// Split-Flap-style display
    ///
    /// # Arguments
    ///
    /// * `str` - string to display
    /// * `ft` - flip type, see [FlipType]
    /// * `max_flap_cnt` - The maximum number of times to flip the display before reaching the target character
    /// * `per_flap_delay_us` - The delay (in microseconds) between each flip. It is recommended to set this value to at least `100_000`.
    /// * `per_char_flap_delay_us` - Used in [FlipType::Sequential] mode, this is the time (in microseconds) to wait between flipping each character
    fn split_flap_write(
        &mut self,
        str: &str,
        ft: FlipType,
        max_flap_cnt: u8,
        per_flap_delay_us: u32,
        per_char_flap_delay_us: Option<u32>,
    );

    /// Move the display window to the specified position (measured from the upper-left corner of the display)
    ///
    /// # Arguments
    ///
    /// * `target_pos` - The target position of the display window
    /// * `mt` - The type of movement, see [MoveType]
    /// * `display_state_when_shift` - Whether to turn off the screen during the move
    /// * `delay_us_per_step` - The delay (in microseconds) between each step of the move
    fn shift_display_to_pos(
        &mut self,
        target_pos: u8,
        mt: MoveType,
        display_state_when_shift: State,
        delay_us_per_step: u32,
    );

    /// Wait for specified milliseconds
    fn delay_ms(&mut self, ms: u32);

    /// Wait for specified microseconds
    fn delay_us(&mut self, us: u32);
}

pub trait LCDExt {
    fn toggle_display(&mut self);
    fn write_char_to_cur(&mut self, char: char);
    fn write_str(&mut self, str: &str);
    fn write_u8_to_pos(&mut self, byte: impl Into<u8>, pos: (u8, u8));
    fn read_u8_from_pos(&mut self, pos: (u8, u8)) -> u8;
    fn write_char_to_pos(&mut self, char: char, pos: (u8, u8));
    fn write_graph_to_pos(&mut self, index: u8, pos: (u8, u8));
    fn read_graph_from_cgram(&mut self, index: u8) -> [u8; 8];
    fn offset_cursor_pos(&mut self, offset: (i8, i8));
}

pub trait LCDBasic {
    fn init_lcd(&mut self);
    fn write_u8_to_cur(&mut self, byte: impl Into<u8>);
    fn read_u8_from_cur(&mut self) -> u8;
    fn write_graph_to_cgram(&mut self, index: u8, graph: &[u8; 8]);
    fn write_graph_to_cur(&mut self, index: u8);
    fn clean_display(&mut self);
    fn return_home(&mut self);
    fn set_line_mode(&mut self, line: LineMode);
    fn get_line_mode(&self) -> LineMode;
    fn set_font(&mut self, font: Font);
    fn get_font(&self) -> Font;
    fn set_display_state(&mut self, display: State);
    fn get_display_state(&self) -> State;
    fn set_cursor_state(&mut self, cursor: State);
    fn get_cursor_state(&self) -> State;
    fn get_ram_type(&self) -> RAMType;
    fn set_cursor_blink_state(&mut self, blink: State);
    fn get_cursor_blink_state(&self) -> State;
    fn set_default_direction(&mut self, dir: MoveDirection);
    fn get_default_direction(&self) -> MoveDirection;
    fn set_default_shift_type(&mut self, shift: ShiftType);
    fn get_default_shift_type(&self) -> ShiftType;
    fn set_cursor_pos(&mut self, pos: (u8, u8));
    fn set_cgram_addr(&mut self, addr: u8);
    fn get_cursor_pos(&self) -> (u8, u8);
    fn shift_cursor_or_display(&mut self, shift_type: ShiftType, dir: MoveDirection);
    fn get_display_offset(&self) -> u8;
    fn set_wait_interval_us(&mut self, interval: u32);
    fn get_wait_interval_us(&self) -> u32;
}

trait StructAPI {
    fn internal_set_line_mode(&mut self, line: LineMode);
    fn internal_set_font(&mut self, font: Font);
    fn internal_set_display_state(&mut self, display: State);
    fn internal_set_cursor_state(&mut self, cursor: State);
    fn internal_set_cursor_pos(&mut self, pos: (u8, u8));
    fn internal_set_ram_type(&mut self, ram_type: RAMType);
    fn internal_set_cursor_blink(&mut self, blink: State);
    fn internal_set_direction(&mut self, dir: MoveDirection);
    fn internal_set_shift(&mut self, shift: ShiftType);
    fn internal_set_display_offset(&mut self, offset: u8);
    fn internal_shift_cursor_or_display(&mut self, st: ShiftType, dir: MoveDirection);
    fn internal_calculate_pos_by_offset(&self, offset: (i8, i8)) -> (u8, u8);
}

trait StructUtils {
    fn calculate_pos_by_offset(&self, original_pos: (u8, u8), offset: (i8, i8)) -> (u8, u8);
}

trait PinsInteraction {
    fn delay_and_send(&mut self, command: impl Into<FullCommand>, wait_ms: u32) -> Option<u8>;
    fn wait_and_send(&mut self, command: impl Into<FullCommand>) -> Option<u8>;
    fn wait_for_idle(&mut self);
    fn check_busy(&mut self) -> bool;
}
