mod channel;

use super::timer;
pub use channel::{Channel, Mode, Polarity};

pub struct Pwm {
    timer: timer::Timer,
    channels: [Channel; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub psc: u16,
    pub arr: u16,
}

impl Config {
    #[inline]
    pub fn make(self, timer: timer::Timer) -> Pwm {
        Pwm::new(timer, self)
    }
}

impl Pwm {
    #[inline]
    pub fn new(mut timer: timer::Timer, config: Config) -> Self {
        timer.enable_rcc();
        timer.write_arr(config.arr);
        timer.write_psc(config.psc);
        Self {
            timer,
            channels: [
                Channel::new(timer, timer::Channel::C1),
                Channel::new(timer, timer::Channel::C2),
                Channel::new(timer, timer::Channel::C3),
                Channel::new(timer, timer::Channel::C4),
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
    pub fn channels<'a>(&'a mut self) -> &'a mut [Channel; 4] {
        &mut self.channels
    }

    #[inline]
    pub fn into_channels(self) -> [Channel; 4] {
        self.channels
    }
}
