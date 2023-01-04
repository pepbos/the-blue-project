#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;

use bluepill::{clock, delay::millis, gpio, Led};
use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    if cfg!(debug_assertions) {
        hprintln!("Hello! This is the Blink example.").unwrap();
    }

    // System setup.
    unsafe {
        clock::init();
        gpio::enable();
    }

    // LED on PC13.
    let mut led = Led::new(gpio::PC13, gpio::OutputMode::PushPull(gpio::Speed::Max2MHz));

    loop {
        // Blink led.
        millis(100);
        led.toggle();
    }
}
