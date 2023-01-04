#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;

use bluepill::{
    clock, delay, gpio,
    timer,
    timer::pwm,
    timer::TIM2,
};

#[entry]
fn main() -> ! {
    unsafe {
        clock::init();
        gpio::enable();
    }

    // Set the pwm frequency.
    let config = pwm::Config {
        psc: 0,
        arr: u16::MAX,
    };

    let mut pwm2 = config.make(TIM2);

    let mut channel = pwm::Channel::new(TIM2, timer::Channel::C1);

    channel.configure(
        pwm::Mode::Pwm1,
        pwm::Polarity::ActiveLow,
        gpio::AlternateFunctionOutputMode::OpenDrain(gpio::Speed::Max50MHz),
    );

    pwm2.enable();

    // Set the pwm value.
    channel.write_ccr(16_000);

    loop {
        delay::millis(1);
    }
}
