//! Drive LCD1602 with a STM32F411RET6 in 4 Pin Mode
//!
//! this demo use many different read/write functions intentionally, to test functions works just fine.

//! Wiring diagram
//!
//! LCD1602 <-> STM32F411RET6
//!     Vss <-> GND
//!     Vdd <-> 5V (It is better to use an external source for the 5V pin, such as a USB powered 5V pin.)
//!      V0 <-> potentiometer <-> 5V & GND (to adjust the display contrast)
//!      RS <-> PA0
//!      RW <-> PA1
//!      EN <-> PA2 (and optionally connect to a 4.7 kOhm Pulldown resistor, to stable voltage level when STM32 reset)
//!      D4 <-> PA3
//!      D5 <-> PA4
//!      D6 <-> PA5
//!      D7 <-> PA6
//!       A <-> 5V
//!       K <-> external NPN transistor collector
//!     external NPN transistor base <-> PA7
//!     external NPN transistor emitter <-> GND

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};

use lcd1602_driver_async::{
    command::{DataWidth, MoveDirection, State},
    lcd::{self, Anim, Basic, CGRAMGraph, Ext, ExtRead, FlipStyle, Lcd, MoveStyle},
    sender::ParallelSender,
    utils::BitOps,
};

use crate::delay::EmbassyDelay;

// a heart shape
//
// This heart shape is a 5x11 Font shape, we can use the upper part in 5x8 Font mode.
const HEART: CGRAMGraph = CGRAMGraph {
    upper: [
        0b00000, 0b00000, 0b01010, 0b11111, 0b01110, 0b00100, 0b00000, 0b00000,
    ],
    lower: Some([0b00100, 0b01110, 0b00100]),
};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    // init needed digital pins
    let rs_pin = Output::new(p.PA0, Level::Low, Speed::Low);
    let rw_pin = Output::new(p.PA1, Level::Low, Speed::Low);
    let en_pin = Output::new(p.PA2, Level::Low, Speed::Low);

    // Use open drain with pull-up for data pins
    let db4_pin = Input::new(p.PA3, Pull::Up);
    let db5_pin = Input::new(p.PA4, Pull::Up);
    let db6_pin = Input::new(p.PA5, Pull::Up);
    let db7_pin = Input::new(p.PA6, Pull::Up);

    let bl_pin = Output::new(p.PA7, Level::High, Speed::Low);

    let mut delayer = EmbassyDelay::new();

    // put pins together
    let mut sender = ParallelSender::new_4pin(
        rs_pin,
        rw_pin,
        en_pin,
        db4_pin,
        db5_pin,
        db6_pin,
        db7_pin,
        Some(bl_pin),
    );

    let lcd_config = lcd::Config::default().set_data_width(DataWidth::Bit4);

    // init LCD1602
    let mut lcd = Lcd::new(&mut sender, &mut delayer, lcd_config, None).await;

    // draw a little heart in CGRAM
    lcd.write_graph_to_cgram(0, &HEART).await;

    // to test cgram read
    // read heart graph from CGRAM, modify it to a diamond shape, then write it to another CGRAM address
    let mut graph_data = lcd.read_graph_from_cgram(0).await;
    graph_data.upper[1].set_bit(2);
    graph_data.upper[2].set_bit(2);
    graph_data.lower.as_mut().unwrap()[1].clear_bit(2);
    lcd.write_graph_to_cgram(1, &graph_data).await;

    lcd.set_cursor_blink_state(State::On).await;

    // to test function works
    // we set cursor 1 step right
    lcd.set_cursor_pos((1, 0)).await;

    // type writer effect
    lcd.typewriter_write("hello,", 250_000).await;

    // relative cursor move
    lcd.offset_cursor_pos((1, 0)).await;

    // to test write string to cur pos
    lcd.write_str_to_cur("world!").await;

    // manually delay
    lcd.delay_ms(250).await;

    let line_capacity = lcd.get_line_capacity();

    // to test write character to specified position
    // since tilde character (~) is not in CGROM of LCD1602A
    // it should be displayed as a full rectangle
    lcd.write_char_to_pos('~', (15, 0)).await;

    // manually delay
    lcd.delay_ms(250).await;

    // to test whether line break works well
    // set cursor to the end of first line, and write a vertical line
    lcd.set_cursor_pos((line_capacity - 1, 0)).await;
    lcd.write_char_to_cur('|').await;

    // turn off cursor blinking, so that cursor will only be a underline
    lcd.set_cursor_blink_state(State::Off).await;

    lcd.typewriter_write("Hello, ", 250_000).await;

    // to test right to left write in
    // move cursor to left end of display window, then type string in reverse order
    lcd.set_direction(MoveDirection::RightToLeft).await;
    lcd.set_cursor_pos((15, 1)).await;
    lcd.typewriter_write("~!", 250_000).await;
    // and the 2 type of split flap display effect
    lcd.split_flap_write("2061", FlipStyle::Simultaneous, None, 150_000, None).await;
    lcd.split_flap_write(
        "DCL",
        FlipStyle::Sequential,
        Some(10),
        150_000,
        Some(250_000),
    )
    .await;

    lcd.set_cursor_state(State::Off).await;

    // replace 2 rectangle with custom heart shape and diamond shape
    lcd.delay_ms(1_000).await;
    lcd.write_graph_to_pos(0, (15, 0)).await;
    lcd.delay_ms(1_000).await;
    // although we define diamond shape as index 1 above,
    // but since we define shape in 5x11 Font, and read as 5x8 Font, the actual index should be 1*2 = 2.
    lcd.write_graph_to_pos(2, (15, 1)).await;

    // to test read from DDRAM
    // read from first line end, and write same character to the second line end
    let char_at_end = lcd.read_byte_from_pos((39, 0)).await;
    lcd.write_byte_to_pos(char_at_end, (39, 1)).await;

    // shift display window
    lcd.delay_ms(1_000).await;
    lcd.shift_display_to_pos(2, MoveStyle::Shortest, State::On, 250_000)
        .await;
    lcd.delay_ms(1_000).await;
    lcd.shift_display_to_pos(40 - 2, MoveStyle::Shortest, State::On, 250_000)
        .await;
    lcd.delay_ms(1_000).await;
    lcd.shift_display_to_pos(0, MoveStyle::Shortest, State::On, 250_000)
        .await;

    // and blinking display 3 times
    lcd.delay_ms(1_000).await;
    lcd.full_display_blink(3, 500_000).await;

    // and blinking backlight 3 times
    for _ in 0..3 {
        lcd.delay_ms(500).await;
        lcd.set_backlight(State::Off).await;
        lcd.delay_ms(500).await;
        lcd.set_backlight(State::On).await;
    }

    // Set the font to Font5x11, and clean screen, and write a few words.
    lcd.delay_ms(1_000).await;
    lcd.set_line_mode(lcd1602_driver_async::command::LineMode::OneLine)
        .await;
    lcd.set_font(lcd1602_driver_async::command::Font::Font5x11)
        .await;
    lcd.clean_display().await;
    lcd.return_home().await;
    lcd.set_cursor_blink_state(State::On).await;

    lcd.write_graph_to_cur(0).await;
    lcd.typewriter_write("Hello, BigFont", 200_000).await;
    lcd.write_graph_to_cur(1).await;

    #[allow(clippy::empty_loop)]
    loop {
        cortex_m::asm::wfi();
    }
}
