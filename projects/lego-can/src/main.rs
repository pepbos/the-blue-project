#![no_main]
#![no_std]

mod motor_driver;
mod telemetry;

extern crate panic_halt;

use bluepill::delay;
use bluepill::{clock, gpio, gpio::Mode, timer, uart, Led};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use motor_driver::Motors;
use stm32_usbd::UsbBus;
use stm32f1xx_hal::pac::interrupt;
use stm32f1xx_hal::pac::Interrupt;
use telemetry::{LegoMotorPoller, TelemetrySource};
use usb_device::{
    class_prelude::UsbBusAllocator,
    prelude::{UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

/// Lego UART bus.
const UART1: uart::Usart = uart::Usart::Usart1(uart::Port::B);
const UART2: uart::Usart = uart::Usart::Usart2;
const UART3: uart::Usart = uart::Usart::Usart3;

const UART: [uart::Usart; 3] = [UART2, UART3, UART1];

/// CAN address id:
const CAN_ID: [gpio::Gpio; 3] = [gpio::PC15, gpio::PC14, gpio::PA15];

/// LED - debug:
const LED: gpio::Gpio = gpio::PC13;

/// LEDs - motor status:
const LEDS: [gpio::Gpio; 3] = [gpio::PB5, gpio::PA10, gpio::PB15];

/// Lego motor 3V3 power enable:
const ENABLE_LEGO: [gpio::Gpio; 3] = [gpio::PA0, gpio::PB3, gpio::PB4];

/// Buffers containing the telemetry feedback from the motors.
static mut TELEMETRY_SOURCE: [TelemetrySource; 3] = [
    TelemetrySource::new(),
    TelemetrySource::new(),
    TelemetrySource::new(),
];

/// GPIO mode for the LED pins.
const LED_MODE: gpio::OutputMode = gpio::OutputMode::PushPull(gpio::Speed::Max2MHz);

/// Counter for timing the polling of the motors.
static POLL_COUNTER: AtomicU8 = AtomicU8::new(0u8);

/// Flags for which motors are enabled (connected).
static ENABLED: [AtomicBool; 3] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

/// Settings for blinking the LEDs.
const LED_TIMER_ARR: usize = 4096; // Timer auto reset register.
const LED_TIMER_CMP: usize = 2048; // Timer comparator value.
const LED_TIMER_CMP_BLIMP: usize = 128; // Shorter timer comparator value.

/// Motor turns off if connection is timed out.
const MOTOR_CMD_TIMEOUT: usize = 128;

#[entry]
fn main() -> ! {
    if cfg!(debug_assertions) {
        hprintln!("Hello LEGO!").unwrap();
    }

    // System setup:

    // Clock and gpio setup.
    unsafe {
        clock::init();
        gpio::enable();
        gpio::free_jtag();
    }

    // Wait for peripherals to enable.
    delay::millis(1);

    // CAN addres:
    for &id in CAN_ID.iter() {
        gpio::configure(id, Mode::InputPullUp);
    }
    if cfg!(debug_assertions) {
        hprintln!("CAN ID = NONE").unwrap();
    }

    // LEDs:
    let mut led = Led::new(LED, LED_MODE);
    led.on();
    let mut leds = LEDS.map(|led| Led::new(led, LED_MODE));

    // LEGO motor 3V3 pwr enable pins:
    for &en in ENABLE_LEGO.iter() {
        gpio::configure(en, gpio::Mode::OutputOpenDrain(gpio::Speed::Max2MHz));
        gpio::write(en, true);
    }

    // Motor FET driver.
    let mut motors = Motors::new();
    motors.enable(true);

    // Turn on interrupt on Timer3 for polling.
    unsafe { NVIC::unmask(Interrupt::TIM3) };
    timer::TIM3.update_interrupt_enable();

    // LEGO motor telemetry:
    if cfg!(debug_assertions) {
        hprintln!("Connecting to LEGO motors...").unwrap();
    }
    let config_uart = uart::Config {
        baudrate: 115200,
        tx_pin: gpio::OutputMode::PushPull(gpio::Speed::Max10MHz),
    };
    let mut lego_poller = [None, None, None];
    for i in 0..3 {
        leds[i].on();
        // Enable power to lego motor.
        gpio::write(ENABLE_LEGO[i], false);
        // Initialize motor.
        lego_poller[i] = LegoMotorPoller::new(config_uart.make(UART[i]));
        // Update status leds.
        let motor_ok = lego_poller[i].is_some();
        // turn on interrupt.
        ENABLED[i].store(motor_ok, Ordering::Relaxed);
        leds[i].write(motor_ok);
        // Control power to lego motor.
        gpio::write(ENABLE_LEGO[i], !motor_ok);
        if cfg!(debug_assertions) {
            if motor_ok {
                hprintln!("    Motor{} online", i).unwrap();
            } else {
                hprintln!("    Motor{} offline", i).unwrap();
            }
        }
    }

    if cfg!(debug_assertions) {
        hprintln!("Open USB connection.").unwrap();
    }

    // Pull the D+ pin down to send a RESET condition to the USB bus.
    gpio::configure(gpio::PA12, gpio::Mode::OuputPushPull(gpio::Speed::Max50MHz));
    gpio::write(gpio::PA12, false);
    delay::millis(10);
    gpio::configure(gpio::PA12, gpio::Mode::FloatingInput);

    // USB:
    let usb = bluepill::usb::Peripheral {};
    let usb_bus: UsbBusAllocator<UsbBus<bluepill::usb::Peripheral>> = UsbBus::new(usb);
    let mut usb_serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();
    let mut usb_rx_buf = [0u8; 64]; // Buffer for receiving pwm commands.
    let mut usb_tx_buf = [0u8; 24]; // Buffer for transmitting motor telemetry.

    let mut watchdog_motor_cmd: usize = MOTOR_CMD_TIMEOUT;
    let mut timer_led: usize = 0;

    // Entering main loop.
    loop {
        // Timer1 runs at 2kHz for the PWM signals.
        if timer::TIM1.read_update_interrupt_flag() {
            timer::TIM1.clear_update_interrupt_flag();
            watchdog_motor_cmd = watchdog_motor_cmd.saturating_add(1);
            timer_led = (timer_led + 1) % LED_TIMER_ARR;
        }

        // Must receive motor commands every 100ms.
        let motor_timed_out = watchdog_motor_cmd >= MOTOR_CMD_TIMEOUT;
        if motor_timed_out {
            motors.off_ground();
        }

        // Read LEGO telemetry sample: Leds flash when receiving samples.
        for (i, s) in unsafe { TELEMETRY_SOURCE.iter().enumerate() } {
            if let Ok(Some(s)) = s.try_read_sample() {
                usb_tx_buf[i * 8] = 1;
                s.write_be_bytes(&mut usb_tx_buf[i * 8 + 1..]);
                leds[i].toggle();
            }
        }

        // Status LED.
        if !motor_timed_out {
            // Solid ON if connected.
            led.on();
        } else {
            // Blink if no motor commands received.
            led.write(timer_led <= LED_TIMER_CMP);
        }

        // Status LED: Short blimps if connected.
        if timer_led <= LED_TIMER_CMP_BLIMP {
            for i in (0..3).filter(|&i| lego_poller[i].is_some()) {
                leds[i].on();
            }
        }

        // Receive PWM commands over USB.
        if !usb_dev.poll(&mut [&mut usb_serial]) {
            continue;
        }

        // Receive motor commands over USB.
        if Some(6) != usb_serial.read(&mut usb_rx_buf).ok() {
            continue;
        }

        motors.set_raw_pwm(&usb_rx_buf[0..6]);
        watchdog_motor_cmd = 0;

        // Transmit LEGO telemetry over USB.
        let mut write_offset = 0;
        while write_offset < usb_tx_buf.len() {
            match usb_serial.write(&usb_tx_buf[write_offset..]) {
                Ok(len) if len > 0 => {
                    write_offset += len;
                }
                _ => (),
            }
        }
        let _ = usb_serial.flush();
        for i in 0..3 {
            usb_tx_buf[i * 8] = 0;
        }
    }
}

/// USART1 interrupt: third motor.
///
/// Triggers when receiving telemetry feedback.
#[interrupt]
fn USART1() {
    // Transmit interrupt is not enabled.

    // Receive interrupt is triggered if the uart-buffer contains a byte.
    if UART1.rx_buffer_not_empty() {
        // Reading the byte clears the interrupt.
        let byte = UART1.read_data_reg();
        // Push the byte to the Lego Telemetry Buffer.
        let _ = unsafe { TELEMETRY_SOURCE[2].write_byte(byte) };
    }
}

/// USART2 interrupt: first motor.
#[interrupt]
fn USART2() {
    if UART2.rx_buffer_not_empty() {
        let byte = UART2.read_data_reg();
        let _ = unsafe { TELEMETRY_SOURCE[0].write_byte(byte) };
    }
}

/// USART3 interrupt: second motor.
#[interrupt]
fn USART3() {
    if UART3.rx_buffer_not_empty() {
        let byte = UART3.read_data_reg();
        let _ = unsafe { TELEMETRY_SOURCE[1].write_byte(byte) };
    }
}

/// TIMER3 interrupt: used to poll the motors at ~10Hz.
#[interrupt]
fn TIM3() {
    timer::TIM3.clear_update_interrupt_flag();
    if POLL_COUNTER.fetch_add(1, Ordering::Relaxed) == 180 {
        POLL_COUNTER.store(0, Ordering::Relaxed);
        for i in 0..3 {
            if ENABLED[i].load(Ordering::Relaxed) {
                UART[i].write_data_reg(telemetry::POLL);
            }
        }
    }
}
