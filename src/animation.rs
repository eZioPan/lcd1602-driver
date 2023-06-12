use crate::{basic::LCDBasic, ext::LCDExt, struct_utils::StructUtils};

use super::{
    enums::basic_command::{MoveDirection, ShiftType, State},
    FlipStyle, MoveStyle,
};

/// The [LCDAnimation] trait provides methods for animating the display
pub trait LCDAnimation: LCDExt + LCDBasic + StructUtils {
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
                (0..count * 2).into_iter().for_each(|_| {
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
        max_flip_cnt: u8,
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

                    let flap_start_byte = if max_flip_cnt == 0 || cur_byte - max_flip_cnt < 0x20 {
                        0x20
                    } else {
                        cur_byte - max_flip_cnt
                    };

                    let cur_pos = self.get_cursor_pos();

                    self.delay_us(per_char_flip_delay_us.unwrap());
                    (flap_start_byte..=cur_byte).for_each(|byte| {
                        self.delay_us(per_flip_delay_us);
                        self.write_byte_to_pos(byte, cur_pos);
                    });

                    self.shift_cursor_or_display(
                        ShiftType::CursorOnly,
                        self.get_default_direction(),
                    );
                })
            }
            FlipStyle::Simultaneous => {
                let min_char_byte = str.chars().min().unwrap() as u8;
                let max_char_byte = str.chars().max().unwrap() as u8;
                let str_len = str.chars().count();

                let flap_start_byte = if max_flip_cnt == 0 {
                    0x20
                } else if max_char_byte - min_char_byte > max_flip_cnt {
                    min_char_byte
                } else if max_char_byte - max_flip_cnt < 0x20 {
                    0x20
                } else {
                    max_char_byte - max_flip_cnt
                };

                let start_pos = self.get_cursor_pos();

                (flap_start_byte..=max_char_byte).for_each(|cur_byte| {
                    self.delay_us(per_flip_delay_us);

                    str.char_indices()
                        .filter(|&(_, target_char)| cur_byte <= target_char as u8) // filter character that still need to flip
                        .for_each(|(index, _)| {
                            let cur_pos = match self.get_default_direction() {
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
                let end_pos = match self.get_default_direction() {
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
    fn delay_ms(&mut self, ms: u32);

    /// Wait for specified microseconds
    fn delay_us(&mut self, us: u32);
}
