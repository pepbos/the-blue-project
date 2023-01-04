use crate::gpio;
use crate::gpio::{PB10, PB11, PB6, PB7, PB8, PB9};
use stm32f1xx_hal::pac::{
    i2c1::RegisterBlock as Ptr, Peripherals as DevicePeripherals, I2C1, I2C2,
};

/// Regsiter controlled by the [I2C bus][super::Bus].
pub trait Register {
    fn adress(self) -> u8;
}

/// Device `WHO_AM_I` register.
///
/// Assumes the 7 lsb bits are the WHO_AM_I value, e.g. the msb bit is ignored:
///
/// ```
/// let who_am_i = WhoAmI(0b0111_1111);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct WhoAmI(pub u8);

impl WhoAmI {
    #[inline]
    pub(crate) fn transmit_address(&self) -> u8 {
        self.0 << 1
    }

    #[inline]
    pub(crate) fn receive_address(&self) -> u8 {
        self.0 << 1 | 1
    }
}

/// GPIO mapping for I2C1 peripheral.
#[allow(non_camel_case_types)]
pub enum Map1 {
    PB6_PB7,
    PB8_PB9,
}

/// Available I2C peripherals.
pub enum I2c {
    I2C1(Map1),
    I2C2,
}

/// I2C speed.
pub enum Speed {
    Fast400kHz,
    Std100kHz,
}

impl I2c {
    #[inline]
    fn ptr(&self) -> *const Ptr {
        match self {
            Self::I2C1(_) => I2C1::ptr(),
            Self::I2C2 => I2C2::ptr(),
        }
    }

    #[inline]
    pub(crate) fn enable_rcc(&self) {
        unsafe {
            let dp = DevicePeripherals::steal();
            match self {
                Self::I2C1(_) => dp.RCC.apb1enr.modify(|_, w| w.i2c1en().enabled()),
                Self::I2C2 => dp.RCC.apb1enr.modify(|_, w| w.i2c2en().enabled()),
            }
        }
    }

    #[inline]
    pub(crate) fn configure_gpio(&self) {
        unsafe {
            let dp = DevicePeripherals::steal();
            let (scl, sda) = match self {
                Self::I2C1(map) => {
                    match map {
                        Map1::PB6_PB7 => {
                            // No remap.
                            dp.AFIO.mapr.modify(|_, w| w.i2c1_remap().clear_bit());
                            (PB6, PB7)
                        }
                        Map1::PB8_PB9 => {
                            // Remap.
                            dp.AFIO.mapr.modify(|_, w| w.i2c1_remap().set_bit());
                            (PB8, PB9)
                        }
                    }
                }
                Self::I2C2 => (PB10, PB11),
            };
            gpio::configure(
                scl,
                gpio::Mode::AlternateFunctionOutputOpenDrain(gpio::Speed::Max50MHz),
            );
            gpio::configure(
                sda,
                gpio::Mode::AlternateFunctionOutputOpenDrain(gpio::Speed::Max50MHz),
            );
        }
    }

    #[inline]
    pub(crate) fn set_speed(&self, speed: Speed) {
        match speed {
            Speed::Std100kHz => self.set_standard_speed(),
            Speed::Fast400kHz => self.set_fast_speed(),
        }
    }

    #[inline]
    fn set_standard_speed(&self) {
        unsafe {
            // i2c frequency = 100kHz.
            let fclk: u32 = 100_000;
            // Set peripheral clock to 8MHz.
            let pclk = 8_000_000;
            (*self.ptr())
                .cr2
                .modify(|_, w| w.freq().bits((pclk / 1_000_000) as u8));
            // Set i2c clock frequency: ccr / pclk = 1 / ( 2 * fclk )
            let ccr = pclk / (fclk * 2);
            (*self.ptr()).ccr.write(|w| w.ccr().bits(ccr as u16)); // Overwrite register.
            // Set rise time.
            let trise = 10 * fclk;
            (*self.ptr())
                .trise
                .modify(|_, w| w.trise().bits((pclk / trise) as u8 + 1));
        }
    }

    #[inline]
    fn set_fast_speed(&self) {
        unsafe {
            // i2c frequency = 400kHz.
            let fclk: u32 = 400_000;
            // Set peripheral clock to 20MHz (must be multiple of 10MHz).
            let pclk = 20_000_000;
            (*self.ptr())
                .cr2
                .modify(|_, w| w.freq().bits((pclk / 1_000_000) as u8));
            // Set i2c clock frequency:
            // Th = 9 * ccr / pclk
            // Tl = 16 * ccr / pclk
            // fclk = 1 / ( Tl + Th ) = pclk / 25 / ccr
            // ccr = pclk / 25 / fclk
            let fmode = 1 << 15;
            let duty = 1 << 14;
            let ccr = pclk / (25 * fclk);
            (*self.ptr())
                .ccr
                .write(|w| w.ccr().bits((ccr | (fmode | duty)) as u16)); // Overwrite register.
            // Set rise time.
            let trise = 10 * fclk;
            (*self.ptr())
                .trise
                .modify(|_, w| w.trise().bits((pclk / trise) as u8 + 1));
        }
    }

    #[inline]
    pub(crate) fn enable(&self) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.pe().enabled());
        }
    }

    #[inline]
    pub(crate) fn disable(&self) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.pe().disabled());
        }
    }

    #[inline]
    pub(crate) fn busy(&self) -> bool {
        unsafe { (*self.ptr()).sr2.read().busy().bit_is_set() }
    }

    #[inline]
    pub(crate) fn master_transmit_data(
        &self,
        adress: WhoAmI,
        register: impl Register,
        data: &[u8],
        stop: bool,
    ) {
        unsafe {
            // Activate Acknowledge.
            (*self.ptr()).cr1.modify(|_, w| w.ack().ack());

            // Transmit START condition.
            // Automatically switches to MASTER mode.
            (*self.ptr()).cr1.modify(|_, w| w.start().set_bit());

            // Read SR1 to check completion of START transmission.
            while !(*self.ptr()).sr1.read().sb().bit_is_set() {}
            // Write slave adress.
            (*self.ptr())
                .dr
                .write(|w| w.dr().bits(adress.transmit_address()));

            // Read SR1 to check ADDRESS transmission completion.
            while !(*self.ptr()).sr1.read().addr().bit_is_set() {}
            // Read SR2 to activate data transmission.
            let _ = (*self.ptr()).sr2.read().bits();

            // Write first register byte.
            // Read SR1 to check if the transmission buffer is empty (TxE).
            while !(*self.ptr()).sr1.read().tx_e().is_empty() {}
            // Write data to DR.
            (*self.ptr()).dr.write(|w| w.dr().bits(register.adress()));

            // Write data to DR
            for byte in data.iter() {
                // Read SR1 to check if the transmission buffer is empty (TxE).
                while !(*self.ptr()).sr1.read().tx_e().is_empty() {}
                // Write data to DR.
                (*self.ptr()).dr.write(|w| w.dr().bits(*byte));
            }

            // Wait until byte transfer is complete (BTF).
            while !(*self.ptr()).sr1.read().btf().is_finished() {}

            // Write STOP condition, unless repeated start follows.
            if stop {
                (*self.ptr()).cr1.modify(|_, w| w.stop().stop());
            }
        }
    }

    #[inline]
    pub(crate) fn master_receive_data(&self, adress: WhoAmI, data: &mut [u8]) {
        let len = data.len();
        if len == 0 {
            return;
        }
        unsafe {
            // Activate Acknowledge.
            (*self.ptr()).cr1.modify(|_, w| w.ack().ack());

            // Transmit START condition.
            // Automatically switches to MASTER mode.
            (*self.ptr()).cr1.modify(|_, w| w.start().set_bit());

            // Read SR1 to check completion of START transmission.
            while !(*self.ptr()).sr1.read().sb().bit_is_set() {}
            // Write slave adress.
            (*self.ptr())
                .dr
                .write(|w| w.dr().bits(adress.receive_address()));

            // Read SR1 to check ADDRESS transmission completion.
            while !(*self.ptr()).sr1.read().addr().bit_is_set() {}
            // Read SR2 to activate data transmission.
            let _ = (*self.ptr()).sr2.read().bits();

            // If only one byte is received: Transmit Non-Acknowledge (NA), and write STOP.
            if data.len() == 1 {
                (*self.ptr()).cr1.modify(|_, w| w.ack().nak());
                (*self.ptr()).cr1.modify(|_, w| w.stop().stop());
            }

            // Read data from DR
            for (i, byte) in data.iter_mut().enumerate() {
                // Read SR1 to check if the receiver buffer is not empty (RxNE)
                while (*self.ptr()).sr1.read().rx_ne().is_empty() {}
                // Transmit Non-Acknowledge (NA) after reading second to last RxNE.
                if (len - 1) == (i + 1) {
                    (*self.ptr()).cr1.modify(|_, w| w.ack().nak());
                    (*self.ptr()).cr1.modify(|_, w| w.stop().stop());
                }
                // Read data from DR.
                *byte = (*self.ptr()).dr.read().dr().bits()
            }
        }
    }
}
