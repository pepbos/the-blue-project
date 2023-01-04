use super::Config;
use crate::clock;
use crate::gpio;
use stm32f1xx_hal::pac::{Peripherals as DevicePeripherals, SPI1, SPI2};

type SpiPtr = stm32f1xx_hal::pac::spi1::RegisterBlock;

/// SPI master or slave mode.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Master {
    Master = 0,
    Slave = 1,
}

/// SPI mode.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Mode0 = 0,
    Mode1 = 1,
    Mode2 = 2,
    Mode3 = 3,
}

/// SPI transmission byte order.
#[derive(Copy, Clone, Debug)]
pub enum ByteOrder {
    MsbFirst,
    LsbFirst,
}

/// SPI peripheral.
#[derive(Copy, Clone, Debug)]
pub enum Spi {
    Spi1(Port),
    Spi2,
}

/// GPIO port for SPI1.
#[derive(Copy, Clone, Debug)]
pub enum Port {
    A,
    B,
}

impl Spi {
    #[inline]
    pub fn ptr(&self) -> *const SpiPtr {
        match self {
            Self::Spi1(_) => return SPI1::ptr(),
            Self::Spi2 => return SPI2::ptr(),
        }
    }

    #[inline]
    pub fn configure(&self, config: Config, mode: Master) {
        unsafe {
            // Configure the GPIO.
            match self {
                Self::Spi1(Port::A) => {
                    gpio_configuration(
                        gpio::Port::A,
                        gpio::Pin::P5,
                        gpio::Pin::P6,
                        gpio::Pin::P7,
                        mode,
                    );
                }
                Self::Spi1(Port::B) => {
                    gpio_configuration(
                        gpio::Port::B,
                        gpio::Pin::P3,
                        gpio::Pin::P4,
                        gpio::Pin::P5,
                        mode,
                    );
                }
                Self::Spi2 => {
                    gpio_configuration(
                        gpio::Port::B,
                        gpio::Pin::P13,
                        gpio::Pin::P14,
                        gpio::Pin::P15,
                        mode,
                    );
                }
            }

            // Enable the SPI peripheral.
            let dp = DevicePeripherals::steal();
            match self {
                Self::Spi1(_) => dp.RCC.apb2enr.modify(|_, w| w.spi1en().enabled()),
                Self::Spi2 => dp.RCC.apb1enr.modify(|_, w| w.spi2en().enabled()),
            }
            dp.RCC.apb2enr.modify(|_, w| w.afioen().enabled());

            // Control register configuration.
            (*self.ptr()).cr1.modify(|_, w| {
                // Baudrate.
                w.br().bits(self.baudrate_register(config.speed));
                // Clock polarity.
                w.cpol().bit((config.mode as u8 >> 1) > 0);
                w.cpha().bit((config.mode as u8 & 1) > 0);
                // 8 bit data frame.
                w.dff().eight_bit();
                // ByteOrder.
                match config.byteorder {
                    ByteOrder::MsbFirst => w.lsbfirst().msbfirst(),
                    ByteOrder::LsbFirst => w.lsbfirst().lsbfirst(),
                };
                // Software slave management.
                w.ssm().enabled();
                // Master/Slave configuration.
                match mode {
                    Master::Master => {
                        w.ssi().set_bit();
                        w.mstr().master()
                    }
                    Master::Slave => {
                        w.ssi().clear_bit();
                        w.mstr().slave()
                    }
                }
            });
        }
    }

    #[inline]
    pub fn enable(&self) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.spe().enabled());
        }
    }

    #[inline]
    pub fn disable(&self) {
        unsafe {
            (*self.ptr()).cr1.modify(|_, w| w.spe().disabled());
        }
    }

    #[inline]
    pub fn clk_speed(&self) -> u32 {
        let shift = unsafe { (*self.ptr()).cr1.read().br().bits() };
        let div = 1 << shift;
        clock::SPEED / div
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
    pub fn busy(&self) -> bool {
        unsafe { (*self.ptr()).sr.read().bsy().bit_is_set() }
    }

    unsafe fn baudrate_register(&self, speed: u32) -> u8 {
        let clk_speed = match self {
            Self::Spi1(_) => {
                // Clock obtained from APB2.
                clock::apb2_speed()
            },
            Self::Spi2 => {
                // Clock obtained from APB1.
                clock::apb1_speed()
            }
        };
        let mut reg = 0u8;
        while speed < clk_speed >> (reg + 1) {
            reg += 1;
        }
        reg.min(7)
    }
}

fn gpio_configuration(
    port: gpio::Port,
    sck: gpio::Pin,
    miso: gpio::Pin,
    mosi: gpio::Pin,
    mode: Master,
) {
    match mode {
        Master::Master => gpio_configuration_master(port, sck, miso, mosi),
        Master::Slave => gpio_configuration_slave(port, sck, miso, mosi),
    }
}

fn gpio_configuration_master(port: gpio::Port, sck: gpio::Pin, miso: gpio::Pin, mosi: gpio::Pin) {
    gpio::configure(
        gpio::Gpio(port, sck),
        gpio::Mode::AlternateFunctionOutputPushPull(gpio::Speed::Max10MHz),
    );
    gpio::configure(gpio::Gpio(port, miso), gpio::Mode::FloatingInput);
    gpio::configure(
        gpio::Gpio(port, mosi),
        gpio::Mode::AlternateFunctionOutputPushPull(gpio::Speed::Max10MHz),
    );
}

fn gpio_configuration_slave(port: gpio::Port, sck: gpio::Pin, miso: gpio::Pin, mosi: gpio::Pin) {
    gpio::configure(gpio::Gpio(port, sck), gpio::Mode::FloatingInput);
    gpio::configure(
        gpio::Gpio(port, miso),
        gpio::Mode::AlternateFunctionOutputPushPull(gpio::Speed::Max10MHz),
    );
    gpio::configure(gpio::Gpio(port, mosi), gpio::Mode::FloatingInput);
}
