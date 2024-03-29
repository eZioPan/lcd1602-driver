# LCD1602 Driver

[![crates.io](https://img.shields.io/crates/v/lcd1602-driver.svg)](https://crates.io/crates/lcd1602-driver)
[![docs.rs](https://docs.rs/lcd1602-driver/badge.svg)](https://docs.rs/lcd1602-driver)

An embedded-hal based driver for the LCD1602 display

### INFO

The most common functions have been implemented, but this crate is still in a **very early stage** of development, so the API and functionality may change.

## Features

- Support both parallel interface, and I2C adapter board interface
- Covers every(?) instruction of the LCD1602
  - 4 Pin / 8 Pin mode
  - 1 line / 2 line display
  - left to right / right to left writing
  - Offset display window
  - Read busy flag
  - Read/write DDRAM and CGRAM
  - Set cursor show/hide, set cursor blink or not
  - And other functions directly provided by the LCD1602 instructions
- All instructions are sent after reading the busy flag for the efficient execution
  - Note: According to the LCD1602 specification, the first few instructions of the initialization process must wait for a specific amount of time
- Simulates the state of the LCD1602 in the MCU's memory to reduce unnecessary reads from the LCD1602 memory
- Some commonly used functionalities
  - Initialize LCD via Builder pattern
  - Write strings at the current position
  - Write strings at a specific position represented by (x,y) coordinates
  - Offset cursor position relatively (represented by (x,y) coordinates)
  - Offset display window to a specific position
  - Write custom character graph represented by an array to a specific position in CGRAM
  - Read custom characters at a specific position in CGRAM
  - Toggle the display of the entire display on/off
- Some simple animation effects
  - Delay execution (microseconds/milliseconds)
  - Full-screen blinking (endless/specific number of times)
  - Typewriter-style string display
  - Split-flap-style string display (one by one/simultaneously)
- And more...

## Examples

See [examples/demo_with_stm32f411](https://github.com/eZioPan/lcd1602-driver/tree/latest/examples/demo_with_stm32f411)

## CHANGELOG

### v0.1.0

- First Release

### v0.2.0

- Upgrade to embedded-hal 1.0
- Add I2C adapter board support
- Simplify codebase
