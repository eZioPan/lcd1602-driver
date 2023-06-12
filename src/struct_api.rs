use super::{
    enums::basic_command::{Font, LineMode, MoveDirection, ShiftType, State},
    RAMType,
};

pub trait StructAPI {
    fn internal_init_lcd(&mut self);
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
    fn internal_calculate_pos_by_offset(&self, offset: (i8, i8)) -> (u8, u8);
}
