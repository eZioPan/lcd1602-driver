use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};

use super::{
    command_set::CommandSet, enums::basic_command::State, LCDBasic, LCDExt, PinsInteraction,
    RAMType, StructAPI, LCD,
};

impl<ControlPin, DBPin, const PIN_CNT: usize, Delayer> LCDExt
    for LCD<ControlPin, DBPin, PIN_CNT, Delayer>
where
    ControlPin: OutputPin,
    DBPin: OutputPin + InputPin,
    Delayer: DelayMs<u32> + DelayUs<u32>,
{
    fn toggle_display(&mut self) {
        match self.get_display_state() {
            State::Off => self.set_display_state(State::On),
            State::On => self.set_display_state(State::Off),
        }
    }

    fn write_str_to_cur(&mut self, str: &str) {
        str.chars().for_each(|char| self.write_char_to_cur(char));
    }

    fn write_str_to_pos(&mut self, str: &str, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.write_str_to_cur(str);
    }

    /// In this implementation, character only support
    /// from ASCII 0x20 (white space) to ASCII 0x7D (`}`)
    fn write_char_to_cur(&mut self, char: char) {
        assert!(
            self.get_ram_type() == RAMType::DDRAM,
            "Current in CGRAM, use .set_cursor_pos() to change to DDRAM"
        );

        // map char out side of ASCII 0x20 and 0x7D to full rectangle
        let out_byte = match char.is_ascii() {
            true if (0x20 <= char as u8) && (char as u8 <= 0x7D) => char as u8,
            _ => 0xFF,
        };

        self.write_u8_to_cur(out_byte);
    }

    fn write_graph_to_pos(&mut self, index: u8, pos: (u8, u8)) {
        assert!(index < 8, "Only 8 graphs allowed in CGRAM");
        self.write_byte_to_pos(index, pos);
    }

    fn write_byte_to_pos(&mut self, byte: impl Into<u8>, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.wait_and_send(CommandSet::WriteDataToRAM(byte.into()));
    }

    fn write_char_to_pos(&mut self, char: char, pos: (u8, u8)) {
        self.set_cursor_pos(pos);
        self.write_char_to_cur(char);
    }

    fn read_byte_from_pos(&mut self, pos: (u8, u8)) -> u8 {
        let original_pos = self.get_cursor_pos();
        self.set_cursor_pos(pos);
        let data = self.read_u8_from_cur();
        self.set_cursor_pos(original_pos);
        data
    }

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

    fn offset_cursor_pos(&mut self, offset: (i8, i8)) {
        self.set_cursor_pos(self.internal_calculate_pos_by_offset(offset));
    }
}
