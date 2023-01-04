use stm32f1xx_hal::pac::Peripherals as DevicePeripherals;
type GpioPtr = stm32f1xx_hal::pac::gpioa::RegisterBlock;
type GPIOA = stm32f1xx_hal::pac::GPIOA;
type GPIOB = stm32f1xx_hal::pac::GPIOB;
type GPIOC = stm32f1xx_hal::pac::GPIOC;

use super::Mode;

/// Available GPIO ports.
#[derive(Clone, Copy, Debug)]
pub enum Port {
    A,
    B,
    C,
}

/// Available GPIO pins.
#[repr(u8)]
#[rustfmt::skip]
#[derive(Clone, Copy, Debug)]
pub enum Pin {
    P0  = 0,
    P1  = 1,
    P2  = 2,
    P3  = 3,
    P4  = 4,
    P5  = 5,
    P6  = 6,
    P7  = 7,
    P8  = 8,
    P9  = 9,
    P10 = 10,
    P11 = 11,
    P12 = 12,
    P13 = 13,
    P14 = 14,
    P15 = 15,
}

impl Port {
    #[inline]
    fn ptr(self) -> *const GpioPtr {
        match self {
            Port::A => GPIOA::ptr(),
            Port::B => GPIOB::ptr(),
            Port::C => GPIOC::ptr(),
        }
    }

    #[inline]
    pub(crate) fn enable(self) {
        unsafe {
            let dp = DevicePeripherals::steal();
            match self {
                Port::A => dp.RCC.apb2enr.modify(|_, w| w.iopaen().enabled()),
                Port::B => dp.RCC.apb2enr.modify(|_, w| w.iopben().enabled()),
                Port::C => dp.RCC.apb2enr.modify(|_, w| w.iopcen().enabled()),
            }
        }
    }
}

fn crx_nibble(mode: Mode) -> u32 {
    match mode {
        Mode::OuputPushPull(speed) => 0 << 2 | (speed as u32),
        Mode::OutputOpenDrain(speed) => 1 << 2 | (speed as u32),
        Mode::AlternateFunctionOutputPushPull(speed) => 2 << 2 | (speed as u32),
        Mode::AlternateFunctionOutputOpenDrain(speed) => 3 << 2 | (speed as u32),
        Mode::AnalogInput => 0 << 2,
        Mode::FloatingInput => 1 << 2,
        Mode::InputPullDown => 2 << 2,
        Mode::InputPullUp => 2 << 2,
    }
}

/// Configure this gpio pin with the given mode.
#[inline]
pub(crate) fn configure(port: Port, pin: Pin, mode: Mode) {
    let pin_nr = pin as usize;
    let nibble = crx_nibble(mode);
    let port_ptr = port.ptr();
    if pin_nr < 8 {
        let shift = pin_nr * 4;
        let mask = !(15 << shift);
        let value = unsafe { (*port_ptr).crl.read().bits() };
        let new_value = (value & mask) | (nibble << shift);
        unsafe { (*port_ptr).crl.write(|w| w.bits(new_value)) };
    } else {
        let shift = (pin_nr - 8) * 4;
        let mask = !(15 << shift);
        let value = unsafe { (*port_ptr).crh.read().bits() };
        let new_value = (value & mask) | (nibble << shift);
        unsafe { (*port_ptr).crh.write(|w| w.bits(new_value)) };
    }
    match mode {
        Mode::InputPullUp => write(port, pin, true),
        Mode::InputPullDown => write(port, pin, false),
        _ => (),
    }
}

/// Sets the pin value.
///
/// Assumes the pin was configured as output mode.
#[inline]
pub(crate) fn write(port: Port, pin: Pin, value: bool) {
    unsafe {
        (*port.ptr()).odr.modify(|r, w| {
            w.bits(if value {
                r.bits() | (1 << pin as u8)
            } else {
                r.bits() & !(1 << pin as u8)
            })
        });
    }
}


/// Read the pin value.
#[inline]
pub(crate) fn read(port: Port, pin: Pin) -> bool {
    let value = unsafe { (*port.ptr()).idr.read().bits() };
    (value & (1 << pin as u8)) > 0
}
