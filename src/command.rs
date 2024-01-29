//! Command to control LCD

use crate::utils::BitOps;

// It contain all commands from LCD1602 datasheet
#[derive(Clone, Copy)]
pub(crate) enum CommandSet {
    ClearDisplay,
    ReturnHome,
    EntryModeSet(MoveDirection, ShiftType),
    DisplayOnOff {
        display: State,
        cursor: State,
        cursor_blink: State,
    },
    CursorOrDisplayShift(ShiftType, MoveDirection),
    // This is not a command from datasheet.
    // It's the first (half) command of 4 pin mode
    // we name it, to make things tidy
    HalfFunctionSet,
    FunctionSet(DataWidth, LineMode, Font),
    SetCGRAM(u8),
    SetDDRAM(u8),
    ReadBusyFlagAndAddress,
    WriteDataToRAM(u8),
    ReadDataFromRAM,
}

/// [`MoveDirection`] defines the cursor and display window move direction
#[derive(Clone, Copy, PartialEq, Default)]
pub enum MoveDirection {
    #[allow(missing_docs)]
    RightToLeft,
    #[allow(missing_docs)]
    #[default]
    LeftToRight,
}

/// [`ShiftType`] defines the movement is cursor only or both cursor and display window
#[derive(Clone, Copy, Default)]
pub enum ShiftType {
    #[allow(missing_docs)]
    #[default]
    CursorOnly,
    #[allow(missing_docs)]
    CursorAndDisplay,
}

/// [`State`] defines a On/Off state
#[derive(Clone, Copy, PartialEq, Default)]
pub enum State {
    #[allow(missing_docs)]
    Off,
    #[allow(missing_docs)]
    #[default]
    On,
}

/// [`DataWidth`] defines data width of a [`Command`]  
/// Should match current Sender's pin config
#[derive(Clone, Copy, Default)]
pub enum DataWidth {
    #[allow(missing_docs)]
    #[default]
    Bit4,
    #[allow(missing_docs)]
    Bit8,
}

/// [`LineMode`] is current LCD display line count
#[derive(Clone, Copy, Default, PartialEq)]
pub enum LineMode {
    #[allow(missing_docs)]
    OneLine,
    #[allow(missing_docs)]
    #[default]
    TwoLine,
}

/// [`Font`] is current display font
#[derive(Clone, Copy, Default, PartialEq)]
pub enum Font {
    #[allow(missing_docs)]
    #[default]
    Font5x8,
    #[allow(missing_docs)]
    Font5x11,
}

/// [`RAMType`] is the type of memory to access
#[derive(Clone, Copy, Default, PartialEq)]
pub enum RAMType {
    /// Display Data RAM
    #[default]
    DDRam,
    /// Character Generator RAM
    CGRam,
}

/// A sender should parse a [`Command`] and send the data to hardware to write/read data to/from hardware.
pub struct Command {
    rs: RegisterSelection,
    rw: ReadWriteOp,
    data: Option<Bits>, // if it's a read command, then data should be filled by reading process
}

/// [`RegisterSelection`] defines LCD1602's register type that driver interact with.  
/// A sender should change its "RS" pin state based on this variant.
#[derive(Clone, Copy, PartialEq)]
pub enum RegisterSelection {
    /// Access to Command register
    Command,
    /// Access to Data register
    Data,
}

/// [`ReadWriteOp`] defines read/write operation that driver interact with.  
/// A sender should change its "RW" pin state based on this variant.
#[derive(Clone, Copy, PartialEq)]
pub enum ReadWriteOp {
    /// It's a write command
    Write,
    /// It's a read command
    Read,
}

/// [`Bits`] defines *current command's* data width.  
/// Most of the command should be 8 bit long, but **fisrt** command in [`DataWidth::Bit4`] mode is special, it requires 4 bit data.
#[derive(Clone, Copy, PartialEq)]
pub enum Bits {
    /// Current command has 4 bit long data
    Bit4(u8),
    /// Current command has 8 bit long data
    Bit8(u8),
}

#[allow(dead_code)]
impl Command {
    pub(crate) fn new(rs: RegisterSelection, rw: ReadWriteOp, data: Option<Bits>) -> Self {
        if (rw == ReadWriteOp::Write) && (data.is_none()) {
            panic!("Write Operation Should have Data");
        }

        Self { rs, rw, data }
    }

    pub(crate) fn get_register_selection(&self) -> RegisterSelection {
        self.rs
    }

    pub(crate) fn set_register_selection(&mut self, rs: RegisterSelection) {
        self.rs = rs
    }

    pub(crate) fn get_read_write_op(&self) -> ReadWriteOp {
        self.rw
    }

    pub(crate) fn set_read_write_op(&mut self, rw: ReadWriteOp) {
        self.rw = rw
    }

    pub(crate) fn get_data(&self) -> Option<Bits> {
        self.data
    }

    pub(crate) fn set_data(&mut self, data: Option<Bits>) {
        self.data = data
    }
}

impl From<CommandSet> for Command {
    fn from(command: CommandSet) -> Self {
        match command {
            CommandSet::ClearDisplay => {
                let raw_bits: u8 = 0b0000_0001;
                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::ReturnHome => {
                let raw_bits: u8 = 0b0000_0010;
                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::EntryModeSet(dir, st) => {
                let mut raw_bits: u8 = 0b0000_0100;

                match dir {
                    MoveDirection::RightToLeft => raw_bits.clear_bit(1),
                    MoveDirection::LeftToRight => raw_bits.set_bit(1),
                };

                match st {
                    ShiftType::CursorOnly => raw_bits.clear_bit(0),
                    ShiftType::CursorAndDisplay => raw_bits.set_bit(0),
                };

                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::DisplayOnOff {
                display,
                cursor,
                cursor_blink,
            } => {
                let mut raw_bits = 0b0000_1000;

                match display {
                    State::Off => raw_bits.clear_bit(2),
                    State::On => raw_bits.set_bit(2),
                };
                match cursor {
                    State::Off => raw_bits.clear_bit(1),
                    State::On => raw_bits.set_bit(1),
                };
                match cursor_blink {
                    State::Off => raw_bits.clear_bit(0),
                    State::On => raw_bits.set_bit(0),
                };

                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::CursorOrDisplayShift(st, dir) => {
                let mut raw_bits = 0b0001_0000;

                match st {
                    ShiftType::CursorOnly => raw_bits.clear_bit(3),
                    ShiftType::CursorAndDisplay => raw_bits.set_bit(3),
                };

                match dir {
                    MoveDirection::RightToLeft => raw_bits.clear_bit(2),
                    MoveDirection::LeftToRight => raw_bits.set_bit(2),
                };

                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::HalfFunctionSet => Self::new(
                RegisterSelection::Command,
                ReadWriteOp::Write,
                Some(Bits::Bit4(0b0010)),
            ),

            CommandSet::FunctionSet(width, line, font) => {
                let mut raw_bits = 0b0010_0000;

                match width {
                    DataWidth::Bit4 => raw_bits.clear_bit(4),
                    DataWidth::Bit8 => raw_bits.set_bit(4),
                };

                match line {
                    LineMode::OneLine => raw_bits.clear_bit(3),
                    LineMode::TwoLine => raw_bits.set_bit(3),
                };

                match font {
                    Font::Font5x8 => raw_bits.clear_bit(2),
                    Font::Font5x11 => raw_bits.set_bit(2),
                };

                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::SetCGRAM(addr) => {
                let mut raw_bits = 0b0100_0000;

                assert!(addr < 2u8.pow(6), "CGRAM address out of range");

                raw_bits += addr;

                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::SetDDRAM(addr) => {
                let mut raw_bits = 0b1000_0000;

                assert!(addr < 2u8.pow(7), "DDRAM address out of range");

                raw_bits += addr;

                Self::new(
                    RegisterSelection::Command,
                    ReadWriteOp::Write,
                    Some(Bits::Bit8(raw_bits)),
                )
            }

            CommandSet::ReadBusyFlagAndAddress => {
                Self::new(RegisterSelection::Command, ReadWriteOp::Read, None)
            }

            CommandSet::WriteDataToRAM(data) => Self::new(
                RegisterSelection::Data,
                ReadWriteOp::Write,
                Some(Bits::Bit8(data)),
            ),

            CommandSet::ReadDataFromRAM => {
                Self::new(RegisterSelection::Data, ReadWriteOp::Read, None)
            }
        }
    }
}
