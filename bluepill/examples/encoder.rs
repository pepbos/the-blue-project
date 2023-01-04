#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;

use bluepill::{
    clock,
    delay::millis,
    gpio,
    timer::{encoder, TIM1, TIM2, TIM3, TIM4},
};
use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    // System setup.
    unsafe {
        clock::init();
        gpio::enable();
    }

    // Let counter overflow at 255.
    let config = encoder::Config { psc: 0, arr: 255 };

    let mut encoder = [
        config.make(TIM1),
        config.make(TIM2),
        config.make(TIM3),
        config.make(TIM4),
    ];

    // Configure the channels, and enable.
    encoder.iter_mut().for_each(|e| {
        let channels = e.channels();
        channels[0].configure(encoder::Polarity::Inverted, gpio::InputMode::FloatingInput);
        channels[1].configure(
            encoder::Polarity::NonInverted,
            gpio::InputMode::FloatingInput,
        );
        e.enable();
    });

    loop {
        millis(100);

        if cfg!(debug_assertions) {
            hprintln!("encoder1 = {}", encoder[0].read_counter_value()).unwrap();
            hprintln!("encoder2 = {}", encoder[1].read_counter_value()).unwrap();
            hprintln!("encoder3 = {}", encoder[2].read_counter_value()).unwrap();
            hprintln!("encoder4 = {}", encoder[3].read_counter_value()).unwrap();
        }
    }
}
