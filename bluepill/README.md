# Bluepill Minimal HAL

Minimal Hardware Abstraction Layer (HAL) for the `STM32F103` microcontroller, a.k.a the Bluepill, written in Rust.

Allows for programming of the Bluepill without having to work on the level of registers.
It is minimal in the sense that it does not follow strict guidelines on HAL crate design, e.g.
it is possible to attempt accessing a gpio pin, without first activating the system clock.
Although less safe, this does give a more power-user interaction with the hardware.

## Supported Peripherals

Support for Peripherals is added as they are required in personal projects.
Currently supported:

- System clock,
- GPIO,
- UART,
- Timers: including PWM and encoder reading,
- SPI,
- I2C.
