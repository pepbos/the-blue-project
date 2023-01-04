mod channel;

use super::timer;
pub use channel::{Channel, Polarity};

#[derive(Clone, Debug)]
pub struct Encoder {
    timer: timer::Timer,
    channels: [Channel; 2],
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub psc: u16,
    pub arr: u16,
}

impl Config {
    #[inline]
    pub fn make(self, timer: timer::Timer) -> Encoder {
        Encoder::new(timer, self)
    }
}

impl Encoder {
    #[inline]
    pub fn new(mut timer: timer::Timer, config: Config) -> Self {
        timer.enable_rcc();
        timer.write_arr(config.arr);
        timer.write_psc(config.psc);
        timer.write_slave_mode(timer::SlaveMode::Encoder3);
        Self {
            timer,
            channels: [
                Channel::new(timer, timer::Channel::C1),
                Channel::new(timer, timer::Channel::C2),
            ],
        }
    }

    #[inline]
    pub fn enable(&mut self) {
        self.timer.enable();
    }

    #[inline]
    pub fn disable(&mut self) {
        self.timer.disable();
    }

    #[inline]
    pub fn read_counter_value(&self) -> u16 {
        self.timer.read_counter_value()
    }

    #[inline]
    pub fn channels<'a>(&'a mut self) -> &'a mut [Channel; 2] {
        &mut self.channels
    }
}
