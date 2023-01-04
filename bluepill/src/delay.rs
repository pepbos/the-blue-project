//! Block program for certain period of time.

const MILLIS_COUNT: u32 = 48_000; // SPEED / 1_000;
const MICROS_COUNT: u32 = 48; // SPEED / 1_000_000;

pub use cortex_m::asm::delay as delay;

/// Blocks program for *atleast* one millisecond.
#[inline]
pub fn millis(count: u32) {
    cortex_m::asm::delay(MILLIS_COUNT * count);
}

/// Blocks program for *atleast* one microsecond.
#[inline]
pub fn micros(count: u32) {
    cortex_m::asm::delay(MICROS_COUNT * count);
}
