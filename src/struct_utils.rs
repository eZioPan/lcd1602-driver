pub trait StructUtils {
    fn calculate_pos_by_offset(&self, original_pos: (u8, u8), offset: (i8, i8)) -> (u8, u8);
}
