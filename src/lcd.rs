use embedded_hal::delay::DelayNs;

use crate::{
    command::{
        CommandSet, DataWidth, Font, LineMode, MoveDirection, RAMType, SendCommand, ShiftType,
        State,
    },
    state::LcdState,
};

pub struct Lcd<'a, 'b, Sender: SendCommand, Delayer: DelayNs> {
    sender: &'a mut Sender,
    delayer: &'b mut Delayer,
    state: LcdState,
    poll_interval_us: u32,
}

impl<'a, 'b, Sender: SendCommand, Delayer: DelayNs> Lcd<'a, 'b, Sender, Delayer> {
    pub(crate) fn new(
        sender: &'a mut Sender,
        delayer: &'b mut Delayer,
        state: LcdState,
        poll_interval_us: u32,
    ) -> Self {
        Self {
            sender,
            delayer,
            state,
            poll_interval_us,
        }
    }
}

impl<'a, 'b, Sender: SendCommand, Delayer: DelayNs> Lcd<'a, 'b, Sender, Delayer> {
    /// Note:
    /// Due to driver implementation, this function may have actual effect, or not
    pub fn set_backlight(&mut self, backlight: State) {
        self.sender.set_backlight(backlight);
        self.state.set_backlight(backlight);
    }

    pub fn get_backlight(self) -> State {
        self.state.get_backlight()
    }

    pub fn read_u8_from_cur(&mut self) -> u8 {
        self.sender
            .wait_and_send(
                CommandSet::ReadDataFromRAM,
                self.delayer,
                self.poll_interval_us,
            )
            .unwrap()
    }

    pub fn write_u8_to_cur(&mut self, byte: impl Into<u8>) {
        assert!(
            self.get_ram_type() == RAMType::DDRam,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        self.sender.wait_and_send(
            CommandSet::WriteDataToRAM(byte.into()),
            self.delayer,
            self.poll_interval_us,
        );

        // since AC of UT7066U will automaticlly increase, we only need to update LCD struct
        // since RAM of UT7066U is looped, we need to mimic it
        let last_pos = self.get_cursor_pos();
        let line_capacity = self.get_line_capacity();

        let raw_pos = match self.get_direction() {
            MoveDirection::RightToLeft => match self.get_line_mode() {
                LineMode::OneLine => {
                    if last_pos.0 == 0 {
                        (line_capacity - 1, 0)
                    } else {
                        (last_pos.0 - 1, 0)
                    }
                }
                LineMode::TwoLine => {
                    if last_pos.0 == 0 {
                        if last_pos.1 == 1 {
                            (line_capacity - 1, 0)
                        } else {
                            (line_capacity - 1, 1)
                        }
                    } else {
                        (last_pos.0 - 1, last_pos.1)
                    }
                }
            },
            MoveDirection::LeftToRight => match self.get_line_mode() {
                LineMode::OneLine => {
                    if last_pos.0 == line_capacity - 1 {
                        (0, 0)
                    } else {
                        (last_pos.0 + 1, 0)
                    }
                }
                LineMode::TwoLine => {
                    if last_pos.0 == line_capacity - 1 {
                        if last_pos.1 == 0 {
                            (0, 1)
                        } else {
                            (0, 0)
                        }
                    } else {
                        (last_pos.0 + 1, last_pos.1)
                    }
                }
            },
        };
        self.set_cursor_pos(raw_pos);
    }

    pub fn write_graph_to_cgram(&mut self, index: u8, graph_data: &[u8; 8]) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");

        assert!(
            graph_data.iter().all(|&line| line < 2u8.pow(5)),
            "Only lower 5 bits use to construct display"
        );

        // if DDRAM is write from right to left, then when we change to CGRAM, graph will write from lower to upper
        // we will change it to left to right, to make writing correct
        let mut direction_fliped = false;
        if self.get_direction() == MoveDirection::RightToLeft {
            self.set_direction(MoveDirection::LeftToRight);
            direction_fliped = true;
        }

        let cgram_data_addr_start = index.checked_shl(3).unwrap();

        self.set_cgram_addr(cgram_data_addr_start);
        graph_data.iter().for_each(|&line_data| {
            self.sender.wait_and_send(
                CommandSet::WriteDataToRAM(line_data),
                self.delayer,
                self.poll_interval_us,
            );
        });

        // if writing direction is changed, then change it back
        if direction_fliped {
            self.set_direction(MoveDirection::RightToLeft)
        }
    }

    pub fn write_graph_to_cur(&mut self, index: u8) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");
        self.write_u8_to_cur(index);
    }

    pub fn clean_display(&mut self) {
        self.sender.wait_and_send(
            CommandSet::ClearDisplay,
            self.delayer,
            self.poll_interval_us,
        );
    }

    pub fn return_home(&mut self) {
        self.sender
            .wait_and_send(CommandSet::ReturnHome, self.delayer, self.poll_interval_us);
    }

    pub fn set_line_mode(&mut self, line: LineMode) {
        self.state.set_line_mode(line);

        self.sender.wait_and_send(
            CommandSet::FunctionSet(DataWidth::Bit4, self.get_line_mode(), self.get_font()),
            self.delayer,
            self.poll_interval_us,
        );
    }

    pub fn get_line_mode(&self) -> LineMode {
        self.state.get_line_mode()
    }

    pub fn set_font(&mut self, font: Font) {
        self.state.set_font(font);

        self.sender.wait_and_send(
            CommandSet::FunctionSet(DataWidth::Bit4, self.get_line_mode(), self.get_font()),
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_font(&self) -> Font {
        self.state.get_font()
    }
    pub fn set_display_state(&mut self, display: State) {
        self.state.set_display_state(display);

        self.sender.wait_and_send(
            CommandSet::DisplayOnOff {
                display: self.get_display_state(),
                cursor: self.get_cursor_state(),
                cursor_blink: self.get_cursor_blink_state(),
            },
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_display_state(&self) -> State {
        self.state.get_display_state()
    }
    pub fn set_cursor_state(&mut self, cursor: State) {
        self.state.set_cursor_state(cursor);

        self.sender.wait_and_send(
            CommandSet::DisplayOnOff {
                display: self.get_display_state(),
                cursor: self.get_cursor_state(),
                cursor_blink: self.get_cursor_blink_state(),
            },
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_cursor_state(&self) -> State {
        self.state.get_cursor_state()
    }
    pub fn get_ram_type(&self) -> RAMType {
        self.state.get_ram_type()
    }
    pub fn set_cursor_blink_state(&mut self, blink: State) {
        self.state.set_cursor_blink(blink);

        self.sender.wait_and_send(
            CommandSet::DisplayOnOff {
                display: self.get_display_state(),
                cursor: self.get_cursor_state(),
                cursor_blink: self.get_cursor_blink_state(),
            },
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_cursor_blink_state(&self) -> State {
        self.state.get_cursor_blink()
    }
    pub fn set_direction(&mut self, dir: MoveDirection) {
        self.state.set_direction(dir);

        self.sender.wait_and_send(
            CommandSet::EntryModeSet(self.get_direction(), self.get_shift_type()),
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_direction(&self) -> MoveDirection {
        self.state.get_direction()
    }
    pub fn set_shift_type(&mut self, shift: ShiftType) {
        self.state.set_shift(shift);

        self.sender.wait_and_send(
            CommandSet::EntryModeSet(self.get_direction(), self.get_shift_type()),
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_shift_type(&self) -> ShiftType {
        self.state.get_shift()
    }
    pub fn set_cursor_pos(&mut self, pos: (u8, u8)) {
        self.state.set_ram_type(RAMType::DDRam);
        self.state.set_cursor_pos(pos);

        // in one line mode, pos.1 will always keep at 0
        // in two line mode, the second line start at 0x40
        let raw_pos: u8 = pos.1 * 0x40 + pos.0;

        self.sender.wait_and_send(
            CommandSet::SetDDRAM(raw_pos),
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn set_cgram_addr(&mut self, addr: u8) {
        assert!(addr < 2u8.pow(6), "CGRAM Address overflow");

        self.state.set_ram_type(RAMType::CGRam);

        self.sender.wait_and_send(
            CommandSet::SetCGRAM(addr),
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_cursor_pos(&self) -> (u8, u8) {
        self.state.get_cursor_pos()
    }
    pub fn shift_cursor_or_display(&mut self, shift_type: ShiftType, dir: MoveDirection) {
        self.state.shift_cursor_or_display(shift_type, dir);

        self.sender.wait_and_send(
            CommandSet::CursorOrDisplayShift(shift_type, dir),
            self.delayer,
            self.poll_interval_us,
        );
    }
    pub fn get_display_offset(&self) -> u8 {
        self.state.get_display_offset()
    }

    pub fn set_poll_interval(&mut self, interval_us: u32) {
        self.poll_interval_us = interval_us;
    }

    pub fn get_poll_interval_us(&self) -> u32 {
        self.poll_interval_us
    }

    pub fn get_line_capacity(&self) -> u8 {
        self.state.get_line_capacity()
    }
}

#[cfg(feature = "LcdExt")]
impl<'a, 'b, Sender: SendCommand, Delayer: DelayNs> Lcd<'a, 'b, Sender, Delayer> {
    /// toggle entire display on and off (it doesn't toggle backlight)
    pub fn toggle_display(&mut self) {
        match self.get_display_state() {
            State::Off => self.set_display_state(State::On),
            State::On => self.set_display_state(State::Off),
        }
    }

    /// write [char] to current position
    /// In default implementation, character only support
    /// from ASCII 0x20 (white space) to ASCII 0x7D (`}`)
    pub fn write_char_to_cur(&mut self, char: char) {
        assert!(
            self.get_ram_type() == RAMType::DDRam,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        // map char out side of ASCII 0x20 and 0x7D to full rectangle
        let out_byte = match char.is_ascii() {
            true if (0x20 <= char as u8) && (char as u8 <= 0x7D) => char as u8,
            _ => 0xFF,
        };

        self.write_u8_to_cur(out_byte);
    }

    /// write string to current position
    pub fn write_str_to_cur(&mut self, str: &str) {
        str.chars().for_each(|char| self.write_char_to_cur(char));
    }

    /// write a byte to specific position
    pub fn write_byte_to_pos(&mut self, byte: impl Into<u8>, pos: (u8, u8)) {
        self.set_cursor_pos(pos);

        self.sender.wait_and_send(
            CommandSet::WriteDataToRAM(byte.into()),
            self.delayer,
            self.poll_interval_us,
        );
    }

    /// read a byte from specific position
    pub fn read_byte_from_pos(&mut self, pos: (u8, u8)) -> u8 {
        let original_pos = self.get_cursor_pos();
        self.set_cursor_pos(pos);
        let data = self.read_u8_from_cur();
        self.set_cursor_pos(original_pos);
        data
    }

    /// write a char to specific position
    pub fn write_char_to_pos(&mut self, char: char, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.write_char_to_cur(char);
    }

    /// write string to specific position
    pub fn write_str_to_pos(&mut self, str: &str, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.write_str_to_cur(str);
    }

    /// write custom graph to specific position
    pub fn write_graph_to_pos(&mut self, index: u8, pos: (u8, u8)) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");
        self.write_byte_to_pos(index, pos);
    }

    // read custom graph data from CGRAM
    pub fn read_graph_from_cgram(&mut self, index: u8) -> [u8; 8] {
        assert!(index < 8, "index too big, should less than 8");

        // convert index to cgram address
        self.set_cgram_addr(index.checked_shl(3).unwrap());

        let mut graph: [u8; 8] = [0u8; 8];

        graph
            .iter_mut()
            .for_each(|line| *line = self.read_u8_from_cur());

        graph
    }

    // change cursor position with relative offset
    pub fn offset_cursor_pos(&mut self, offset: (i8, i8)) {
        self.set_cursor_pos(
            self.state
                .calculate_pos_by_offset(self.state.get_cursor_pos(), offset),
        );
    }
}

#[cfg(feature = "LcdAnim")]
/// The style of the offset display window
pub enum MoveStyle {
    /// Always move to left
    ForceMoveLeft,
    /// Always move to right
    ForceMoveRight,
    /// Top left of display window won't cross display boundary
    NoCrossBoundary,
    /// Automatic find the shortest path
    Shortest,
}

#[cfg(feature = "LcdAnim")]
/// The flip style of split flap display
pub enum FlipStyle {
    /// Flip first character to target character, then flip next one
    Sequential,
    /// Flip all characters at once, automatically stop when the characters reach the target one
    Simultaneous,
}

#[cfg(feature = "LcdAnim")]
impl<'a, 'b, Sender: SendCommand, Delayer: DelayNs> Lcd<'a, 'b, Sender, Delayer> {
    /// Make the entire screen blink
    ///
    /// # Arguments
    ///
    /// * `count` - the number of times to blink the screen. If the value is `0`, the screen will blink endless.
    /// * `interval_us` - The interval (in microseconds) at which the screen state changes
    pub fn full_display_blink(&mut self, count: u32, interval_us: u32) {
        match count == 0 {
            true => loop {
                self.delay_us(interval_us);
                self.toggle_display();
            },
            false => {
                (0..count * 2).for_each(|_| {
                    self.delay_us(interval_us);
                    self.toggle_display();
                });
            }
        }
    }

    /// Typewriter-style display
    ///
    /// # Arguments
    ///
    /// * `str` - string to display
    /// * `delay_us` - The interval (in microseconds) of each character show up
    pub fn typewriter_write(&mut self, str: &str, delay_us: u32) {
        str.chars().for_each(|char| {
            self.delay_us(delay_us);
            self.write_char_to_cur(char);
        })
    }

    /// Split-Flap-style display
    ///
    /// # Arguments
    ///
    /// * `str` - string to display
    /// * `fs` - flip style, see [FlipStyle]
    /// * `max_flip_cnt` - The maximum number of times to flip the display before reaching the target character
    /// * `per_flip_delay_us` - The delay (in microseconds) between each flip. It is recommended to set this value to at least `100_000`.
    /// * `per_char_flip_delay_us` - Used in [FlipStyle::Sequential] mode, this is the time (in microseconds) to wait between flipping each character
    pub fn split_flap_write(
        &mut self,
        str: &str,
        fs: FlipStyle,
        max_flip_cnt: Option<u8>,
        per_flip_delay_us: u32,
        per_char_flip_delay_us: Option<u32>,
    ) {
        // Checking if all characters are suitable for split flap effect (should in ASCII 0x20 to 0x7D)
        let test_result = str
            .chars()
            .all(|char| char.is_ascii() && (0x20 <= char as u8) && (char as u8 <= 0x7D));

        assert!(test_result, "Currently only support ASCII 0x20 to 0x7D");

        let mut cursor_state_changed = false;

        // turn off cursor, since it will always shift to next position
        if self.get_cursor_state() != State::Off {
            self.set_cursor_state(State::Off);
            cursor_state_changed = true;
        }

        match fs {
            FlipStyle::Sequential => {
                assert!(
                    per_char_flip_delay_us.is_some(),
                    "Should set some per char delay in Sequential Mode"
                );
                str.chars().for_each(|char| {
                    let cur_byte = char as u8;

                    let flap_start_byte = match max_flip_cnt {
                        None => 0x20,
                        Some(max_flip_cnt) => {
                            if cur_byte - max_flip_cnt < 0x20 {
                                0x20
                            } else {
                                cur_byte - max_flip_cnt
                            }
                        }
                    };

                    let cur_pos = self.get_cursor_pos();

                    self.delay_us(per_char_flip_delay_us.unwrap());
                    (flap_start_byte..=cur_byte).for_each(|byte| {
                        self.delay_us(per_flip_delay_us);
                        self.write_byte_to_pos(byte, cur_pos);
                    });

                    self.shift_cursor_or_display(ShiftType::CursorOnly, self.get_direction());
                })
            }
            FlipStyle::Simultaneous => {
                let min_char_byte = str.chars().min().unwrap() as u8;
                let max_char_byte = str.chars().max().unwrap() as u8;
                let str_len = str.chars().count();

                let flap_start_byte = match max_flip_cnt {
                    None => 0x20,
                    Some(max_flip_cnt) => {
                        if max_char_byte - min_char_byte > max_flip_cnt {
                            min_char_byte
                        } else if max_char_byte - max_flip_cnt < 0x20 {
                            0x20
                        } else {
                            max_char_byte - max_flip_cnt
                        }
                    }
                };

                let start_pos = self.get_cursor_pos();

                (flap_start_byte..=max_char_byte).for_each(|cur_byte| {
                    self.delay_us(per_flip_delay_us);

                    str.char_indices()
                        .filter(|&(_, target_char)| cur_byte <= target_char as u8) // filter character that still need to flip
                        .for_each(|(index, _)| {
                            let cur_pos = match self.get_direction() {
                                MoveDirection::RightToLeft => self
                                    .state
                                    .calculate_pos_by_offset(start_pos, (-(index as i8), 0)),
                                MoveDirection::LeftToRight => self
                                    .state
                                    .calculate_pos_by_offset(start_pos, (index as i8, 0)),
                            };
                            self.write_byte_to_pos(cur_byte, cur_pos);
                        });
                });

                // after the flip finished, we cannot ensure cursor position (since .filter() method)
                // move cursor to string end
                let end_pos = match self.get_direction() {
                    MoveDirection::RightToLeft => self
                        .state
                        .calculate_pos_by_offset(start_pos, (-((str_len) as i8), 0)),
                    MoveDirection::LeftToRight => self
                        .state
                        .calculate_pos_by_offset(start_pos, ((str_len as i8), 0)),
                };
                self.set_cursor_pos(end_pos);
            }
        }

        // remeber to restore cursor state
        if cursor_state_changed {
            self.set_cursor_state(State::On);
        }
    }

    /// Move the display window to the specified position (measured from the upper-left corner of the display)
    ///
    /// # Arguments
    ///
    /// * `target_pos` - The target position of the display window
    /// * `ms` - The style of movement, see [MoveStyle]
    /// * `display_state_when_shift` - Whether to turn off the screen during the move
    /// * `delay_us_per_step` - The delay (in microseconds) between each step of the move
    pub fn shift_display_to_pos(
        &mut self,
        target_pos: u8,
        ms: MoveStyle,
        display_state_when_shift: State,
        delay_us_per_step: u32,
    ) {
        let before_pos = self.get_display_offset();

        // if target position is current position, just return
        if before_pos == target_pos {
            return;
        }

        let line_capacity = self.get_line_capacity();

        let before_state = self.get_display_state();

        self.set_display_state(display_state_when_shift);

        // calculate offset distance
        let (distance, direction) = match ms {
            MoveStyle::ForceMoveLeft => {
                if target_pos < before_pos {
                    (before_pos - target_pos, MoveDirection::RightToLeft)
                } else {
                    (
                        line_capacity - (target_pos - before_pos),
                        MoveDirection::RightToLeft,
                    )
                }
            }

            MoveStyle::ForceMoveRight => {
                if target_pos > before_pos {
                    (target_pos - before_pos, MoveDirection::LeftToRight)
                } else {
                    (
                        line_capacity - (before_pos - target_pos),
                        MoveDirection::LeftToRight,
                    )
                }
            }

            MoveStyle::NoCrossBoundary => {
                if target_pos > before_pos {
                    (target_pos - before_pos, MoveDirection::LeftToRight)
                } else {
                    (before_pos - target_pos, MoveDirection::RightToLeft)
                }
            }

            MoveStyle::Shortest => {
                if target_pos > before_pos {
                    if target_pos - before_pos <= line_capacity / 2 {
                        (target_pos - before_pos, MoveDirection::LeftToRight)
                    } else {
                        (
                            line_capacity - (target_pos - before_pos),
                            MoveDirection::RightToLeft,
                        )
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if before_pos - target_pos <= line_capacity / 2 {
                        (before_pos - target_pos, MoveDirection::RightToLeft)
                    } else {
                        (
                            line_capacity - (before_pos - target_pos),
                            MoveDirection::LeftToRight,
                        )
                    }
                }
            }
        };

        (0..(distance)).for_each(|_| {
            self.delay_us(delay_us_per_step);
            self.shift_cursor_or_display(ShiftType::CursorAndDisplay, direction);
        });

        // restore original display state
        self.set_display_state(before_state);
    }

    /// Wait for specified milliseconds
    pub fn delay_ms(&mut self, ms: u32) {
        self.delayer.delay_ms(ms);
    }

    /// Wait for specified microseconds
    pub fn delay_us(&mut self, us: u32) {
        self.delayer.delay_us(us)
    }
}
