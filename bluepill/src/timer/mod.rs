pub mod encoder;
pub mod pwm;
mod timer;

pub use timer::{Channel, OutputCompareMode, Timer};

pub const TIM1: timer::Timer = timer::Timer::Tim1;
pub const TIM2: timer::Timer = timer::Timer::Tim2;
pub const TIM3: timer::Timer = timer::Timer::Tim3;
pub const TIM4: timer::Timer = timer::Timer::Tim4;
