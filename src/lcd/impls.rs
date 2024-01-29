use embedded_hal::delay::DelayNs;

use crate::command::{DataWidth, Font, LineMode, MoveDirection, RAMType, ShiftType};
use crate::sender::SendCommand;
use crate::{command::CommandSet, lcd::State};

use super::{Anim, Basic, Ext, Lcd};

impl<'a, 'b, Sender, Delayer> Basic for Lcd<'a, 'b, Sender, Delayer>
where
    Sender: SendCommand<Delayer>,
    Delayer: DelayNs,
{
    fn set_backlight(&mut self, backlight: State) {
        self.sender.set_backlight(backlight);
        self.state.set_backlight(backlight);
    }

    fn get_backlight(self) -> State {
        self.state.get_backlight()
    }

    fn read_u8_from_cur(&mut self) -> u8 {
        self.sender
            .wait_and_send(
                CommandSet::ReadDataFromRAM.into(),
                self.delayer,
                self.poll_interval_us,
            )
            .unwrap()
    }

    fn write_u8_to_cur(&mut self, byte: u8) {
        assert!(
            self.get_ram_type() == RAMType::DDRam,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        self.sender.wait_and_send(
            CommandSet::WriteDataToRAM(byte).into(),
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

    fn write_graph_to_cgram(&mut self, index: u8, graph_data: &[u8; 8]) {
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
                CommandSet::WriteDataToRAM(line_data).into(),
                self.delayer,
                self.poll_interval_us,
            );
        });

        // if writing direction is changed, then change it back
        if direction_fliped {
            self.set_direction(MoveDirection::RightToLeft)
        }
    }

    fn write_graph_to_cur(&mut self, index: u8) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");
        self.write_u8_to_cur(index);
    }

    fn clean_display(&mut self) {
        self.sender.wait_and_send(
            CommandSet::ClearDisplay.into(),
            self.delayer,
            self.poll_interval_us,
        );
    }

    fn return_home(&mut self) {
        self.sender.wait_and_send(
            CommandSet::ReturnHome.into(),
            self.delayer,
            self.poll_interval_us,
        );
    }

    fn set_line_mode(&mut self, line: LineMode) {
        self.state.set_line_mode(line);

        self.sender.wait_and_send(
            CommandSet::FunctionSet(DataWidth::Bit4, self.get_line_mode(), self.get_font()).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }

    fn get_line_mode(&self) -> LineMode {
        self.state.get_line_mode()
    }

    fn set_font(&mut self, font: Font) {
        self.state.set_font(font);

        self.sender.wait_and_send(
            CommandSet::FunctionSet(DataWidth::Bit4, self.get_line_mode(), self.get_font()).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_font(&self) -> Font {
        self.state.get_font()
    }
    fn set_display_state(&mut self, display: State) {
        self.state.set_display_state(display);

        self.sender.wait_and_send(
            CommandSet::DisplayOnOff {
                display: self.get_display_state(),
                cursor: self.get_cursor_state(),
                cursor_blink: self.get_cursor_blink_state(),
            }
            .into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_display_state(&self) -> State {
        self.state.get_display_state()
    }
    fn set_cursor_state(&mut self, cursor: State) {
        self.state.set_cursor_state(cursor);

        self.sender.wait_and_send(
            CommandSet::DisplayOnOff {
                display: self.get_display_state(),
                cursor: self.get_cursor_state(),
                cursor_blink: self.get_cursor_blink_state(),
            }
            .into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_cursor_state(&self) -> State {
        self.state.get_cursor_state()
    }
    fn get_ram_type(&self) -> RAMType {
        self.state.get_ram_type()
    }
    fn set_cursor_blink_state(&mut self, blink: State) {
        self.state.set_cursor_blink(blink);

        self.sender.wait_and_send(
            CommandSet::DisplayOnOff {
                display: self.get_display_state(),
                cursor: self.get_cursor_state(),
                cursor_blink: self.get_cursor_blink_state(),
            }
            .into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_cursor_blink_state(&self) -> State {
        self.state.get_cursor_blink()
    }
    fn set_direction(&mut self, dir: MoveDirection) {
        self.state.set_direction(dir);

        self.sender.wait_and_send(
            CommandSet::EntryModeSet(self.get_direction(), self.get_shift_type()).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_direction(&self) -> MoveDirection {
        self.state.get_direction()
    }
    fn set_shift_type(&mut self, shift: ShiftType) {
        self.state.set_shift_type(shift);

        self.sender.wait_and_send(
            CommandSet::EntryModeSet(self.get_direction(), self.get_shift_type()).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_shift_type(&self) -> ShiftType {
        self.state.get_shift_type()
    }
    fn set_cursor_pos(&mut self, pos: (u8, u8)) {
        self.state.set_ram_type(RAMType::DDRam);
        self.state.set_cursor_pos(pos);

        // in one line mode, pos.1 will always keep at 0
        // in two line mode, the second line start at 0x40
        let raw_pos: u8 = pos.1 * 0x40 + pos.0;

        self.sender.wait_and_send(
            CommandSet::SetDDRAM(raw_pos).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn set_cgram_addr(&mut self, addr: u8) {
        assert!(addr < 2u8.pow(6), "CGRAM Address overflow");

        self.state.set_ram_type(RAMType::CGRam);

        self.sender.wait_and_send(
            CommandSet::SetCGRAM(addr).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_cursor_pos(&self) -> (u8, u8) {
        self.state.get_cursor_pos()
    }
    fn shift_cursor_or_display(&mut self, shift_type: ShiftType, dir: MoveDirection) {
        self.state.shift_cursor_or_display(shift_type, dir);

        self.sender.wait_and_send(
            CommandSet::CursorOrDisplayShift(shift_type, dir).into(),
            self.delayer,
            self.poll_interval_us,
        );
    }
    fn get_display_offset(&self) -> u8 {
        self.state.get_display_offset()
    }

    fn set_poll_interval(&mut self, interval_us: u32) {
        self.poll_interval_us = interval_us;
    }

    fn get_poll_interval_us(&self) -> u32 {
        self.poll_interval_us
    }

    fn get_line_capacity(&self) -> u8 {
        self.state.get_line_capacity()
    }

    fn calculate_pos_by_offset(&self, start: (u8, u8), offset: (i8, i8)) -> (u8, u8) {
        self.state.calculate_pos_by_offset(start, offset)
    }

    fn delay_ms(&mut self, ms: u32) {
        self.delayer.delay_ms(ms);
    }

    fn delay_us(&mut self, us: u32) {
        self.delayer.delay_us(us)
    }
}

impl<'a, 'b, Sender, Delayer> Ext for Lcd<'a, 'b, Sender, Delayer>
where
    Delayer: DelayNs,
    Sender: SendCommand<Delayer>,
{
}

impl<'a, 'b, Sender, Delayer> Anim for Lcd<'a, 'b, Sender, Delayer>
where
    Delayer: DelayNs,
    Sender: SendCommand<Delayer>,
{
}
