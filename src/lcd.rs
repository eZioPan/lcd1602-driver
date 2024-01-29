//! [`Lcd`] is the main driver for LCD1602

use embedded_hal::delay::DelayNs;

use crate::{
    command::{Font, LineMode, MoveDirection, RAMType, ShiftType, State},
    state::LcdState,
};

mod init;

pub use init::Config;

mod impls;

/// [`Lcd`] is the main struct to drive a LCD1602
pub struct Lcd<'a, 'b, Sender, Delayer>
where
    Delayer: DelayNs,
{
    sender: &'a mut Sender,
    delayer: &'b mut Delayer,
    state: LcdState,
    poll_interval_us: u32,
}

/// All basic command to control LCD1602
#[allow(missing_docs)]
pub trait Basic {
    fn read_u8_from_cur(&mut self) -> u8;

    fn write_u8_to_cur(&mut self, byte: u8);

    fn write_graph_to_cgram(&mut self, index: u8, graph_data: &[u8; 8]);

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

    fn set_direction(&mut self, dir: MoveDirection);

    fn get_direction(&self) -> MoveDirection;

    fn set_shift_type(&mut self, shift: ShiftType);

    fn get_shift_type(&self) -> ShiftType;

    fn set_cursor_pos(&mut self, pos: (u8, u8));

    fn set_cgram_addr(&mut self, addr: u8);

    fn get_cursor_pos(&self) -> (u8, u8);

    fn shift_cursor_or_display(&mut self, shift_type: ShiftType, dir: MoveDirection);

    fn get_display_offset(&self) -> u8;

    fn set_poll_interval(&mut self, interval_us: u32);

    fn get_poll_interval_us(&self) -> u32;

    fn get_line_capacity(&self) -> u8;

    /// Note:
    /// Due to driver implementation, this function may have actual effect, or not
    fn set_backlight(&mut self, backlight: State);

    fn get_backlight(self) -> State;

    fn calculate_pos_by_offset(&self, start: (u8, u8), offset: (i8, i8)) -> (u8, u8);

    /// Wait for specified milliseconds
    fn delay_ms(&mut self, ms: u32);

    /// Wait for specified microseconds
    fn delay_us(&mut self, us: u32);
}

/// Useful command to control LCD1602
pub trait Ext: Basic {
    /// toggle entire display on and off (it doesn't toggle backlight)
    fn toggle_display(&mut self) {
        match self.get_display_state() {
            State::Off => self.set_display_state(State::On),
            State::On => self.set_display_state(State::Off),
        }
    }

    /// write [char] to current position
    /// In default implementation, character only support
    /// from ASCII 0x20 (white space) to ASCII 0x7D (`}`)
    fn write_char_to_cur(&mut self, char: char) {
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
    fn write_str_to_cur(&mut self, str: &str) {
        str.chars().for_each(|char| self.write_char_to_cur(char));
    }

    /// write a byte to specific position
    fn write_byte_to_pos(&mut self, byte: u8, pos: (u8, u8)) {
        self.set_cursor_pos(pos);

        self.write_u8_to_cur(byte);
    }

    /// read a byte from specific position
    fn read_byte_from_pos(&mut self, pos: (u8, u8)) -> u8 {
        let original_pos = self.get_cursor_pos();
        self.set_cursor_pos(pos);
        let data = self.read_u8_from_cur();
        self.set_cursor_pos(original_pos);
        data
    }

    /// write a char to specific position
    fn write_char_to_pos(&mut self, char: char, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.write_char_to_cur(char);
    }

    /// write string to specific position
    fn write_str_to_pos(&mut self, str: &str, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.write_str_to_cur(str);
    }

    /// write custom graph to specific position
    fn write_graph_to_pos(&mut self, index: u8, pos: (u8, u8)) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");
        self.write_byte_to_pos(index, pos);
    }

    /// read custom graph data from CGRAM
    fn read_graph_from_cgram(&mut self, index: u8) -> [u8; 8] {
        assert!(index < 8, "index too big, should less than 8");

        // convert index to cgram address
        self.set_cgram_addr(index.checked_shl(3).unwrap());

        let mut graph: [u8; 8] = [0u8; 8];

        graph
            .iter_mut()
            .for_each(|line| *line = self.read_u8_from_cur());

        graph
    }

    /// change cursor position with relative offset
    fn offset_cursor_pos(&mut self, offset: (i8, i8)) {
        self.set_cursor_pos(self.calculate_pos_by_offset(self.get_cursor_pos(), offset));
    }
}

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

/// The flip style of split flap display
pub enum FlipStyle {
    /// Flip first character to target character, then flip next one
    Sequential,
    /// Flip all characters at once, automatically stop when the characters reach the target one
    Simultaneous,
}

/// Show animation on LCD1602
pub trait Anim: Ext {
    /// Make the entire screen blink
    ///
    /// # Arguments
    ///
    /// * `count` - the number of times to blink the screen. If the value is `0`, the screen will blink endless.
    /// * `interval_us` - The interval (in microseconds) at which the screen state changes
    fn full_display_blink(&mut self, count: u32, interval_us: u32) {
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
    fn typewriter_write(&mut self, str: &str, delay_us: u32) {
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
    fn split_flap_write(
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
                                MoveDirection::RightToLeft => {
                                    self.calculate_pos_by_offset(start_pos, (-(index as i8), 0))
                                }
                                MoveDirection::LeftToRight => {
                                    self.calculate_pos_by_offset(start_pos, (index as i8, 0))
                                }
                            };
                            self.write_byte_to_pos(cur_byte, cur_pos);
                        });
                });

                // after the flip finished, we cannot ensure cursor position (since .filter() method)
                // move cursor to string end
                let end_pos = match self.get_direction() {
                    MoveDirection::RightToLeft => {
                        self.calculate_pos_by_offset(start_pos, (-((str_len) as i8), 0))
                    }
                    MoveDirection::LeftToRight => {
                        self.calculate_pos_by_offset(start_pos, ((str_len as i8), 0))
                    }
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
    fn shift_display_to_pos(
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
}
