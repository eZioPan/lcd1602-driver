use crate::command::{DataWidth, Font, LineMode, MoveDirection, RAMType, ShiftType, State};

#[derive(Default)]
pub(crate) struct LcdState {
    data_width: DataWidth,
    line: LineMode,
    font: Font,
    display_on: State,
    cursor_on: State,
    cursor_blink: State,
    direction: MoveDirection,
    shift_type: ShiftType,
    cursor_pos: (u8, u8),
    display_offset: u8,
    ram_type: RAMType,
    backlight: State,
}

impl LcdState {
    pub(crate) fn get_backlight(&self) -> State {
        self.backlight
    }

    pub(crate) fn set_backlight(&mut self, backlight: State) {
        self.backlight = backlight;
    }

    pub(crate) fn get_data_width(&self) -> DataWidth {
        self.data_width
    }

    pub(crate) fn set_data_width(&mut self, data_width: DataWidth) {
        self.data_width = data_width;
    }

    pub(crate) fn get_line_mode(&self) -> LineMode {
        self.line
    }

    pub(crate) fn set_line_mode(&mut self, line: LineMode) {
        assert!(
            (self.get_font() == Font::Font5x11) && (line == LineMode::OneLine),
            "font is 5x11, line cannot be 2"
        );

        self.line = line;
    }

    pub(crate) fn get_line_capacity(&self) -> u8 {
        match self.get_line_mode() {
            LineMode::OneLine => 80,
            LineMode::TwoLine => 40,
        }
    }

    pub(crate) fn get_font(&self) -> Font {
        self.font
    }

    pub(crate) fn set_font(&mut self, font: Font) {
        assert!(
            (self.get_line_mode() == LineMode::TwoLine) && (font == Font::Font5x8),
            "there is 2 line, font cannot be 5x11"
        );

        self.font = font;
    }

    pub(crate) fn get_display_state(&self) -> State {
        self.display_on
    }

    pub(crate) fn set_display_state(&mut self, display: State) {
        self.display_on = display;
    }

    pub(crate) fn get_cursor_state(&self) -> State {
        self.cursor_on
    }

    pub(crate) fn set_cursor_state(&mut self, cursor: State) {
        self.cursor_on = cursor;
    }

    pub(crate) fn get_cursor_blink(&self) -> State {
        self.cursor_blink
    }

    pub(crate) fn set_cursor_blink(&mut self, blink: State) {
        self.cursor_blink = blink;
    }

    pub(crate) fn get_direction(&self) -> MoveDirection {
        self.direction
    }

    pub(crate) fn set_direction(&mut self, dir: MoveDirection) {
        self.direction = dir;
    }

    pub(crate) fn get_shift_type(&self) -> ShiftType {
        self.shift_type
    }

    pub(crate) fn set_shift_type(&mut self, shift: ShiftType) {
        self.shift_type = shift;
    }

    pub(crate) fn get_cursor_pos(&self) -> (u8, u8) {
        assert!(
            self.get_ram_type() == RAMType::DDRam,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        self.cursor_pos
    }

    pub(crate) fn set_cursor_pos(&mut self, pos: (u8, u8)) {
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

    pub(crate) fn get_display_offset(&self) -> u8 {
        self.display_offset
    }

    pub(crate) fn set_display_offset(&mut self, offset: u8) {
        if offset >= self.get_line_capacity() {
            match self.get_line_mode() {
                LineMode::OneLine => panic!("Display Offset too big, should not bigger than 79"),
                LineMode::TwoLine => panic!("Display Offset too big, should not bigger than 39"),
            }
        }

        self.display_offset = offset;
    }

    pub(crate) fn shift_cursor_or_display(&mut self, st: ShiftType, dir: MoveDirection) {
        let cur_display_offset = self.get_display_offset();
        let cur_cursor_pos = self.get_cursor_pos();
        let line_capacity = self.get_line_capacity();

        match st {
            ShiftType::CursorOnly => match dir {
                MoveDirection::LeftToRight => match self.get_line_mode() {
                    LineMode::OneLine => {
                        if cur_cursor_pos.0 == line_capacity - 1 {
                            self.set_cursor_pos((0, 0));
                        } else {
                            self.set_cursor_pos((cur_cursor_pos.0 + 1, 0));
                        }
                    }
                    LineMode::TwoLine => {
                        if cur_cursor_pos.0 == line_capacity - 1 {
                            if cur_cursor_pos.1 == 0 {
                                self.set_cursor_pos((0, 1));
                            } else {
                                self.set_cursor_pos((0, 0));
                            }
                        } else {
                            self.set_cursor_pos((cur_cursor_pos.0 + 1, cur_cursor_pos.1));
                        }
                    }
                },
                MoveDirection::RightToLeft => match self.get_line_mode() {
                    LineMode::OneLine => {
                        if cur_cursor_pos.0 == 0 {
                            self.set_cursor_pos((line_capacity - 1, 0));
                        } else {
                            self.set_cursor_pos((cur_cursor_pos.0 - 1, 0));
                        }
                    }
                    LineMode::TwoLine => {
                        if cur_cursor_pos.0 == 0 {
                            if cur_cursor_pos.1 == 0 {
                                self.set_cursor_pos((line_capacity - 1, 1));
                            } else {
                                self.set_cursor_pos((line_capacity - 1, 0));
                            }
                        } else {
                            self.set_cursor_pos((cur_cursor_pos.0 - 1, cur_cursor_pos.1));
                        }
                    }
                },
            },
            ShiftType::CursorAndDisplay => match dir {
                MoveDirection::LeftToRight => {
                    if cur_display_offset == line_capacity - 1 {
                        self.set_display_offset(0)
                    } else {
                        self.set_display_offset(cur_display_offset + 1)
                    };
                }
                MoveDirection::RightToLeft => {
                    if cur_display_offset == 0 {
                        self.set_display_offset(line_capacity - 1)
                    } else {
                        self.set_display_offset(cur_display_offset - 1)
                    };
                }
            },
        }
    }

    pub(crate) fn get_ram_type(&self) -> RAMType {
        self.ram_type
    }

    pub(crate) fn set_ram_type(&mut self, ram_type: RAMType) {
        self.ram_type = ram_type;
    }

    pub(crate) fn calculate_pos_by_offset(
        &self,
        original_pos: (u8, u8),
        offset: (i8, i8),
    ) -> (u8, u8) {
        let line_capacity = self.get_line_capacity();

        match self.get_line_mode() {
            LineMode::OneLine => {
                assert!(
                    offset.0.unsigned_abs() < line_capacity,
                    "x offset too big, should greater than -80 and less than 80"
                );
                assert!(offset.1 == 0, "y offset should always be 0 on OneLine Mode")
            }
            LineMode::TwoLine => {
                assert!(
                    offset.0.unsigned_abs() < line_capacity,
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
