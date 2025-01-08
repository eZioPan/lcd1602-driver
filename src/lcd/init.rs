use embedded_hal::delay::DelayNs;

use crate::{
    command::{CommandSet, DataWidth, Font, LineMode, MoveDirection, RAMType, ShiftType, State},
    lcd::Lcd,
    sender::SendCommand,
    state::LcdState,
};

/// [`Config`] is the init config of a [`Lcd`]
#[derive(Default)]
pub struct Config {
    state: LcdState,
}

#[allow(missing_docs)]
impl Config {
    pub fn get_backlight(&self) -> State {
        self.state.get_backlight()
    }

    pub fn set_backlight(mut self, backlight: State) -> Self {
        self.state.set_backlight(backlight);
        self
    }

    pub fn get_data_width(&self) -> DataWidth {
        self.state.get_data_width()
    }

    pub fn set_data_width(mut self, data_width: DataWidth) -> Self {
        self.state.set_data_width(data_width);
        self
    }

    pub fn get_line_mode(&self) -> LineMode {
        self.state.get_line_mode()
    }

    pub fn set_line_mode(mut self, line: LineMode) -> Self {
        self.state.set_line_mode(line);
        self
    }

    pub fn get_line_capacity(&self) -> u8 {
        self.state.get_line_capacity()
    }

    pub fn get_font(&self) -> Font {
        self.state.get_font()
    }

    pub fn set_font(mut self, font: Font) -> Self {
        self.state.set_font(font);
        self
    }

    pub fn get_display_state(&self) -> State {
        self.state.get_display_state()
    }

    pub fn set_display_state(mut self, display: State) -> Self {
        self.state.set_display_state(display);
        self
    }

    pub fn get_cursor_state(&self) -> State {
        self.state.get_cursor_state()
    }

    pub fn set_cursor_state(mut self, cursor: State) -> Self {
        self.state.set_cursor_state(cursor);
        self
    }

    pub fn get_cursor_blink(&self) -> State {
        self.state.get_cursor_blink()
    }

    pub fn set_cursor_blink(mut self, blink: State) -> Self {
        self.state.set_cursor_blink(blink);
        self
    }

    pub fn get_direction(&self) -> MoveDirection {
        self.state.get_direction()
    }

    pub fn set_direction(mut self, dir: MoveDirection) -> Self {
        self.state.set_direction(dir);
        self
    }

    pub fn get_shift_type(&self) -> ShiftType {
        self.state.get_shift_type()
    }

    pub fn set_shift_type(mut self, shift: ShiftType) -> Self {
        self.state.set_shift_type(shift);
        self
    }

    pub fn get_cursor_pos(&self) -> (u8, u8) {
        self.state.get_cursor_pos()
    }

    pub fn set_cursor_pos(mut self, pos: (u8, u8)) -> Self {
        self.state.set_cursor_pos(pos);
        self
    }

    pub fn get_display_offset(&self) -> u8 {
        self.state.get_display_offset()
    }

    pub fn set_display_offset(mut self, offset: u8) -> Self {
        self.state.set_display_offset(offset);
        self
    }

    pub fn get_ram_type(&self) -> RAMType {
        self.state.get_ram_type()
    }

    pub fn set_ram_type(mut self, ram_type: RAMType) -> Self {
        self.state.set_ram_type(ram_type);
        self
    }
}

fn lcd_init<'a, 'b, Sender, Delayer, const READABLE: bool>(
    sender: &'a mut Sender,
    delayer: &'b mut Delayer,
    config: Config,
    poll_interval_us: u32,
) -> Lcd<'a, 'b, Sender, Delayer, READABLE>
where
    Sender: SendCommand<Delayer, READABLE>,
    Delayer: DelayNs,
{
    let state = config.state;

    // in initialization process, we'd better use "raw command", to strictly follow datasheet

    // only first 2 or 3 commands are different between 4 pin and 8 pin mode
    match state.get_data_width() {
        DataWidth::Bit4 => {
            sender.delay_and_send(CommandSet::HalfFunctionSet.into(), delayer, 40_000);

            sender.delay_and_send(
                CommandSet::FunctionSet(DataWidth::Bit4, state.get_line_mode(), state.get_font())
                    .into(),
                delayer,
                40,
            );

            sender.delay_and_send(
                CommandSet::FunctionSet(DataWidth::Bit4, state.get_line_mode(), state.get_font())
                    .into(),
                delayer,
                40,
            );
        }

        DataWidth::Bit8 => {
            sender.delay_and_send(
                CommandSet::FunctionSet(DataWidth::Bit8, state.get_line_mode(), state.get_font())
                    .into(),
                delayer,
                40_000,
            );

            sender.delay_and_send(
                CommandSet::FunctionSet(DataWidth::Bit8, state.get_line_mode(), state.get_font())
                    .into(),
                delayer,
                40,
            );
        }
    }

    sender.wait_and_send(
        CommandSet::DisplayOnOff {
            display: state.get_display_state(),
            cursor: state.get_cursor_state(),
            cursor_blink: state.get_cursor_blink(),
        }
        .into(),
        delayer,
        poll_interval_us,
    );

    sender.wait_and_send(CommandSet::ClearDisplay.into(), delayer, poll_interval_us);

    // Accroding to ST6077U datasheet, after CommandSet::ClearDisplay we will need to manually
    // wait at least 1.52 ms before send CommandSet::EntryModeSet
    // This only needs to happen if we don't know when the command finishes (aka not readable)
    if !READABLE {
        delayer.delay_us(1520);
    }

    sender.wait_and_send(
        CommandSet::EntryModeSet(state.get_direction(), state.get_shift_type()).into(),
        delayer,
        poll_interval_us,
    );

    // set backlight after LCD init
    sender.set_backlight(state.get_backlight());

    Lcd {
        sender,
        delayer,
        state,
        poll_interval_us,
    }
}

impl<'a, 'b, Sender, Delayer> Lcd<'a, 'b, Sender, Delayer, false>
where
    Sender: SendCommand<Delayer, false>,
    Delayer: DelayNs,
{
    /// Create a [`Lcd`] driver, and init LCD hardware
    pub fn new_write_only(
        sender: &'a mut Sender,
        delayer: &'b mut Delayer,
        config: Config,
    ) -> Self {
        lcd_init(sender, delayer, config, 0)
    }
}

impl<'a, 'b, Sender, Delayer> Lcd<'a, 'b, Sender, Delayer, true>
where
    Sender: SendCommand<Delayer, true>,
    Delayer: DelayNs,
{
    /// Create a [`Lcd`] driver, and init LCD hardware
    pub fn new(
        sender: &'a mut Sender,
        delayer: &'b mut Delayer,
        config: Config,
        poll_interval_us: u32,
    ) -> Self {
        lcd_init(sender, delayer, config, poll_interval_us)
    }
}
