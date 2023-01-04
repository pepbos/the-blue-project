//! GPIO peripheral.
//!
//! Example usage:
//!
//! ```
//! clock::init();
//! gpio::enable();
//!
//! gpio::configure(PC13, gpio::Mode::FloatingInput);
//! let value: bool = gpio::read(PC13);
//! ```

mod pac;
mod pinout;
mod mode;

pub use pac::{Pin, Port};
pub use pinout::*;
pub use mode::*;

use stm32f1xx_hal::pac::Peripherals as DevicePeripherals;

/// Enable GPIO ports.
///
/// Enables ports A, B and C, and enables the alternate function IO peripheral.
#[inline]
pub fn enable() {
    Port::A.enable();
    Port::B.enable();
    Port::C.enable();
    enable_alternate_function_io();
}

/// GPIO pin tuple struct.
///
/// Can be used to [configure][configure()], [read][read()] from or
/// [write][write()] to a pin.
#[derive(Clone, Copy, Debug)]
pub struct Gpio(pub Port, pub Pin);

/// Configure the given GPIO pin mode.
#[inline]
pub fn configure(pin: Gpio, mode: Mode) {
    pac::configure(pin.0, pin.1, mode);
}

/// Set the GPIO pin value.
///
/// Assumes pin was [configured][configure] as [output][OutputMode] before calling this.
#[inline]
pub fn write(pin: Gpio, value: bool) {
    pac::write(pin.0, pin.1, value)
}

/// Read the GPIO pin value.
#[inline]
pub fn read(pin: Gpio) -> bool {
    pac::read(pin.0, pin.1)
}

/// Enable the alternate function IO peripheral.
#[inline]
pub fn enable_alternate_function_io() {
    unsafe {
        let dp = DevicePeripherals::steal();
        dp.RCC.apb2enr.modify(|_, w| w.afioen().enabled());
    }
}

/// Remaps the JTAG pins as regular GPIO.
#[inline]
pub fn free_jtag() {
    unsafe {
        let dp = DevicePeripherals::steal();
        dp.AFIO.mapr.modify(|_, w| w.swj_cfg().bits(2));
    }
}
