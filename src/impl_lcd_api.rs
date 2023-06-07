use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};

use super::{
    command_set::CommandSet,
    enums::basic_command::{DataWidth, Font, LineMode, MoveDirection, ShiftType, State},
    LCDBasic, PinsInteraction, RAMType, StructAPI, LCD,
};

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> LCDBasic
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    fn clean_display(&mut self) {
        self.wait_and_send(CommandSet::ClearDisplay);
    }

    fn return_home(&mut self) {
        self.wait_and_send(CommandSet::ReturnHome);
    }

    fn set_cgram_addr(&mut self, addr: u8) {
        assert!(addr < 2u8.pow(6), "CGRAM Address overflow");

        self.internal_set_ram_type(RAMType::CGRAM);

        self.wait_and_send(CommandSet::SetCGRAM(addr));
    }

    fn write_graph_to_cur(&mut self, index: u8) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");
        self.write_u8_to_cur(index);
    }

    fn write_u8_to_cur(&mut self, byte: impl Into<u8>) {
        assert!(
            self.get_ram_type() == RAMType::DDRAM,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        self.wait_and_send(CommandSet::WriteDataToRAM(byte.into()));

        // 由于 LCD1602 的计数器会自动自增，因此这里只需要更新结构体的计数即可
        // 由于 UT7066U 的内存是循环的，因此到最后一个位置之后，内存的地址是回到原点的
        let last_pos = self.get_cursor_pos();
        let line_capacity: u8 = match self.get_line_mode() {
            LineMode::OneLine => 40,
            LineMode::TwoLine => 80,
        };

        match self.get_default_direction() {
            MoveDirection::RightToLeft => match self.line {
                LineMode::OneLine => {
                    if last_pos.0 == 0 {
                        self.internal_set_cursor_pos((line_capacity - 1, 0));
                    } else {
                        self.internal_set_cursor_pos((last_pos.0 - 1, 0));
                    }
                }
                LineMode::TwoLine => {
                    if last_pos.0 == 0 {
                        if last_pos.1 == 1 {
                            self.internal_set_cursor_pos((line_capacity - 1, 0));
                        } else {
                            self.internal_set_cursor_pos((line_capacity - 1, 1));
                        }
                    } else {
                        self.internal_set_cursor_pos((last_pos.0 - 1, last_pos.1));
                    }
                }
            },
            MoveDirection::LeftToRight => match self.line {
                LineMode::OneLine => {
                    if last_pos.0 == line_capacity - 1 {
                        self.internal_set_cursor_pos((0, 0));
                    } else {
                        self.internal_set_cursor_pos((last_pos.0 + 1, 0));
                    }
                }
                LineMode::TwoLine => {
                    if last_pos.0 == line_capacity - 1 {
                        if last_pos.1 == 0 {
                            self.internal_set_cursor_pos((1, 0));
                        } else {
                            self.internal_set_cursor_pos((0, 0));
                        }
                    } else {
                        self.internal_set_cursor_pos((last_pos.0 + 1, last_pos.1));
                    }
                }
            },
        }
    }

    fn read_u8_from_cur(&mut self) -> u8 {
        self.wait_and_send(CommandSet::ReadDataFromRAM).unwrap()
    }

    fn write_graph_to_cgram(&mut self, index: u8, graph_data: &[u8; 8]) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");

        // 所有的行，设置为 1 的位，都应仅限于低 4 位
        assert!(
            graph_data.iter().all(|&line| line < 2u8.pow(5)),
            "Only lower 5 bits use to construct display"
        );

        // 有一个问题是，如果写入方向是从右到左，那么这里需要临时调整一下方向
        // 调整为从左到右，这样我们绘制字符的时候，就是从上到下绘制
        let mut direction_fliped = false;
        if self.get_default_direction() == MoveDirection::RightToLeft {
            self.set_default_direction(MoveDirection::LeftToRight);
            direction_fliped = true;
        }

        let cgram_data_addr_start = index.checked_shl(3).unwrap();

        // 注意 AC 在 CGRAM 里也是会自增的，因此不需要每一步都设置位置
        self.set_cgram_addr(cgram_data_addr_start as u8);
        graph_data.iter().for_each(|&line_data| {
            self.wait_and_send(CommandSet::WriteDataToRAM(line_data));
        });

        // 最后我们检查一下书写方向是否被翻转，
        // 如果被翻转表示原始书写的方向为从右向左，记得需要翻转回去
        if direction_fliped {
            self.set_default_direction(MoveDirection::RightToLeft)
        }
    }

    fn shift_cursor_or_display(&mut self, st: ShiftType, dir: MoveDirection) {
        self.internal_shift_cursor_or_display(st, dir);
        self.wait_and_send(CommandSet::CursorOrDisplayShift(st, dir));
    }

    fn get_display_offset(&self) -> u8 {
        self.display_offset
    }

    fn set_line_mode(&mut self, line: LineMode) {
        self.internal_set_line_mode(line);
        self.wait_and_send(CommandSet::FunctionSet(
            DataWidth::Bit4,
            self.get_line_mode(),
            self.get_font(),
        ));
    }

    fn get_line_mode(&self) -> LineMode {
        self.line
    }

    fn set_font(&mut self, font: Font) {
        self.internal_set_font(font);
        self.wait_and_send(CommandSet::FunctionSet(
            DataWidth::Bit4,
            self.get_line_mode(),
            self.get_font(),
        ));
    }

    fn get_font(&self) -> Font {
        self.font
    }

    fn set_display_state(&mut self, display: State) {
        self.internal_set_display_state(display);
        self.wait_and_send(CommandSet::DisplayOnOff {
            display: self.get_display_state(),
            cursor: self.get_cursor_state(),
            cursor_blink: self.get_cursor_blink_state(),
        });
    }

    fn get_display_state(&self) -> State {
        self.display_on
    }

    fn set_cursor_state(&mut self, cursor: State) {
        self.internal_set_cursor_state(cursor);
        self.wait_and_send(CommandSet::DisplayOnOff {
            display: self.get_display_state(),
            cursor: self.get_cursor_state(),
            cursor_blink: self.get_cursor_blink_state(),
        });
    }

    fn get_cursor_state(&self) -> State {
        self.cursor_on
    }

    fn set_cursor_blink_state(&mut self, blink: State) {
        self.internal_set_cursor_blink(blink);
        self.wait_and_send(CommandSet::DisplayOnOff {
            display: self.get_display_state(),
            cursor: self.get_cursor_state(),
            cursor_blink: self.get_cursor_blink_state(),
        });
    }

    fn get_cursor_blink_state(&self) -> State {
        self.cursor_blink
    }

    fn set_default_direction(&mut self, dir: MoveDirection) {
        self.internal_set_direction(dir);
        self.wait_and_send(CommandSet::EntryModeSet(
            self.get_default_direction(),
            self.get_default_shift_type(),
        ));
    }

    fn get_default_direction(&self) -> MoveDirection {
        self.direction
    }

    fn set_default_shift_type(&mut self, shift: ShiftType) {
        self.internal_set_shift(shift);
        self.wait_and_send(CommandSet::EntryModeSet(
            self.get_default_direction(),
            self.get_default_shift_type(),
        ));
    }

    fn get_default_shift_type(&self) -> ShiftType {
        self.shift_type
    }

    fn set_cursor_pos(&mut self, pos: (u8, u8)) {
        self.internal_set_ram_type(RAMType::DDRAM);
        self.internal_set_cursor_pos(pos);

        // 这里比较特殊，
        // 如果处于单行模式，没有啥好说的，y 永远是 0，x 是几，实际的地址就是几
        // 如果处于双行模式，y 对于实际地址的偏移量为第二行开头的地址 0x40，x 的偏移量为该行中的偏移量
        // 虽然 LCD1602 说明书中，每一行都没有取到 x 的最大范围，但是我们这里并不怕这个问题，因为我们已经在 internal_set_cursor_pos 方法中检查过这个问题了
        let raw_pos: u8 = pos.1 * 0x40 + pos.0;

        self.wait_and_send(CommandSet::SetDDRAM(raw_pos));
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
