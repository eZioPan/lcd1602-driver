//! Built-in sender  
//! If you want to create a new sender, you will need to implement [`SendCommand`] trait

use embedded_hal::delay::DelayNs;

use crate::{
    command::{Command, CommandSet, State},
    utils::BitOps,
};

mod i2c_sender;
mod parallel_sender;

pub use i2c_sender::I2cSender;
pub use parallel_sender::ParallelSender;

/// [`SendCommand`] is the trait a sender should implement to communicate with the hardware
pub trait SendCommand<Delayer: DelayNs> {
    /// Parse a [`Command`] and sending data to hardware,
    /// and return the result value when [`Command`] is a [`ReadWriteOp::Read`](crate::command::ReadWriteOp::Read) command
    fn send(&mut self, command: Command) -> Option<u8>;

    /// Wait specific duration, and send command
    fn delay_and_send(
        &mut self,
        command: Command,
        delayer: &mut Delayer,
        delay_us: u32,
    ) -> Option<u8> {
        delayer.delay_us(delay_us);
        self.send(command)
    }

    /// Check LCD busy state, when LCD is idle, send the command
    fn wait_and_send(
        &mut self,
        command: Command,
        delayer: &mut Delayer,
        poll_interval_us: u32,
    ) -> Option<u8> {
        self.wait_for_idle(delayer, poll_interval_us);
        self.send(command)
    }

    /// Wait in a busy loop, until LCD is idle
    fn wait_for_idle(&mut self, delayer: &mut Delayer, poll_interval_us: u32) {
        while self.check_busy() {
            delayer.delay_us(poll_interval_us);
        }
    }

    /// Check LCD busy state
    fn check_busy(&mut self) -> bool {
        use crate::utils::BitState;

        let busy_state = self
            .send(CommandSet::ReadBusyFlagAndAddress.into())
            .unwrap();
        matches!(busy_state.check_bit(7), BitState::Set)
    }

    /// Get the current backlight
    ///
    /// Note:
    /// If a driver doesn't support read backlight state, just silently bypass it
    fn get_backlight(&mut self) -> State {
        State::default()
    }

    /// Set the backlight
    ///
    /// Note:
    /// If a driver doesn't support change backlight, just silently bypass it
    #[allow(unused_variables)]
    fn set_backlight(&mut self, backlight: State) {}
}
