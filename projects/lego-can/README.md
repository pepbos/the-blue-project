# Regain Control of Your Lego!

This project revolves around a custom board for controlling Lego motors.

Lego motors nowadays have encoders built in, and communicate some telemetry over UART: Speed, angle, and accumulated angle.
Unfortunately, using these motors rquires programming in the software environment provided by Lego, which is not a satisfying experience.
Furthermore, the protocol designed by Lego is closed source, preventing you from using them easily. Finally, Lego designed custom connectors, further discouraging you from building your own projects.

This project gives you back control over your Lego motors!

Up to three motors can be connected to the board. The board has a USB connector, for simple access using your PC for example.
A CAN bus is included, such that up to 8 boards can be connected to a single bus.
Simply send three PWM commands, and the board will send back the latest telemetry samples.

Naturally, the conversion of the encoder value to UART (by the Lego motor), and then converting it to USB/CAN is not very efficient. Opening the motors, and replacing the controller inside would have been more efficient. But the purpose of this board is to prevent having to break anyhting, leaving your precious Lego motors intact, but for you to use as you wish.

## Hardware

The board contains:
- `STM32F103` microcontroller handling communication and motor drivers,
- Three full-bridge motor drivers, for controlling the motor spin direction and voltage.
- USB and CAN bus for communication,
- Four status LEDs: one for each detected motor, and one for the USB/CAN status.
- Power connector: `9V-12V`.

![Lego-can](https://user-images.githubusercontent.com/46680867/210623396-f0d733bd-051d-478f-9b75-4caff7ba5fc6.png)

## Interface

Controlling the motors is meant to be straightforward:
- The three status leds show which motors are detected,
- Start sending pwm commands over either USB or CAN.
- The board responds to each received package by sending the telemetry sample.

### Connecting over USB

- Control the motors by transmitting six bytes, representing three `i16` pwm values.
    - Sending zero will turn off that motor (motor leads are connected to each other).
    - The sign of the pwm value determines the rotation direction.
- The board responds to each received USB package by sending `24` bytes back, for each motor it sends:
    - `STATUS:   u8 `: `1` if the sample is new, `0` otherwise,
    - `SPEED:    i8 `: encoder speed,
    - `ANGLE:    i32`: accumulated encoder angle,
    - `POSITION: i16`: absolute encoder angle,
- If no commands are received for `~100ms` the motors are turned off, and the status led starts flashing.

### Connecting over CAN

The CAN interface hardware is present, but the software is for future work.

## Software Description

The `STM32F103` lies at the heart of the board. It performs two tasks:

#### Relay of Enocder Data

The Lego motors must be polled every `100ms` to keep communication going. Any byte received from the motor over UART triggers an interrupt, and the byte is buffered. When a dataframe is completed (10 bytes), the telemetry data is decoded, the fresh sample is ready for transmitting over USB or CAN, and the buffer is reset. After receiving any package over USB the board responds by sending back the latest telemetry samples. The `STATUS` byte can be checked to see which samples are new. The motors send telemetry at a maximum of `250Hz`, so it is advised to poll the board at `1kHz` to make sure no samples are missed.

#### Motor Driver

The board contains three full-bridge motor drivers. Using pwm the motor voltage can be controlled in `i16::MAX` steps. The spin direction is controlled by switching the correct MOSFETs. A MOSFET driver chip is used to prevent short circuits when switching direction. For the protocol see the Interface section.

## Detailed Hardware Layout

 The `STM32F103` is used as the brains of the board. The pins are connected as follows:

- Three bit CAN-address solder jumpers:
    - `PC15`    -> `ADD0`
    - `PC14`    -> `ADD1`
    - `PA15`    -> `ADD2`
- Status leds:
    - `PC13`    -> `LED1`
    - `PA10`    -> `LED2`
    - `PB15`    -> `LED3`
    - `PB5`     -> `LED4`
- Pins for controlling power to the microcontroller onboard each Lego motor:
    - `PA0`     -> `EN0`
    - `PB3`     -> `EN1`
    - `PB4`     -> `EN2`
- Pin for enabling/disabling all mosfets:
    - `PA4`     -> `EN_FET`
- UART bus for each motor:
    - `PA2:3`   -> `UART2` -> Motor0
    - `PB10:11` -> `UART3` -> Motor1
    - `PB6:7`   -> `UART1` -> Motor2
- Low voltage MOSFET gate driver:
    - `PA1,5`   -> Motor0
    - `PB14,2`  -> Motor1
    - `PB12:13` -> Motor2
- High voltage level MOSFET gate driver (PWM):
    - `PA8:9`   -> `TIM1C1:2`
    - `PA6:7`   -> `TIM3C1:2`
    - `PB0:1`   -> `TIM3C3:4`
- CAN bus:
    - `PB8:9`   -> `CAN`
- USB:
    - `PA11:12` -> `USB`
