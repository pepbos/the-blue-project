//! USART peripheral.
//!
//! Example use:
//!
//! ```
//! // Enable system clock.
//! clock::init();
//!
//! // Create usart bus.
//! let peripheral = usart::Usart::Usart2;
//! let mut bus = usart::Config {
//!     baudrate: 1_000_000,
//!     tx_pin: OutputMode::PushPull(Speed::Max50MHz),
//! }.make(peripheral);
//!
//! // Write data to bus.
//! let data = [3, 4];
//! bus.write_bytes(&data);
//! ```

mod pac;

pub use pac::{Port, Usart};
use gpio::{OutputMode, InputMode};

use crate::gpio;

/// Usart peripheral configuration.
///
/// Use [make][Config::make()] to create a new [Bus].
#[derive(Copy, Clone, Debug)]
pub struct Config {
    /// Baudrate.
    pub baudrate: u32,
    /// Set output mode of the TX pin.
    pub tx_pin: OutputMode,
}

impl Config {
    #[inline]
    pub fn make(self, usart: Usart) -> Bus {
        Bus::new(usart, self)
    }
}

/// Uart bus.
///
/// Can be constructed using [Config][Config::make()].
pub struct Bus {
    usart: Usart,
    tx_pin: OutputMode,
}

impl Bus {
    #[inline]
    pub fn new(usart: Usart, config: Config) -> Self {
        usart.configure(config.baudrate);
        Self {
            usart,
            tx_pin: config.tx_pin,
        }
    }
}

impl Bus {
    /// Read byte received byte.
    ///
    /// Returns None if buffer is empty.
    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.usart.rx_buffer_not_empty() {
            Some(self.usart.read_data_reg())
        } else {
            None
        }
    }

    /// Write byte.
    ///
    /// Returns Error if buffer is not empty.
    #[inline]
    pub fn write_byte(&mut self, byte: u8) -> Result<(), ()> {
        if self.usart.tx_buffer_empty() {
            self.usart.write_data_reg(byte);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Returns TX pin of current USART peripheral.
    #[inline]
    pub fn get_tx_pin(&self) -> gpio::Gpio {
       self.usart.get_tx_pin()
    }

    /// Returns RX pin of current USART peripheral.
    #[inline]
    pub fn get_rx_pin(&self) -> gpio::Gpio {
       self.usart.get_rx_pin()
    }

    /// Enable or disable receiver.
    #[inline]
    pub fn rx_enable(&mut self, enable: bool) {
        self.usart.rx_enable(enable);
    }

    /// Enable or disable transmitter.
    ///
    /// When disabled, the TX pin will be configured as floating input.
    #[inline]
    pub fn tx_enable(&mut self, enable: bool) {
        self.usart.tx_enable(enable);
        if enable {
            self.usart.configure_tx_pin(self.tx_pin);
        } else {
            gpio::configure(
                self.usart.get_tx_pin(),
                InputMode::FloatingInput.into(),
            );
        }
    }

    /// Blocking read.
    ///
    /// If there is no byte, keep waiting.
    #[inline]
    pub fn wait_read_byte(&mut self) -> u8 {
        loop {
            if let Some(byte) = self.read_byte() {
                return byte;
            }
        }
    }

    /// Blocking write byte.
    ///
    /// Blocks until the byte has been written to the transmit buffer.
    #[inline]
    pub fn wait_write_byte(&mut self, byte: u8) {
        while self.write_byte(byte).is_err() {}
    }

    /// Write multiple bytes.
    ///
    /// This method blocks until all bytes have been transmitted.
    #[inline]
    pub fn write_bytes(&mut self, data: &[u8]) {
        for &byte in data {
            self.wait_write_byte(byte);
        }
    }

    // Enable or disable interrupts.
    #[inline]
    pub fn set_intterupts_mask(&mut self, mask: bool) {
        if mask {
            self.mask_interrupts();
        } else {
            self.unmask_interrupts();
        }
    }

    #[inline]
    pub fn unmask_interrupts(&mut self) {
        self.usart.unmask_interrupts();
    }

    #[inline]
    pub fn mask_interrupts(&mut self) {
        self.usart.mask_interrupts();
    }

    #[inline]
    pub fn rx_interrupt_enable(&mut self, enable: bool) {
        self.usart.rx_interrupt_enable(enable)
    }

    #[inline]
    pub fn tx_interrupt_enable(&mut self, enable: bool) {
        self.usart.rx_interrupt_enable(enable)
    }
}
