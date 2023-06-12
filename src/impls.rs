use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};

use crate::{
    animation::LCDAnimation,
    basic::LCDBasic,
    enums::basic_command::{Font, LineMode, MoveDirection, RAMType, ShiftType, State},
    ext::LCDExt,
    struct_api::StructAPI,
    struct_utils::StructUtils,
    LCD,
};

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> LCDExt
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
}

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> LCDAnimation
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    fn delay_ms(&mut self, ms: u32) {
        if ms > 0 {
            self.delayer.delay_ms(ms);
        }
    }

    fn delay_us(&mut self, us: u32) {
        if us > 0 {
            self.delayer.delay_us(us);
        }
    }
}

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> LCDBasic
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    fn get_display_offset(&self) -> u8 {
        self.display_offset
    }

    fn get_line_mode(&self) -> LineMode {
        self.line
    }

    fn get_font(&self) -> Font {
        self.font
    }

    fn get_display_state(&self) -> State {
        self.display_on
    }

    fn get_cursor_state(&self) -> State {
        self.cursor_on
    }

    fn get_cursor_blink_state(&self) -> State {
        self.cursor_blink
    }

    fn get_default_direction(&self) -> MoveDirection {
        self.direction
    }

    fn get_default_shift_type(&self) -> ShiftType {
        self.shift_type
    }

    fn get_cursor_pos(&self) -> (u8, u8) {
        assert!(
            self.get_ram_type() == RAMType::DDRAM,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        self.cursor_pos
    }

    fn set_wait_interval_us(&mut self, interval: u32) {
        self.wait_interval_us = interval
    }

    fn get_wait_interval_us(&self) -> u32 {
        self.wait_interval_us
    }

    fn get_ram_type(&self) -> RAMType {
        self.ram_type
    }
}

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> StructAPI
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    fn internal_init_lcd(&mut self) {}

    fn internal_set_line_mode(&mut self, line: LineMode) {
        assert!(
            (self.get_font() == Font::Font5x11) && (line == LineMode::OneLine),
            "font is 5x11, line cannot be 2"
        );

        self.line = line;
    }

    fn internal_set_font(&mut self, font: Font) {
        assert!(
            (self.get_line_mode() == LineMode::TwoLine) && (font == Font::Font5x8),
            "there is 2 line, font cannot be 5x11"
        );

        self.font = font;
    }

    fn internal_set_display_state(&mut self, display: State) {
        self.display_on = display;
    }

    fn internal_set_cursor_state(&mut self, cursor: State) {
        self.cursor_on = cursor;
    }

    fn internal_set_cursor_blink(&mut self, blink: State) {
        self.cursor_blink = blink;
    }

    fn internal_set_direction(&mut self, dir: MoveDirection) {
        self.direction = dir;
    }

    fn internal_set_shift(&mut self, shift: ShiftType) {
        self.shift_type = shift;
    }

    fn internal_set_cursor_pos(&mut self, pos: (u8, u8)) {
        let line_capacity = self.get_line_capacity();
        match self.line {
            LineMode::OneLine => {
                assert!(pos.0 < line_capacity, "x offset too big");
                assert!(pos.1 < 1, "always keep y as 0 on OneLine mode");
            }
            LineMode::TwoLine => {
                assert!(pos.0 < line_capacity, "x offset too big");
                assert!(pos.1 < 2, "y offset too big");
            }
        }

        self.cursor_pos = pos;
    }

    fn internal_set_display_offset(&mut self, offset: u8) {
        if offset >= self.get_line_capacity() {
            match self.get_line_mode() {
                LineMode::OneLine => panic!("Display Offset too big, should not bigger than 79"),
                LineMode::TwoLine => panic!("Display Offset too big, should not bigger than 39"),
            }
        }

        self.display_offset = offset;
    }

    fn internal_shift_cursor_or_display(&mut self, st: ShiftType, dir: MoveDirection) {
        let cur_display_offset = self.get_display_offset();
        let cur_cursor_pos = self.get_cursor_pos();
        let line_capacity = self.get_line_capacity();

        match st {
            ShiftType::CursorOnly => match dir {
                MoveDirection::LeftToRight => match self.get_line_mode() {
                    LineMode::OneLine => {
                        if cur_cursor_pos.0 == line_capacity - 1 {
                            self.internal_set_cursor_pos((0, 0));
                        } else {
                            self.internal_set_cursor_pos((cur_cursor_pos.0 + 1, 0));
                        }
                    }
                    LineMode::TwoLine => {
                        if cur_cursor_pos.0 == line_capacity - 1 {
                            if cur_cursor_pos.1 == 0 {
                                self.internal_set_cursor_pos((0, 1));
                            } else {
                                self.internal_set_cursor_pos((0, 0));
                            }
                        } else {
                            self.internal_set_cursor_pos((cur_cursor_pos.0 + 1, cur_cursor_pos.1));
                        }
                    }
                },
                MoveDirection::RightToLeft => match self.get_line_mode() {
                    LineMode::OneLine => {
                        if cur_cursor_pos.0 == 0 {
                            self.internal_set_cursor_pos((line_capacity - 1, 0));
                        } else {
                            self.internal_set_cursor_pos((cur_cursor_pos.0 - 1, 0));
                        }
                    }
                    LineMode::TwoLine => {
                        if cur_cursor_pos.0 == 0 {
                            if cur_cursor_pos.1 == 0 {
                                self.internal_set_cursor_pos((line_capacity - 1, 1));
                            } else {
                                self.internal_set_cursor_pos((line_capacity - 1, 0));
                            }
                        } else {
                            self.internal_set_cursor_pos((cur_cursor_pos.0 - 1, cur_cursor_pos.1));
                        }
                    }
                },
            },
            ShiftType::CursorAndDisplay => match dir {
                MoveDirection::LeftToRight => {
                    if cur_display_offset == line_capacity - 1 {
                        self.internal_set_display_offset(0)
                    } else {
                        self.internal_set_display_offset(cur_display_offset + 1)
                    };
                }
                MoveDirection::RightToLeft => {
                    if cur_display_offset == 0 {
                        self.internal_set_display_offset(line_capacity - 1)
                    } else {
                        self.internal_set_display_offset(cur_display_offset - 1)
                    }
                }
            },
        }
    }

    fn internal_set_ram_type(&mut self, ram_type: RAMType) {
        self.ram_type = ram_type;
    }

    fn internal_calculate_pos_by_offset(&self, offset: (i8, i8)) -> (u8, u8) {
        self.calculate_pos_by_offset(self.get_cursor_pos(), offset)
    }
}

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> StructUtils
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    fn calculate_pos_by_offset(&self, original_pos: (u8, u8), offset: (i8, i8)) -> (u8, u8) {
        let line_capacity = self.get_line_capacity();
        match self.get_line_mode() {
            LineMode::OneLine => {
                assert!(
                    (offset.0.abs() as u8) < line_capacity,
                    "x offset too big, should greater than -80 and less than 80"
                );
                assert!(offset.1 == 0, "y offset should always be 0 on OneLine Mode")
            }
            LineMode::TwoLine => {
                assert!(
                    (offset.0.abs() as u8) < line_capacity,
                    "x offset too big, should greater than -40 and less than 40"
                );
                assert!(
                    offset.1.abs() < 2,
                    "y offset too big, should between -1 and 1"
                )
            }
        }

        match self.get_line_mode() {
            LineMode::OneLine => {
                let raw_x_pos = (original_pos.0 as i8) + offset.0;
                if raw_x_pos < 0 {
                    ((raw_x_pos + line_capacity as i8) as u8, 0)
                } else if raw_x_pos > line_capacity as i8 {
                    ((raw_x_pos - line_capacity as i8) as u8, 0)
                } else {
                    (raw_x_pos as u8, 0)
                }
            }
            LineMode::TwoLine => {
                let mut x_overflow: i8 = 0;

                // this likes a "adder" in logic circuit design

                let mut raw_x_pos = (original_pos.0 as i8) + offset.0;

                if raw_x_pos < 0 {
                    raw_x_pos += 2;
                    x_overflow = -1;
                } else if raw_x_pos > line_capacity as i8 {
                    raw_x_pos -= 2;
                    x_overflow = 1;
                }

                let mut raw_y_pos = (original_pos.1 as i8) + offset.1 + x_overflow;
                if raw_y_pos < 0 {
                    raw_y_pos += 2
                } else if raw_y_pos > 2 {
                    raw_y_pos -= 2
                };

                (raw_x_pos as u8, raw_y_pos as u8)
            }
        }
    }
}
