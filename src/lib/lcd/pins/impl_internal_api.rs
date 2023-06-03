use crate::utils::{check_bit, clear_bit, set_bit, BitState};

use super::{Pins, PinsInternalAPI};

impl PinsInternalAPI for Pins {
    fn push_4_bits(&mut self, raw_bits: u8) {
        self.db_pins
            .iter_mut()
            .enumerate()
            .for_each(|(index, pin)| match check_bit(raw_bits, index as u8) {
                BitState::Set => pin.set_high(),
                BitState::Clear => pin.set_low(),
            });
    }

    fn fetch_4_bits(&mut self) -> u8 {
        self.db_pins
            .iter_mut()
            .enumerate()
            // .fold() 在这里用于在每次迭代中，不断修改同一个值
            .fold(0u8, |mut acc, (index, pin)| {
                // 在使用开漏脚的读取形式时，记得将引脚“置高”，以“释放”对引脚的拉低
                pin.set_high();
                // 这里不可以 pin.get_state() 函数，
                // .get_state() 返回的是该引脚被软件设置的状态，对应的是 .is_set_high() 和 .is_set_low() 函数
                // 这里只能用 .is_high() 或 .is_low() 来读取开漏脚监测到的外部电平
                match pin.is_low() {
                    false => set_bit(&mut acc, index as u8),
                    true => clear_bit(&mut acc, index as u8),
                }
                acc
            })
    }
}
