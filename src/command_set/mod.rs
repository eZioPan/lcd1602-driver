#![doc(hidden)]

use crate::enums::basic_command::{DataWidth, Font, LineMode, MoveDirection, ShiftType, State};

#[derive(Clone, Copy)]
pub(super) enum CommandSet {
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
