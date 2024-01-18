use embedded_hal::delay::DelayNs;

use crate::utils::BitOps;

#[derive(Clone, Copy)]
pub enum CommandSet {
    ClearDisplay,
    ReturnHome,
    EntryModeSet(MoveDirection, ShiftType),
    DisplayOnOff {
        display: State,
        cursor: State,
        cursor_blink: State,
    },
    CursorOrDisplayShift(ShiftType, MoveDirection),
    // this is not a command from datasheet,
    // it's the first (half) command of 4 pin mode
    // we name it, to make things tidy
    HalfFunctionSet,
    FunctionSet(DataWidth, LineMode, Font),
    SetCGRAM(u8),
    SetDDRAM(u8),
    ReadBusyFlagAndAddress,
    WriteDataToRAM(u8),
    ReadDataFromRAM,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum MoveDirection {
    RightToLeft,
    #[default]
    LeftToRight,
}

#[derive(Clone, Copy, Default)]
pub enum ShiftType {
    #[default]
    CursorOnly,
    CursorAndDisplay,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum State {
    Off,
    #[default]
    On,
}

#[derive(Clone, Copy, Default)]
pub enum DataWidth {
    #[default]
    Bit4,
    Bit8,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum LineMode {
    OneLine,
    #[default]
    TwoLine,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Font {
    #[default]
    Font5x8,
    Font5x11,
}

/// The type of memory to access
#[derive(Clone, Copy, Default, PartialEq)]
pub enum RAMType {
    /// Display Data RAM
    #[default]
    DDRam,
    /// Character Generator RAM
    CGRam,
}

pub struct Command {
    rs: RegisterSelection,
    rw: ReadWriteOp,
    data: Option<Bits>, // if it's a read command, then data should be filled by reading process
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum RegisterSelection {
    Command,
    Data,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ReadWriteOp {
    Write,
    Read,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Bits {
    Bit4(u8),
    Bit8(u8),
}

pub trait SendCommand {
    /// Note:
    /// If a driver doesn't implement this command, just silently bypass it
    fn get_backlight(&mut self) -> State;

    /// Note:
    /// If a driver doesn't implement this command, just silently bypass it
    fn set_backlight(&mut self, backlight: State);

    fn send(&mut self, command: impl Into<Command>) -> Option<u8>;

    fn delay_and_send(
        &mut self,
        command: impl Into<Command>,
        delayer: &mut impl DelayNs,
        delay_us: u32,
    ) -> Option<u8> {
        delayer.delay_us(delay_us);
        self.send(command)
    }

    fn wait_and_send(
        &mut self,
        command: impl Into<Command>,
        delayer: &mut impl DelayNs,
        poll_interval_us: u32,
    ) -> Option<u8> {
        self.wait_for_idle(delayer, poll_interval_us);
        self.send(command)
    }

    fn wait_for_idle(&mut self, delayer: &mut impl DelayNs, poll_interval_us: u32) {
        while self.check_busy() {
            delayer.delay_us(poll_interval_us);
        }
    }

    fn check_busy(&mut self) -> bool {
        use crate::utils::BitState;

        let busy_state = self.send(CommandSet::ReadBusyFlagAndAddress).unwrap();
        matches!(busy_state.check_bit(7), BitState::Set)
    }
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
                }

                match st {
                    ShiftType::CursorOnly => raw_bits.clear_bit(0),
                    ShiftType::CursorAndDisplay => raw_bits.set_bit(0),
                }

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
                }
                match cursor {
                    State::Off => raw_bits.clear_bit(1),
                    State::On => raw_bits.set_bit(1),
                }
                match cursor_blink {
                    State::Off => raw_bits.clear_bit(0),
                    State::On => raw_bits.set_bit(0),
                }

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
                }

                match dir {
                    MoveDirection::RightToLeft => raw_bits.clear_bit(2),
                    MoveDirection::LeftToRight => raw_bits.set_bit(2),
                }

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
                }

                match line {
                    LineMode::OneLine => raw_bits.clear_bit(3),
                    LineMode::TwoLine => raw_bits.set_bit(3),
                }

                match font {
                    Font::Font5x8 => raw_bits.clear_bit(2),
                    Font::Font5x11 => raw_bits.set_bit(2),
                }

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
