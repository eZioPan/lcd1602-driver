//! the builder pattern to build a [LCD]

use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};

mod impl_builder_api;

use super::{
    enums::basic_command::{Font, LineMode, MoveDirection, ShiftType, State},
    pins::Pins,
    LCD,
};

/// struct which collect infomation to build [LCD][crate::LCD] and initialize LCD1602
pub struct Builder<ControlPin, DBPin, const PIN_CNT: usize, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    pins: Option<Pins<ControlPin, DBPin, PIN_CNT>>,
    delayer: Option<Delayer>,
    line: LineMode,
    font: Font,
    display_on: State,
    cursor_on: State,
    cursor_blink: State,
    dir: MoveDirection,
    shift_type: ShiftType,
    wait_interval_us: u32,
}

pub trait BuilderAPI<ControlPin, DBPin, const PIN_CNT: usize, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    /// combine [Pins] and delay timer into a [Builder]
    fn new(pins: Pins<ControlPin, DBPin, PIN_CNT>, delayer: Delayer) -> Self;
    /// consume [Builder], build a [LCD][crate::LCD] and initialize LCD1602
    fn build_and_init(self) -> LCD<ControlPin, DBPin, PIN_CNT, Delayer>;
    /// when LCD1602 report busy state, wait some time before poll the state again
    /// this function set interval (in microseconds) between 2 polls
    fn set_wait_interval_us(self, interval: u32) -> Self;
    /// when LCD1602 report busy state, wait some time before poll the state again
    /// this function get interval value (in microseconds)
    fn get_wait_interval_us(&self) -> u32;
    fn pop_pins(&mut self) -> Pins<ControlPin, DBPin, PIN_CNT>;
    fn pop_delayer(&mut self) -> Delayer;
    fn set_line(self, line: LineMode) -> Self;
    fn get_line(&self) -> LineMode;
    fn set_font(self, font: Font) -> Self;
    fn get_font(&self) -> Font;
    fn set_display(self, display: State) -> Self;
    fn get_display(&self) -> State;
    fn set_cursor(self, cursor: State) -> Self;
    fn get_cursor(&self) -> State;
    fn set_blink(self, blink: State) -> Self;
    fn get_blink(&self) -> State;
    fn set_direction(self, dir: MoveDirection) -> Self;
    fn get_direction(&self) -> MoveDirection;
    fn set_shift(self, shift: ShiftType) -> Self;
    fn get_shift(&self) -> ShiftType;
}
