use crate::{basic::LCDBasic, enums::basic_command::LineMode};

pub trait StructUtils: LCDBasic {
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
