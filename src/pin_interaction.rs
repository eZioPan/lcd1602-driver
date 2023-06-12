use super::full_command::FullCommand;

pub trait PinsInteraction {
    fn delay_and_send(&mut self, command: impl Into<FullCommand>, wait_ms: u32) -> Option<u8>;
    fn wait_and_send(&mut self, command: impl Into<FullCommand>) -> Option<u8>;
    fn wait_for_idle(&mut self);
    fn check_busy(&mut self) -> bool;
}
