//! SPI peripheral.
//!
//! Example use:
//!
//! ```
//! // Enable system clock.
//! clock::init();
//!
//! // Create spi bus.
//! let mut spi = spi::Config {
//!     speed: 1_000_000,
//!     mode: spi::Mode::Mode0,
//!     byteorder: spi::ByteOrder::MsbFirst,
//! }.make(spi::Spi2);
//!
//! // Fake device register.
//! let register = DeviceRegister(2); 
//!
//! // Write data to device.
//! let data = [3, 4];
//! bus.write(register, &data);
//! ```

mod pac;

pub use pac::{ByteOrder, Master, Mode, Port, Spi};

use crate::delay::micros;
use crate::gpio;
use cortex_m::interrupt;

/// Register controlled by the [spi bus][Bus].
pub trait Register {
    fn adress(self) -> u8;
}

/// Spi bus configuration.
///
/// Use [make][Self::make] for creating a new [spi bus][Bus].
#[derive(Copy, Clone, Debug)]
pub struct Config {
    /// Clock speed.
    ///
    /// The actual speed is obtained from the sysclk by division of factors of 2.
    /// For example:
    /// With the sysclk at 72MHz and the configured speed at 8MHz, the actual spi
    /// bus can operate at either 9MHz (div = 8) or 4.5MHz (div = 16). Of these two
    /// the lower is chosen. This means that despite configuring the speed to be
    /// 8MHz a 4.5MHz clock results.
    pub speed: u32,
    /// Spi mode.
    pub mode: Mode,
    /// Byte order: lsb or msb first.
    pub byteorder: ByteOrder,
}

impl Config {
    #[inline]
    pub fn make(self, spi: Spi) -> Bus {
        Bus::new(spi, self)
    }
}

/// Master SPI bus.
///
/// Does not support slave mode.
/// Does not control the chip select pin.
pub struct Bus {
    spi: Spi,
}

impl Bus {
    #[inline]
    pub fn new(spi: Spi, config: Config) -> Self {
        spi.configure(config, Master::Master);
        spi.enable();
        Self { spi }
    }

    /// Write multiple bytes to [Register].
    #[inline]
    pub fn write(&mut self, register: impl Register, data: &[u8]) {
        // 1. Assumed that: spi is enabled, CSN is pulled.

        // Enter interrupt free block (Critical Section).
        interrupt::free(|_cs| {
            // 2. Write first byte = register.
            self.spi.write_data_reg(register.adress());

            for byte in data {
                // 3. Wait for TXE == 1
                while !self.spi.tx_buffer_empty() {}
                // ... and write byte
                self.spi.write_data_reg(*byte);
                // ... and wait until RXNE == 1
                while !self.spi.rx_buffer_not_empty() {}
                // ... and read byte
                let _ = self.spi.read_data_reg();
                // ... repeat
            }

        // 4. Wait until RXNE=1
        while !self.spi.rx_buffer_not_empty() {}
        // ... and read last received data.
        let _ = self.spi.read_data_reg();
        // 5. wait until TXE == 1
        while !self.spi.tx_buffer_empty() {}
        // ... wait until BSY == 0
        while self.spi.busy() {}

        });

        // ... optionally disable the SPI
        // ... release CSN.
    }

    /// Read multiple bytes from [Register].
    #[inline]
    pub fn read(&mut self, register: impl Register, buffer: &mut [u8]) -> u8 {
        // 1. Optionally enable spi, pull CSN.

        // Enter interrupt free block (Critical Section).
        let len = buffer.len();
        let mut read_register = 0u8;
        interrupt::free(|_cs| {
            // 2. Write first byte = register.
            self.spi.write_data_reg(register.adress());

            for i in 0..len {
                // 3. Wait for TXE == 1
                while !self.spi.tx_buffer_empty() {}
                // ... and write byte
                self.spi.write_data_reg(0u8);
                // ... and wait until RXNE == 1
                while !self.spi.rx_buffer_not_empty() {}
                // ... and read byte
                let value = self.spi.read_data_reg();
                if i == 0 {
                    read_register = value;
                } else {
                    buffer[i - 1] = value;
                }
                // ... repeat
            }

        // 4. Wait untial RXNE=1
        while !self.spi.rx_buffer_not_empty() {}
        // ... and read last received data.
        buffer[len - 1] = self.spi.read_data_reg();
        // 5. wait until TXE == 1
        while !self.spi.tx_buffer_empty() {}
        // ... wait until BSY == 0
        while self.spi.busy() {}

        });

        // ... optionally disable the SPI
        // ... release CSN.
        read_register
    }

    /// Simultaneously read and write bytes to [register][Register].
    #[inline]
    pub fn write_and_read(
        &mut self,
        register: impl Register,
        write_data: &[u8],
        read_data: &mut [u8],
    ) -> u8 {
        // 1. Assumed that: spi is enabled, CSN is pulled.

        // Enter interrupt free block (Critical Section).
        let mut read_register = 0u8;
        let mut read_count = 0;
        interrupt::free(|_cs| {

            // 2. Write first byte = register.
            self.spi.write_data_reg(register.adress());

            for byte in write_data {
                // 3. Wait for TXE == 1
                while !self.spi.tx_buffer_empty() {}
                // ... and write byte
                self.spi.write_data_reg(*byte);
                // ... and wait until RXNE == 1
                while !self.spi.rx_buffer_not_empty() {}
                // ... and read byte
                if read_count == 0 {
                    read_register = self.spi.read_data_reg();
                } else {
                    read_data[read_count - 1] = self.spi.read_data_reg();
                }
                read_count += 1;
                // ... repeat
            }

        // 4. Wait until RXNE=1
        while !self.spi.rx_buffer_not_empty() {}
        // ... and read last received data.
        read_data[read_count - 1] = self.spi.read_data_reg();
        // 5. wait until TXE == 1
        while !self.spi.tx_buffer_empty() {}
        // ... wait until BSY == 0
        while self.spi.busy() {}
        });

        // ... optionally disable the SPI
        // ... release CSN.

        read_register
    }

    /// Write bytes without specifying the register.
    #[inline]
    pub fn write_direct(&mut self, data: &[u8]) {
        self.write(DebugRegister(data[0]), &data[1..]);
    }

    /// Read bytes without specifying the register.
    #[inline]
    pub fn read_direct(&mut self, buffer: &mut [u8]) {
        let reg = self.read(DebugRegister(0u8), &mut buffer[1..]);
        buffer[0] = reg;
    }

    /// Simultaneously read and write bytes without specifying the register.
    #[inline]
    pub fn write_and_read_direct(&mut self, write_data: &[u8], read_data: &mut [u8]) {
        let read_reg = self.write_and_read(
            DebugRegister(write_data[0]),
            &write_data[1..],
            &mut read_data[1..],
        );
        read_data[0] = read_reg;
    }

    /// Write single byte to [register][Register].
    #[inline]
    pub fn write_single(&mut self, register: impl Register, byte: u8) {
        self.write(register, &[byte]);
    }

    /// Read single byte from [register][Register].
    #[inline]
    pub fn read_single(&mut self, register: impl Register) -> u8 {
        let mut byte = [0];
        self.read(register, &mut byte);
        byte[0]
    }

    /// Write to register, and verify write by reading from register.
    #[inline]
    pub fn write_and_check(
        &mut self,
        write_register: impl Register + Copy,
        read_register: impl Register + Copy,
        expected: u8,
        wait_micros: u32,
        csn: Option<gpio::Gpio>,
    ) -> Result<(), Error> {
        micros(wait_micros);
        if let Some(pin) = csn {
            write_csn_and_wait(pin, false, wait_micros);
        }
        micros(wait_micros);
        self.write_single(write_register, expected);
        micros(wait_micros);
        if let Some(pin) = csn {
            write_csn_and_wait(pin, true, wait_micros);
        }
        micros(wait_micros);
        if let Some(pin) = csn {
            write_csn_and_wait(pin, false, wait_micros);
        }
        micros(wait_micros);
        let found = self.read_single(read_register);
        micros(wait_micros);
        if let Some(pin) = csn {
            write_csn_and_wait(pin, true, wait_micros);
        }
        micros(wait_micros);
        if found == expected {
            Ok(())
        } else {
            Err(Error { expected, found })
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Error {
    pub expected: u8,
    pub found: u8,
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

fn write_csn_and_wait(pin: gpio::Gpio, value: bool, wait_micros: u32) {
    gpio::write(pin, value);
    micros(wait_micros);
    while gpio::read(pin) != value {}
}
