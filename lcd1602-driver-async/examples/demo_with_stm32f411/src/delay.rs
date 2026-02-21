use core::default::Default;

use embassy_time::Timer;
use embedded_hal_async::delay::DelayNs;

/// A delay implementation using embassy-time
pub struct EmbassyDelay;

impl EmbassyDelay {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmbassyDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl DelayNs for EmbassyDelay {
    async fn delay_ns(&mut self, ns: u32) {
        // embassy-time has a minimum resolution of microseconds
        // for better precision, we use the appropriate timer method
        if ns >= 1_000 {
            // Convert to micros for better precision with embassy-time
            Timer::after_nanos(ns as u64).await;
        } else {
            // For very small delays, just do a tiny busy wait
            // embassy-time doesn't support sub-microsecond delays well
            Timer::after_nanos(1).await;
        }
    }
}
