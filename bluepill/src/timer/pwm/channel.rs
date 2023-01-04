use super::super::timer;
use crate::gpio;

#[derive(Clone, Debug)]
pub struct Channel {
    timer: timer::Timer,
    channel: timer::Channel,
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Pwm1,
    Pwm2,
}

impl Into<timer::OutputCompareMode> for Mode {
    #[inline]
    fn into(self) -> timer::OutputCompareMode {
        match self {
            Self::Pwm1 => timer::OutputCompareMode::Pwm1,
            Self::Pwm2 => timer::OutputCompareMode::Pwm2,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Polarity {
    ActiveHigh = 0,
    ActiveLow = 1,
}

impl Channel {
    #[inline]
    pub fn new(timer: timer::Timer, channel: timer::Channel) -> Self {
        Self { timer, channel }
    }

    #[inline]
    pub fn configure(
        &mut self,
        mode: Mode,
        polarity: Polarity,
        gpio_mode: gpio::AlternateFunctionOutputMode,
    ) {
        self.timer.output_compare_mode(self.channel, mode.into());
        self.timer.polarity(self.channel, polarity as u8 > 0);
        gpio::configure(self.timer.gpio(self.channel), gpio_mode.into());
        self.timer.output_enable(self.channel);
    }

    #[inline]
    pub fn write_ccr(&mut self, ccr: u16) {
        self.timer.write_ccr(self.channel, ccr);
    }

    #[inline]
    pub fn read_ccr(&self) -> u16 {
        self.timer.read_ccr(self.channel)
    }

    #[inline]
    pub fn read_arr(&self) -> u16 {
        self.timer.read_arr()
    }

    #[inline]
    pub fn gpio(&self) -> gpio::Gpio {
        self.timer.gpio(self.channel)
    }
}

impl core::ops::AddAssign<u16> for Channel {
    #[inline]
    fn add_assign(&mut self, rhs: u16) {
        let arr = self.read_arr();
        self.write_ccr(((rhs as u32 + self.read_ccr() as u32) % (arr as u32 + 1)) as u16);
    }
}
