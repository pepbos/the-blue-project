#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;

use bluepill::{clock, gpio};
use cortex_m_semihosting::hprintln;

// LED on PC13.
const LED_PIN: gpio::Gpio = gpio::PC13;

// Input pin on PA0
const INPUT_PIN: gpio::Gpio = gpio::PA0;

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

    // Configure GPIO modes.
    gpio::configure(LED_PIN, gpio::Mode::OuputPushPull(gpio::Speed::Max2MHz));
    gpio::configure(INPUT_PIN, gpio::Mode::FloatingInput);

    loop {
        // Read input pin.
        let value = gpio::read(INPUT_PIN);

        // Control LED with Pin.
        gpio::write(LED_PIN, value);
    }
}
