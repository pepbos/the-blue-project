//! I2C peripheral.
//!
//! Example use:
//!
//! ```
//! // Enable system clock.
//! clock::init();
//!
//! // Create I2C bus.
//! let mut bus = Bus::new(I2c::I2C1(Map1::PB8_PB9), I2cSpeed::Std100kHz);
//!
//! // Device specific registers.
//! let who_am_i = WhoAmI(1);
//! let register = DeviceRegister(2); // (fake device register)
//!
//! // Write data to device.
//! let data = [3, 4];
//! bus.write(who_am_i, register, &data);
//! ```

mod pac;

pub use pac::{I2c, Map1, Register, Speed, WhoAmI};

/// Master I2C bus.
///
/// Does not support slave mode.
pub struct Bus {
    i2c: I2c,
}

impl Bus {
    /// Enable I2C peripheral, and map GPIO pin.
    #[inline]
    pub fn new(i2c: I2c, speed: Speed) -> Self {
        i2c.enable_rcc();
        i2c.set_speed(speed);
        i2c.configure_gpio();
        i2c.enable();
        Self { i2c }
    }

    /// Returns whether peripheral is busy.
    #[inline]
    pub fn busy(&self) -> bool {
        self.i2c.busy()
    }

    /// Write multiple bytes to [Register] of device with id [WhoAmI].
    #[inline]
    pub fn write(&mut self, address: WhoAmI, register: impl Register, data: &[u8]) {
        self.i2c.master_transmit_data(address, register, data, true);
    }

    /// Read multiple bytes from [Register] of device with id [WhoAmI].
    #[inline]
    pub fn read(&self, address: WhoAmI, register: impl Register, data: &mut [u8]) {
        self.i2c.master_transmit_data(address, register, &[], false);
        self.read_direct(address, data);
    }

    /// Read multiple bytes from device with id [WhoAmI], without specifying the register.
    ///
    /// Some simple devices have only one register to read from, in which case it is often ommited.
    #[inline]
    pub fn read_direct(&self, address: WhoAmI, data: &mut [u8]) {
        self.i2c.master_receive_data(address, data);
    }

    /// Read [Register] value from device with id [WhoAmI].
    #[inline]
    pub fn read_single(&self, address: WhoAmI, register: impl Register) -> u8 {
        let mut data = [0u8];
        self.read(address, register, &mut data);
        data[0]
    }

    /// Write byte to [Register] of device with id [WhoAmI].
    #[inline]
    pub fn write_single(&mut self, address: WhoAmI, register: impl Register, value: u8) {
        self.i2c
            .master_transmit_data(address, register, &[value], true);
    }

    /// Disable peripheral.
    #[inline]
    pub fn disable(&mut self) {
        self.i2c.disable()
    }
}

/// Dummy register for debugging purposes.
#[derive(Copy, Clone, Debug)]
pub struct DebugRegister(pub u8);

impl Register for DebugRegister {
    #[inline]
    fn adress(self) -> u8 {
        self.0
    }
}
