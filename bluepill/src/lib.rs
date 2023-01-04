//! Minimal HAL crate for the STM32f103 microcontroller.
//!
//! Allows for flexible peripheral access, without direct interaction with registers.
//! It is minimal in the sense that it does not follow strict guidelines on HAL crate design, e.g.
//! it is possible to create a gpio pin, without activating the system clock.

#![no_std]

pub mod clock;
pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod usb;
pub mod spi;
pub mod timer;
pub mod uart;

mod led;

pub use led::Led;
