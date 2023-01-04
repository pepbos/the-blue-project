use super::super::timer;
use crate::gpio;

#[derive(Clone, Debug)]
pub struct Channel {
    timer: timer::Timer,
    channel: timer::Channel,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Polarity {
    NonInverted = 0,
    Inverted = 1,
}

impl Channel {
    #[inline]
    pub fn new(timer: timer::Timer, channel: timer::Channel) -> Self {
        Self { timer, channel }
    }

    #[inline]
    pub fn configure(&mut self, polarity: Polarity, gpio_mode: gpio::InputMode) {
        self.timer.polarity(self.channel, polarity as u8 > 0);
        gpio::configure(self.timer.gpio(self.channel), gpio_mode.into());
    }
}
