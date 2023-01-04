use crate::{clock, gpio};
use cortex_m::peripheral::NVIC;
use stm32f1xx_hal::pac::Interrupt;
use stm32f1xx_hal::pac::{Peripherals as DevicePeripherals, USART1, USART2, USART3};

use gpio::{Mode, OutputMode};

type UsartPtr = stm32f1xx_hal::pac::usart1::RegisterBlock;

/// Available USART peripherals.
#[derive(Copy, Clone, Debug)]
pub enum Usart {
    Usart1(Port),
    Usart2,
    Usart3,
}

/// Available GPIO ports for Usart1.
#[derive(Copy, Clone, Debug)]
pub enum Port {
    A,
    B,
}

impl Usart {
    /// Get the pointer.
    #[inline]
    fn ptr(&self) -> *const UsartPtr {
        match self {
            Self::Usart1(_) => return USART1::ptr(),
            Self::Usart2 => return USART2::ptr(),
            Self::Usart3 => return USART3::ptr(),
        }
    }

    pub fn get_tx_pin(&self) -> gpio::Gpio {
        match self {
            Self::Usart1(Port::A) => gpio::PA9,
            Self::Usart1(Port::B) => gpio::PB6,
            Self::Usart2 => gpio::PA2,
            Self::Usart3 => gpio::PB10,
        }
    }

    pub fn get_rx_pin(&self) -> gpio::Gpio {
        match self {
            Self::Usart1(Port::A) => gpio::PA10,
            Self::Usart1(Port::B) => gpio::PB7,
            Self::Usart2 => gpio::PA3,
            Self::Usart3 => gpio::PB11,
        }
    }

    pub fn configure_af_remap(&self) {
        unsafe {
            let dp = DevicePeripherals::steal();
            match self {
                Self::Usart1(Port::A) => dp.AFIO.mapr.modify(|_, w| w.usart1_remap().clear_bit()),
                Self::Usart1(Port::B) => dp.AFIO.mapr.modify(|_, w| w.usart1_remap().set_bit()),
                _ => (),
            }
        }
    }

    pub fn configure_tx_pin(&self, mode: OutputMode) {
        gpio::configure(self.get_tx_pin(), mode.as_af().into());
    }

    pub fn configure_rx_pin(&self) {
        gpio::configure(self.get_rx_pin(), Mode::FloatingInput);
    }

    #[inline]
    pub fn configure(&self, baudrate: u32) {
        unsafe {
            // Enable the peripheral.
            let dp = DevicePeripherals::steal();
            match self {
                Self::Usart1(_) => dp.RCC.apb2enr.modify(|_, w| w.usart1en().enabled()),
                Self::Usart2 => dp.RCC.apb1enr.modify(|_, w| w.usart2en().enabled()),
                Self::Usart3 => dp.RCC.apb1enr.modify(|_, w| w.usart3en().enabled()),
            }
            gpio::enable_alternate_function_io();
            self.configure_af_remap();

            // Baudrate register.
            // let peripheral_clock = crate::clock::SPEED / 2;
            let peripheral_clock = match self {
                Self::Usart1(_) => {
                    // Clock obtained from APB2.
                    clock::apb2_speed()
                }
                _ => {
                    // Clock obtained from APB1.
                    clock::apb1_speed()
                }
            };
            let divider = peripheral_clock / baudrate;
            (*self.ptr()).brr.modify(|_, w| {
                w.div_mantissa().bits((divider / 16) as u16);
                w.div_fraction().bits((divider % 16) as u8)
            });

            (*self.ptr()).cr1.modify(|_, w| {
                w.ue().enabled(); // Enable the USART.
                w.m().m8(); // 8 data bits.
                w.pce().disabled() // No parity check.
            });

            (*self.ptr()).cr2.modify(|_, w| {
                w.stop().stop1() // One stop bit.
            });
        }
    }

    #[inline]
    pub fn rx_enable(&self, enable: bool) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.re().bit(enable));
        }
    }

    #[inline]
    pub fn tx_enable(&self, enable: bool) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.te().bit(enable));
        }
    }

    #[inline]
    pub fn unmask_interrupts(&self) {
        unsafe {
            match self {
                Self::Usart1(_) => NVIC::unmask(Interrupt::USART1),
                Self::Usart2 => NVIC::unmask(Interrupt::USART2),
                Self::Usart3 => NVIC::unmask(Interrupt::USART3),
            }
        }
    }

    #[inline]
    pub fn mask_interrupts(&self) {
        match self {
            Self::Usart1(_) => NVIC::mask(Interrupt::USART1),
            Self::Usart2 => NVIC::mask(Interrupt::USART2),
            Self::Usart3 => NVIC::mask(Interrupt::USART3),
        }
    }

    #[inline]
    pub fn rx_interrupt_enable(&self, enable: bool) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.rxneie().bit(enable));
        }
    }

    #[inline]
    pub fn tx_interrupt_enable(&self, enable: bool) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.txeie().bit(enable));
        }
    }

    #[inline]
    pub fn write_data_reg(&self, byte: u8) {
        unsafe {
            (*self.ptr()).dr.write(|w| w.dr().bits(byte as u16));
        }
    }

    #[inline]
    pub fn read_data_reg(&self) -> u8 {
        unsafe { (*self.ptr()).dr.read().bits() as u8 }
    }

    #[inline]
    pub fn rx_buffer_not_empty(&self) -> bool {
        unsafe { (*self.ptr()).sr.read().rxne().bit_is_set() }
    }

    #[inline]
    pub fn tx_buffer_empty(&self) -> bool {
        unsafe { (*self.ptr()).sr.read().txe().bit_is_set() }
    }

    #[inline]
    pub fn is_transmission_complete(&self) -> bool {
        unsafe { (*self.ptr()).sr.read().tc().bit_is_set() }
    }
}
