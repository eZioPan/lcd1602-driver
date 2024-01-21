use embedded_hal::delay::DelayNs;

use crate::{
    command::{CommandSet, DataWidth, SendCommand},
    lcd::Lcd,
    state::LcdState,
};

pub struct Builder<'a, 'b, Sender: SendCommand, Delayer: DelayNs> {
    sender: Option<&'a mut Sender>,
    delayer: Option<&'b mut Delayer>,
    state: Option<LcdState>,
    poll_interval_us: u32,
}

impl<'a, 'b, Sender: SendCommand, Delayer: DelayNs> Builder<'a, 'b, Sender, Delayer> {
    pub fn new(
        sender: &'a mut Sender,
        delayer: &'b mut Delayer,
        state: LcdState,
        poll_interval_us: u32,
    ) -> Self {
        Self {
            sender: Some(sender),
            delayer: Some(delayer),
            state: Some(state),
            poll_interval_us,
        }
    }

    pub fn init(mut self) -> Lcd<'a, 'b, Sender, Delayer> {
        let sender = self.sender.take().unwrap();
        let delayer = self.delayer.take().unwrap();
        let state = self.state.take().unwrap();

        // in initialization process, we'd better use "raw command", to strictly follow datasheet

        // only first 2 or 3 commands are different between 4 pin and 8 pin mode
        match state.get_data_width() {
            DataWidth::Bit4 => {
                sender.delay_and_send(CommandSet::HalfFunctionSet, delayer, 40_000);

                sender.delay_and_send(
                    CommandSet::FunctionSet(
                        DataWidth::Bit4,
                        state.get_line_mode(),
                        state.get_font(),
                    ),
                    delayer,
                    40,
                );

                sender.delay_and_send(
                    CommandSet::FunctionSet(
                        DataWidth::Bit4,
                        state.get_line_mode(),
                        state.get_font(),
                    ),
                    delayer,
                    40,
                );
            }

            DataWidth::Bit8 => {
                sender.delay_and_send(
                    CommandSet::FunctionSet(
                        DataWidth::Bit8,
                        state.get_line_mode(),
                        state.get_font(),
                    ),
                    delayer,
                    40_000,
                );

                sender.delay_and_send(
                    CommandSet::FunctionSet(
                        DataWidth::Bit8,
                        state.get_line_mode(),
                        state.get_font(),
                    ),
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
            },
            delayer,
            self.poll_interval_us,
        );

        sender.wait_and_send(CommandSet::ClearDisplay, delayer, self.poll_interval_us);

        sender.wait_and_send(
            CommandSet::EntryModeSet(state.get_direction(), state.get_shift()),
            delayer,
            self.poll_interval_us,
        );

        // set backlight after LCD init
        sender.set_backlight(state.get_backlight());

        Lcd::new(sender, delayer, state, self.poll_interval_us)
    }
}
