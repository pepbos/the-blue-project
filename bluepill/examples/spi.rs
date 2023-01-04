#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;

use bluepill::{clock, delay::micros, gpio, spi, Led};

const SPI: spi::Spi = spi::Spi::Spi2;
const CSN: gpio::Gpio = gpio::PB12;

#[entry]
fn main() -> ! {
    unsafe {
        clock::init();
        gpio::enable();
    }

    let mut led = Led::new(gpio::PC13, gpio::OutputMode::PushPull(gpio::Speed::Max2MHz));

    let mut spi = spi::Config {
        speed: 1_000_000,
        mode: spi::Mode::Mode0,
        byteorder: spi::ByteOrder::MsbFirst,
    }
    .make(SPI);

    gpio::configure(CSN, gpio::Mode::OuputPushPull(gpio::Speed::Max10MHz));

    let byte = 0b0101_0101;
    let reg = spi::DebugRegister(byte);
    loop {
        led.on();
        // Arbitrary data.
        let mut data = [19, 20];
        // Pull chip-select.
        gpio::write(CSN, false);
        // Write data over spi.
        spi.write(reg, &data);
        // Read data over spi.
        spi.read(reg, &mut data);
        // Set chip-select.
        gpio::write(CSN, true);
        led.off();
        micros(100);
    }
}
