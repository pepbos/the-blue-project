#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;

use bluepill::{
    clock,
    delay::millis,
    gpio,
    i2c::{Bus, DebugRegister, I2c, Map1, Speed as I2cSpeed, WhoAmI},
};

#[entry]
fn main() -> ! {
    unsafe {
        clock::init();
        gpio::enable();
    }

    let mut bus = Bus::new(I2c::I2C1(Map1::PB8_PB9), I2cSpeed::Std100kHz);

    let who_am_i = WhoAmI(1);
    let register = DebugRegister(2);

    bus.write(who_am_i, register, &[3, 4]);
    loop {
        millis(1);
        bus.write(who_am_i, register, &[3, 4]);
        millis(1);
        let mut data = [0u8, 0u8];
        bus.read(who_am_i, register, &mut data);
    }
}
