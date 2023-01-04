//! System clock setup.

use stm32f1xx_hal::pac::Peripherals as DevicePeripherals;

/// System clock speed in Hertz.
pub const SPEED: u32 = 72_000_000;

/// Setup of the system clock.
///
/// Assumes a `16Mhz` external crystal is used.
pub unsafe fn init() {
    let dp = DevicePeripherals::steal();

    dp.FLASH.acr.write(|w| {
        // Enable the prefetch buffer.
        w.prftbe().set_bit();
        // Flash half cycle access: disabled.
        w.hlfcya().clear_bit();
        // Latency: two wait states if 48MHz < SYSCLK <= 72MHz.
        w.latency().ws2()
    });

    while !dp.FLASH.acr.read().latency().is_ws2() {}

    dp.RCC.cfgr.write(|w| {
        // HSE oscillator clock selected as PLL input clock.
        w.pllsrc().hse_div_prediv();
        // PLL multitplication factor: mul 9
        w.pllmul().mul9();
        // APB low-speed prescaler:  div 2
        w.ppre1().div2();
        // USB prescaler: PLL clock is divided by 1.5
        w.usbpre().div1_5()
    });

    // Enable HSE (crystal), PLL and clock security.
    dp.RCC.cr.write(|w| {
        w.csson().set_bit();
        w.hseon().set_bit();
        w.pllon().set_bit()
    });

    // Wait for PLL to become ready.
    while !dp.RCC.cr.read().pllrdy().is_ready() {}

    // Switch to PLL as system clock.
    dp.RCC.cfgr.modify(|_, w| w.sw().pll());

    // Wait for switch to complete.
    while !dp.RCC.cfgr.read().sws().is_pll() {}
}

/// Clock speed for Peripherals connected to APB1.
pub(crate) unsafe fn apb1_speed() -> u32 {
    let dp = DevicePeripherals::steal();
    let reg = dp.RCC.cfgr.read().ppre1().bits();
    if ( reg & 4 ) > 0 {
        SPEED >> ( (reg & 3) + 1 )
    } else {
        SPEED
    }
}

/// Clock speed for Peripherals connected to APB2.
pub(crate) unsafe fn apb2_speed() -> u32 {
    let dp = DevicePeripherals::steal();
    let reg = dp.RCC.cfgr.read().ppre2().bits();
    if ( reg & 4 ) > 0 {
        SPEED >> ( (reg & 3) + 1 )
    } else {
        SPEED
    }
}
