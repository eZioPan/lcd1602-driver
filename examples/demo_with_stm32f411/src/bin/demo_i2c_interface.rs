//! Drive LCD1602 with a STM32F411RET6 with a PCF8574T I2C Adapt Board
//!
//! this demo use many different read/write functions intentionally, to test functions works just fine.

//! Wiring diagram
//!
//! I2C Board <-> STM32F411RET6
//!     VCC <-> 5V (It is better to use an external source for the 5V pin, such as a USB powered 5V pin.)
//!     GND <-> GND
//!     SCL <-> PB6
//!     SDA <-> PB7

#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use stm32f4xx_hal::{
    i2c::{self, I2c},
    pac,
    prelude::*,
};

use lcd1602_driver::{
    command::{DataWidth, MoveDirection, State},
    lcd::{self, Anim, Basic, CGRAMGraph, Ext, ExtRead, FlipStyle, Lcd, MoveStyle},
    sender::I2cSender,
    utils::BitOps,
};

// a heart shape
//
// This heart shape is a 5x11 Font shape, we can use the upper part in 5x8 Font mode.
const HEART: CGRAMGraph = CGRAMGraph {
    upper: [
        0b00000, 0b00000, 0b01010, 0b11111, 0b01110, 0b00100, 0b00000, 0b00000,
    ],
    lower: Some([0b00100, 0b01110, 0b00100]),
};

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = pac::Peripherals::take().expect("Cannot take device peripherals");
    let cp = pac::CorePeripherals::take().expect("Cannot take core peripherals");

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(12.MHz()).freeze();

    let mut delayer = cp.SYST.delay(&clocks);

    // init needed digital pins

    let gpiob = dp.GPIOB.split();

    let mut i2c = I2c::new(
        dp.I2C1,
        (gpiob.pb6, gpiob.pb7),
        i2c::Mode::standard(100.kHz()), // The PCF8574T max I2C speed
        &clocks,
    );

    // put pins together
    let mut sender = I2cSender::new(&mut i2c, 0x27u8);

    let lcd_config = lcd::Config::default().set_data_width(DataWidth::Bit4);

    // init LCD1602
    let mut lcd = Lcd::new(&mut sender, &mut delayer, lcd_config, None);

    // draw a little heart in CGRAM
    lcd.write_graph_to_cgram(0, &HEART);

    // to test cgram read
    // read heart graph from CGRAM, modify it to a diamond shape, then write it to another CGRAM address
    let mut graph_data = lcd.read_graph_from_cgram(0);
    graph_data.upper[1].set_bit(2);
    graph_data.upper[2].set_bit(2);
    graph_data.lower.as_mut().unwrap()[1].clear_bit(2);
    lcd.write_graph_to_cgram(1, &graph_data);

    lcd.set_cursor_blink_state(State::On);

    // to test function works
    // we set cursor 1 step right
    lcd.set_cursor_pos((1, 0));

    // type writer effect
    lcd.typewriter_write("hello,", 250_000);

    // relative cursor move
    lcd.offset_cursor_pos((1, 0));

    // to test write string to cur pos
    lcd.write_str_to_cur("world!");

    // manually delay
    lcd.delay_ms(250);

    let line_capacity = lcd.get_line_capacity();

    // to test write character to specified position
    // since tilde character (~) is not in CGROM of LCD1602A
    // it should be displayed as a full rectangle
    lcd.write_char_to_pos('~', (15, 0));

    // manually delay
    lcd.delay_ms(250);

    // to test whether line break works well
    // set cursor to the end of first line, and write a vertical line
    lcd.set_cursor_pos((line_capacity - 1, 0));
    lcd.write_char_to_cur('|');

    // turn off cursor blinking, so that cursor will only be a underline
    lcd.set_cursor_blink_state(State::Off);

    lcd.typewriter_write("Hello, ", 250_000);

    // to test right to left write in
    // move cursor to left end of display window, then type string in reverse order
    lcd.set_direction(MoveDirection::RightToLeft);
    lcd.set_cursor_pos((15, 1));
    lcd.typewriter_write("~!", 250_000);
    // and the 2 type of split flap display effect
    lcd.split_flap_write("2061", FlipStyle::Simultaneous, None, 150_000, None);
    lcd.split_flap_write(
        "DCL",
        FlipStyle::Sequential,
        Some(10),
        150_000,
        Some(250_000),
    );

    lcd.set_cursor_state(State::Off);

    // replace 2 rectangle with custom heart shape and diamond shape
    lcd.delay_ms(1_000);
    lcd.write_graph_to_pos(0, (15, 0));
    lcd.delay_ms(1_000);
    // although we define diamond shape as index 1 above,
    // but since we define shape in 5x11 Font, and read as 5x8 Font, the actual index should be 1*2 = 2.
    lcd.write_graph_to_pos(2, (15, 1));

    // to test read from DDRAM
    // read from first line end, and write same character to the second line end
    let char_at_end = lcd.read_byte_from_pos((39, 0));
    lcd.write_byte_to_pos(char_at_end, (39, 1));

    // shift display window
    lcd.delay_ms(1_000);
    lcd.shift_display_to_pos(2, MoveStyle::Shortest, State::On, 250_000);
    lcd.delay_ms(1_000);
    lcd.shift_display_to_pos(40 - 2, MoveStyle::Shortest, State::On, 250_000);
    lcd.delay_ms(1_000);
    lcd.shift_display_to_pos(0, MoveStyle::Shortest, State::On, 250_000);

    // and blinking display 3 times
    lcd.delay_ms(1_000);
    lcd.full_display_blink(3, 500_000);

    // and blinking backlight 3 times
    for _ in 0..3 {
        lcd.delay_ms(500);
        lcd.set_backlight(State::Off);
        lcd.delay_ms(500);
        lcd.set_backlight(State::On);
    }

    // Set the font to Font5x11, and clean screen, and write a few words.
    lcd.delay_ms(1_000);
    lcd.set_line_mode(lcd1602_driver::command::LineMode::OneLine);
    lcd.set_font(lcd1602_driver::command::Font::Font5x11);
    lcd.clean_display();
    lcd.return_home();
    lcd.set_cursor_blink_state(State::On);

    lcd.write_graph_to_cur(0);
    lcd.typewriter_write("Hello, BigFont", 200_000);
    lcd.write_graph_to_cur(1);

    #[allow(clippy::empty_loop)]
    loop {}
}
