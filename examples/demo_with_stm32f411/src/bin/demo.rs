//! Drive LCD1602 with a STM32F411RET6 in 4 Pin Mode
//!
//! this demo use many different read/write functions intentionally, to test functions works just fine.

//! Wiring diagram
//!
//! LCD1602 <-> STM32F411RET6
//!     Vss <-> GND
//!     Vdd <-> 5V (It is best to use an external source for the 5V pin, such as the 5V output from a DAPLink device or USB.)
//!      V0 <-> potentiometer <-> 5V & GND (to adjust the display contrast)
//!      RS <-> PA0
//!      RW <-> PA1
//!      EN <-> PA2 (and optionally connect to a 4.7 kOhm Pulldown resistor, to stable voltage level when STM32 reset)
//!      D4 <-> PA3
//!      D5 <-> PA4
//!      D6 <-> PA5
//!      D7 <-> PA6
//!       A <-> 5V
//!       K <-> GND

#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use stm32f4xx_hal::{pac, prelude::*};

use lcd1602_driver::{
    builder::Builder,
    command::{DataWidth, MoveDirection, State},
    impls::parallel_sender::ParallelSender,
    lcd::{FlipStyle, MoveStyle},
    state::LcdState,
    utils::BitOps,
};

// a heart shape
const HEART: [u8; 8] = [
    0b00000, 0b00000, 0b01010, 0b11111, 0b01110, 0b00100, 0b00000, 0b00000,
];

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = pac::Peripherals::take().expect("Cannot take device peripherals");
    let cp = pac::CorePeripherals::take().expect("Cannot take core peripherals");

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(12.MHz()).freeze();

    let mut delayer = cp.SYST.delay(&clocks);

    // init needed digital pins

    let gpioa = dp.GPIOA.split();

    // Push-pull mode for a fast interaction
    let rs_pin = gpioa.pa0.into_push_pull_output().erase();
    let rw_pin = gpioa.pa1.into_push_pull_output().erase();
    let en_pin = gpioa.pa2.into_push_pull_output().erase();

    let db4_pin = gpioa
        .pa3
        .into_open_drain_output()
        .internal_pull_up(true)
        .erase();
    let db5_pin = gpioa
        .pa4
        .into_open_drain_output()
        .internal_pull_up(true)
        .erase();
    let db6_pin = gpioa
        .pa5
        .into_open_drain_output()
        .internal_pull_up(true)
        .erase();
    let db7_pin = gpioa
        .pa6
        .into_open_drain_output()
        .internal_pull_up(true)
        .erase();

    // put pins together
    let mut sender =
        ParallelSender::new_4pin(rs_pin, rw_pin, en_pin, db4_pin, db5_pin, db6_pin, db7_pin);

    let mut lcd_state = LcdState::default();
    lcd_state.set_data_width(DataWidth::Bit4);

    // init LCD1602
    let mut lcd = Builder::new(&mut sender, &mut delayer, lcd_state, 10).init();

    lcd.clean_display();

    // draw a little heart in CGRAM
    lcd.write_graph_to_cgram(1, &HEART);

    // to test cgram read
    // read heart graph from CGRAM, modify it to a diamond shape, then write it to another CGRAM address
    let mut graph_data = lcd.read_graph_from_cgram(1);
    graph_data[1].set_bit(2);
    graph_data[2].set_bit(2);
    lcd.write_graph_to_cgram(2, &graph_data);

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
    // since tilde chracter (~) is not in CGROM of LCD1602A
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
    lcd.write_graph_to_pos(1, (15, 0));
    lcd.delay_ms(1_000);
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

    #[allow(clippy::empty_loop)]
    loop {}
}
