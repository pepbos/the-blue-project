use crate::gpio;
use stm32f1xx_hal::pac::{
    tim1::RegisterBlock as RegisterBlock1, tim2::RegisterBlock as RegisterBlock2,
    Peripherals as DevicePeripherals, TIM1, TIM2, TIM3, TIM4,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SlaveMode {
    Disabled = 0,
    Encoder1 = 1,
    Encoder2 = 2,
    Encoder3 = 3,
    ResetMode = 4,
    GatedMode = 5,
    TriggerMode = 6,
    ExternalClockMode = 7,
}

enum TimerPtr {
    Tim1(*const RegisterBlock1),
    Tim234(*const RegisterBlock2),
}

#[derive(Clone, Copy, Debug)]
pub enum Timer {
    Tim1,
    Tim2,
    Tim3,
    Tim4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Channel {
    C1 = 0,
    C2 = 1,
    C3 = 2,
    C4 = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum OutputCompareMode {
    Frozen = 0,
    ActiveMatch = 1,
    InactiveMatch = 2,
    Toggle = 3,
    ForceInactive = 4,
    ForceActive = 5,
    Pwm1 = 6,
    Pwm2 = 7,
}

impl Timer {
    #[inline]
    fn ptr(&self) -> TimerPtr {
        match self {
            Timer::Tim1 => TimerPtr::Tim1(TIM1::ptr()),
            Timer::Tim2 => TimerPtr::Tim234(TIM2::ptr()),
            Timer::Tim3 => TimerPtr::Tim234(TIM3::ptr()),
            Timer::Tim4 => TimerPtr::Tim234(TIM4::ptr()),
        }
    }

    #[inline]
    pub fn enable_rcc(&mut self) {
        unsafe {
            let dp = DevicePeripherals::steal();
            match self {
                Timer::Tim1 => dp.RCC.apb2enr.modify(|_, w| w.tim1en().enabled()),
                Timer::Tim2 => dp.RCC.apb1enr.modify(|_, w| w.tim2en().enabled()),
                Timer::Tim3 => dp.RCC.apb1enr.modify(|_, w| w.tim3en().enabled()),
                Timer::Tim4 => dp.RCC.apb1enr.modify(|_, w| w.tim4en().enabled()),
            }
        }
    }

    #[inline]
    pub fn write_arr(&mut self, arr: u16) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).arr.write(|w| w.arr().bits(arr)),
                TimerPtr::Tim234(ptr) => (*ptr).arr.write(|w| w.arr().bits(arr)),
            }
        }
    }

    #[inline]
    pub fn read_arr(&self) -> u16 {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).arr.read().bits() as u16,
                TimerPtr::Tim234(ptr) => (*ptr).arr.read().bits() as u16,
            }
        }
    }

    #[inline]
    pub fn write_psc(&mut self, psc: u16) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).psc.write(|w| w.bits(psc as u32)),
                TimerPtr::Tim234(ptr) => (*ptr).psc.write(|w| w.bits(psc as u32)),
            }
        }
    }

    #[inline]
    pub fn read_psc(&mut self) -> u16 {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).psc.read().bits() as u16,
                TimerPtr::Tim234(ptr) => (*ptr).psc.read().bits() as u16,
            }
        }
    }

    #[inline]
    pub fn enable(&mut self) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).cr1.modify(|_, w| w.cen().enabled()),
                TimerPtr::Tim234(ptr) => (*ptr).cr1.modify(|_, w| w.cen().enabled()),
            }
        }
    }

    #[inline]
    pub fn disable(&mut self) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).cr1.modify(|_, w| w.cen().disabled()),
                TimerPtr::Tim234(ptr) => (*ptr).cr1.modify(|_, w| w.cen().disabled()),
            }
        }
    }

    #[inline]
    pub fn read_counter_value(&self) -> u16 {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).cnt.read().bits() as u16,
                TimerPtr::Tim234(ptr) => (*ptr).cnt.read().bits() as u16,
            }
        }
    }

    #[inline]
    pub fn write_slave_mode(&mut self, mode: SlaveMode) {
        let x = mode as u32;
        let mask = !7;
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => {
                    let value = (*ptr).smcr.read().bits() & mask;
                    (*ptr).smcr.modify(|_, w| w.bits(x | value));
                }
                TimerPtr::Tim234(ptr) => {
                    let value = (*ptr).smcr.read().bits() & mask;
                    (*ptr).smcr.modify(|_, w| w.bits(x | value));
                }
            }
        }
    }

    #[inline]
    pub fn gpio(&self, channel: Channel) -> gpio::Gpio {
        match self {
            Timer::Tim1 => match channel {
                Channel::C1 => gpio::PA8,
                Channel::C2 => gpio::PA9,
                Channel::C3 => gpio::PA10,
                Channel::C4 => gpio::PA11,
            },
            Timer::Tim2 => match channel {
                Channel::C1 => gpio::PA0,
                Channel::C2 => gpio::PA1,
                Channel::C3 => gpio::PA2,
                Channel::C4 => gpio::PA3,
            },
            Timer::Tim3 => match channel {
                Channel::C1 => gpio::PA6,
                Channel::C2 => gpio::PA7,
                Channel::C3 => gpio::PB0,
                Channel::C4 => gpio::PB1,
            },
            Timer::Tim4 => match channel {
                Channel::C1 => gpio::PB6,
                Channel::C2 => gpio::PB7,
                Channel::C3 => gpio::PB8,
                Channel::C4 => gpio::PB9,
            },
        }
    }

    #[inline]
    pub fn output_compare_mode(&self, channel: Channel, mode: OutputCompareMode) {
        let shift = match channel {
            Channel::C1 | Channel::C3 => 4,
            Channel::C2 | Channel::C4 => 4 + 8,
        };
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => match channel {
                    Channel::C1 | Channel::C2 => {
                        let value = (*ptr).ccmr1_output().read().bits() & !(7 << shift);
                        (*ptr)
                            .ccmr1_output()
                            .modify(|_, w| w.bits(value | ((mode as u32) << shift)));
                    }
                    Channel::C3 | Channel::C4 => {
                        let value = (*ptr).ccmr2_output().read().bits() & !(7 << shift);
                        (*ptr)
                            .ccmr2_output()
                            .modify(|_, w| w.bits(value | ((mode as u32) << shift)));
                    }
                },
                TimerPtr::Tim234(ptr) => match channel {
                    Channel::C1 | Channel::C2 => {
                        let value = (*ptr).ccmr1_output().read().bits() & !(7 << shift);
                        (*ptr)
                            .ccmr1_output()
                            .modify(|_, w| w.bits(value | ((mode as u32) << shift)));
                    }
                    Channel::C3 | Channel::C4 => {
                        let value = (*ptr).ccmr2_output().read().bits() & !(7 << shift);
                        (*ptr)
                            .ccmr2_output()
                            .modify(|_, w| w.bits(value | ((mode as u32) << shift)));
                    }
                },
            }
        }
    }

    #[inline]
    pub fn output_enable(&mut self, channel: Channel) {
        unsafe {
            let x = 1 << (4 * channel as u8);
            match self.ptr() {
                TimerPtr::Tim1(ptr) => {
                    // Main output enable.
                    (*ptr).bdtr.modify(|_, w| w.moe().set_bit());
                    let value = (*ptr).ccer.read().bits();
                    (*ptr).ccer.modify(|_, w| w.bits(x | value));
                }
                TimerPtr::Tim234(ptr) => {
                    let value = (*ptr).ccer.read().bits();
                    (*ptr).ccer.modify(|_, w| w.bits(x | value));
                }
            }
        }
    }

    #[inline]
    pub fn polarity(&self, channel: Channel, pol: bool) {
        unsafe {
            let shift = 1 + 4 * channel as u8;
            let x = (pol as u32) << shift;
            let mask = !(1 << shift);
            match self.ptr() {
                TimerPtr::Tim1(ptr) => {
                    let value = (*ptr).ccer.read().bits() & mask;
                    (*ptr).ccer.modify(|_, w| w.bits(x | value));
                }
                TimerPtr::Tim234(ptr) => {
                    let value = (*ptr).ccer.read().bits() & mask;
                    (*ptr).ccer.modify(|_, w| w.bits(x | value));
                }
            }
        }
    }

    #[inline]
    pub fn write_ccr(&self, channel: Channel, ccr: u16) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => match channel {
                    Channel::C1 => (*ptr).ccr1.write(|w| w.bits(ccr as u32)),
                    Channel::C2 => (*ptr).ccr2.write(|w| w.bits(ccr as u32)),
                    Channel::C3 => (*ptr).ccr3.write(|w| w.bits(ccr as u32)),
                    Channel::C4 => (*ptr).ccr4.write(|w| w.bits(ccr as u32)),
                },
                TimerPtr::Tim234(ptr) => match channel {
                    Channel::C1 => (*ptr).ccr1.write(|w| w.bits(ccr as u32)),
                    Channel::C2 => (*ptr).ccr2.write(|w| w.bits(ccr as u32)),
                    Channel::C3 => (*ptr).ccr3.write(|w| w.bits(ccr as u32)),
                    Channel::C4 => (*ptr).ccr4.write(|w| w.bits(ccr as u32)),
                },
            }
        }
    }

    #[inline]
    pub fn read_ccr(&self, channel: Channel) -> u16 {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => match channel {
                    Channel::C1 => (*ptr).ccr1.read().bits() as u16,
                    Channel::C2 => (*ptr).ccr2.read().bits() as u16,
                    Channel::C3 => (*ptr).ccr3.read().bits() as u16,
                    Channel::C4 => (*ptr).ccr4.read().bits() as u16,
                },
                TimerPtr::Tim234(ptr) => match channel {
                    Channel::C1 => (*ptr).ccr1.read().bits() as u16,
                    Channel::C2 => (*ptr).ccr2.read().bits() as u16,
                    Channel::C3 => (*ptr).ccr3.read().bits() as u16,
                    Channel::C4 => (*ptr).ccr4.read().bits() as u16,
                },
            }
        }
    }

    #[inline]
    pub fn update_interrupt_enable(&self) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).dier.modify(|_, w| w.uie().enabled()),
                TimerPtr::Tim234(ptr) => (*ptr).dier.modify(|_, w| w.uie().enabled()),
            }
        }
    }

    #[inline]
    pub fn read_update_interrupt_flag(&self) -> bool {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).sr.read().uif().bit_is_set(),
                TimerPtr::Tim234(ptr) => (*ptr).sr.read().uif().bit_is_set(),
            }
        }
    }

    #[inline]
    pub fn clear_update_interrupt_flag(&self) {
        unsafe {
            match self.ptr() {
                TimerPtr::Tim1(ptr) => (*ptr).sr.modify(|_, w| w.uif().clear_bit()),
                TimerPtr::Tim234(ptr) => (*ptr).sr.modify(|_, w| w.uif().clear_bit()),
            }
        }
    }
}
