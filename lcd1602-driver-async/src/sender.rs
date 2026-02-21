//! Built-in sender
//!
//! If you want to create a new sender, you will need to implement [`SendCommand`] trait

#![allow(async_fn_in_trait)]

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::delay::DelayNs;

use crate::{
    command::{Command, CommandSet, State},
    utils::{BitOps, BitState},
};

mod i2c_sender;
mod parallel_sender;

pub use i2c_sender::I2cSender;
pub use parallel_sender::ParallelSender;

/// [`IfReadable`] trait is a simple const bound trait
///
/// Note:
///
/// If your [`SendCommand`] implementation has same code on both Read-Write and Write-Only mode,
/// you can replace trait bound `Output + Input` with `Output + IfReadable<READABLE>`.
pub trait IfReadable<const READABLE: bool> {}

impl<T> IfReadable<false> for T where T: OutputPin {}
impl<T> IfReadable<true> for T where T: OutputPin + InputPin {}

/// [`SendCommand`] is the trait a sender should implement to communicate with the hardware
pub trait SendCommand<Delayer: DelayNs, const READABLE: bool> {
    /// Parse a [`Command`] and sending data to hardware,
    /// and return the result value when [`Command`] is a [`ReadWriteOp::Read`](crate::command::ReadWriteOp::Read) command
    fn send(&mut self, command: Command) -> Option<u8>;

    /// Wait specific duration, and send command
    async fn delay_and_send(
        &mut self,
        command_set: CommandSet,
        delayer: &mut Delayer,
        delay_us: u32,
    ) -> Option<u8> {
        delayer.delay_us(delay_us).await;
        self.send(command_set.into())
    }

    /// Check LCD busy state, when LCD is idle, send the command
    async fn wait_and_send(
        &mut self,
        command_set: CommandSet,
        delayer: &mut Delayer,
        poll_interval_us: u32,
    ) -> Option<u8> {
        self.wait_for_idle(delayer, poll_interval_us).await;

        let res = self.send(command_set.into());

        // According to ST6077U datasheet, after CommandSet::ClearDisplay or CommandSet::ReturnHome,
        // we will need to manually wait at least 1.52 ms before sending other command.
        // This only needs to happen if we don't know when the command finishes (aka not readable)
        if !READABLE
            && (command_set == CommandSet::ClearDisplay || command_set == CommandSet::ReturnHome)
        {
            self.wait_for_idle(delayer, poll_interval_us.max(1520)).await;
        }

        res
    }

    /// Wait in a busy loop, until LCD is idle
    ///
    /// If LCD driver is Write-Only, this method will wait one `poll_interval_us` and return.
    async fn wait_for_idle(&mut self, delayer: &mut Delayer, poll_interval_us: u32) {
        if READABLE {
            while self.check_busy() {
                delayer.delay_us(poll_interval_us).await;
            }
        } else {
            delayer.delay_us(poll_interval_us).await;
        }
    }

    /// Check LCD busy state
    ///
    /// If LCD driver is Write-Only, this method will return false instantly.
    fn check_busy(&mut self) -> bool {
        if READABLE {
            let busy_state = self
                .send(CommandSet::ReadBusyFlagAndAddress.into())
                .unwrap();
            matches!(busy_state.check_bit(7), BitState::Set)
        } else {
            false
        }
    }

    /// Get the current backlight
    ///
    /// Note:
    /// If a driver doesn't support read backlight state, just silently bypass it
    fn get_actual_backlight(&mut self) -> State {
        State::default()
    }

    /// Set the backlight
    ///
    /// Note:
    /// If a driver doesn't support change backlight, just silently bypass it
    #[allow(unused_variables)]
    fn set_actual_backlight(&mut self, backlight: State) {}
}
