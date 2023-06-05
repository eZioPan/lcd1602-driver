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
pub(crate) mod utils;

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

pub trait LCDAnimation {
    fn full_display_blink(&mut self, count: u32, change_interval_us: u32);
    fn typewriter_write(&mut self, str: &str, extra_delay_us: u32);
    fn shift_display_to_pos(
        &mut self,
        target_offset: u8,
        mt: MoveType,
        display_state_when_shift: State,
        delay_us_per_step: u32,
    );
    fn delay_ms(&mut self, ms: u32);
    fn delay_us(&mut self, us: u32);
}

pub trait LCDExt {
    fn toggle_display(&mut self);
    fn write_char_to_cur(&mut self, char: char);
    fn write_str(&mut self, str: &str);
    fn write_u8_to_pos(&mut self, byte: impl Into<u8>, pos: (u8, u8));
    fn write_char_to_pos(&mut self, char: char, pos: (u8, u8));
    fn write_custom_char_to_pos(&mut self, index: u8, pos: (u8, u8));
    fn extract_graph_from_cgram(&mut self, index: u8) -> [u8; 8];
}

pub trait LCDBasic {
    fn init_lcd(&mut self);
    fn write_u8_to_cur(&mut self, byte: impl Into<u8>);
    fn read_u8_from_cur(&mut self) -> u8;
    fn draw_graph_to_cgram(&mut self, index: u8, graph: [u8; 8]);
    fn write_custom_char_to_cur(&mut self, index: u8);
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
}

trait PinsInteraction {
    fn delay_and_send(&mut self, command: impl Into<FullCommand>, wait_ms: u32) -> Option<u8>;
    fn wait_and_send(&mut self, command: impl Into<FullCommand>) -> Option<u8>;
    fn wait_for_idle(&mut self);
    fn check_busy(&mut self) -> bool;
}