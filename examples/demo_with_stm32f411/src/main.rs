//! 用 STM32F411RET6 驱动一个 LCD1602
//! 使用了 LCD1602 的 4 bit 模式

//! 接线图
//!
//! 其实这个连线图还是比较随意的，除了 GND 和 V5 是固定的引脚之外，其它的 GPIO 引脚是可以随便调整的
//!
//! LCD <-> STM32
//! Vss <-> GND
//! Vdd <-> 5V（这个 5V 最好直接来源于外部，比如 DAPLink 的 5V 输出，或者 USB，使用核心板上 3.3V 升压到 5V 的电流其实不够用）
//! V0 <-> 可变电阻 <-> 5V（调节显示对比度）
//! RS <-> PA0
//! RW <-> PA1
//! EN [<-> PA2, <-> 4.7 kOhm 下拉电阻 <-> GND]
//! D4 <-> PA3
//! D5 <-> PA4
//! D6 <-> PA5
//! D7 <-> PA6
//! A <-> 可变电阻 <-> 5V（这里路的可变电阻我设计用来调节背光亮度，是可选的，而且准确来说应该用 PWM 调光，我这里就不再设计了）
//! K <-> GND

#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use stm32f4xx_hal::{pac, prelude::*};

use lcd1602_driver::{
    builder::{Builder, BuilderAPI},
    enums::{
        animation::{FlipStyle, MoveStyle},
        basic_command::{Font, LineMode, MoveDirection, ShiftType, State},
    },
    pins::{FourPinsAPI, Pins},
    utils::BitOps,
    LCDAnimation, LCDBasic, LCDExt,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = pac::Peripherals::take().expect("Cannot take device peripherals");
    let cp = pac::CorePeripherals::take().expect("Cannot take core peripherals");

    let rcc = dp.RCC.constrain();

    // 其实这个 Clocks 还挺有趣的，它记录了各种总线、Cortex 核心，以及 I2S 的运行频率，以及两个 APB 的分频值
    // 算是 STM32CubeMX Clock 视图的替换了
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

    let delayer = cp.SYST.delay(&clocks);

    let gpioa = dp.GPIOA.split();

    // 准确来说，这三个引脚应该在外部接分别接一个小一点的上拉电阻（比如 4.7KOhm 的）
    // 不过我手上没有合适的电阻，这里就先用 pull_push 模式替代了
    let rs_pin = gpioa.pa0.into_push_pull_output().erase();
    let rw_pin = gpioa.pa1.into_push_pull_output().erase();

    // EN 引脚的问题，我还么有想好，准确来说，它应该在外部接一个下拉电阻，防止单片机重启的时候，电平跳动，导致 LCD1602 收到奇怪的信号
    // 但如果我们将这个口设置为开漏输出，则它又要求接一个上拉电阻，这和我们默认需要将其下拉的要求相冲突
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

    let lcd_pins = Pins::new(rs_pin, rw_pin, en_pin, db4_pin, db5_pin, db6_pin, db7_pin);

    let lcd_builder = Builder::new(lcd_pins, delayer)
        .set_blink(State::On)
        .set_cursor(State::On)
        .set_direction(MoveDirection::LeftToRight)
        .set_display(State::On)
        .set_font(Font::Font5x8)
        .set_line(LineMode::TwoLine)
        .set_shift(ShiftType::CursorOnly)
        .set_wait_interval_us(10);

    let mut lcd = lcd_builder.build_and_init();

    // 在 CGRAM 里画一个小爱心
    lcd.write_graph_to_cgram(1, &HEART);

    // 测试读取 CGRAM 的功能
    // 修改心形图案为菱形,并存储到另一个位置上
    let mut graph_data = lcd.read_graph_from_cgram(1);
    graph_data[1].set_bit(2);
    graph_data[2].set_bit(2);
    lcd.write_graph_to_cgram(2, &graph_data);

    lcd.set_cursor_pos((1, 0)); // 这里我们故意向右偏移了一个字符，测试偏移功能是否正常

    lcd.typewriter_write("hello, world! ~", 250_000); // 这里故意追加了一个波浪线，应该被映射为全亮方块

    lcd.delay_ms(250);
    lcd.set_cursor_pos((39, 0)); // 这里故意设置到第一行的末尾，测试换行功能是否正常
    lcd.write_char_to_cur('|'); // 让后我们在第一行的行尾写入一个竖线

    lcd.set_cursor_blink_state(State::Off);

    lcd.typewriter_write("hello, ", 250_000);

    // 测试从右至左的写入
    lcd.set_default_direction(MoveDirection::RightToLeft);
    lcd.set_cursor_pos((15, 1));
    lcd.typewriter_write("~!", 250_000);
    // 测试两种滚动写入
    lcd.split_flap_write("2061", FlipStyle::Simultaneous, 0, 150_000, None);
    lcd.split_flap_write("DCL", FlipStyle::Sequential, 10, 150_000, Some(250_000));

    lcd.set_cursor_state(State::Off);

    // 用我们绘制的心形和菱形覆盖全亮的方块
    lcd.delay_ms(1_000);
    lcd.write_graph_to_pos(1, (15, 0));
    lcd.delay_ms(1_000);
    lcd.write_graph_to_pos(2, (15, 1));

    // 测试读取 DDRAM 的功能
    // 偷偷在 DDRAM 的末尾写上一个竖线
    let char_at_end = lcd.read_byte_from_pos((39, 0));
    lcd.write_byte_to_pos(char_at_end, (39, 1));

    // 挪动一下屏幕
    lcd.delay_ms(1_000);
    lcd.shift_display_to_pos(2, MoveStyle::Shortest, State::On, 250_000);
    lcd.delay_ms(1_000);
    lcd.shift_display_to_pos(40 - 2, MoveStyle::Shortest, State::On, 250_000);
    lcd.delay_ms(1_000);
    lcd.shift_display_to_pos(0, MoveStyle::Shortest, State::On, 250_000);

    // 让后让整个屏幕闪烁三次
    lcd.delay_ms(1_000);
    lcd.full_display_blink(3, 500_000);

    loop {}
}

const HEART: [u8; 8] = [
    0b00000, 0b00000, 0b01010, 0b11111, 0b01110, 0b00100, 0b00000, 0b00000,
];
