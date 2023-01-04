use super::POLL;
use bluepill::{delay, gpio, uart};

/// First sequence of bytes measured during connecting with Control Hub.
const HELLO: [u8; 6] = [0x52, 0x00, 0xC2, 0x01, 0x00, 0x6E];
/// Second sequence of bytes measured during connecting with Control Hub.
const CONFIRMED: [u8; 10] = [0x5C, 0x23, 0x00, 0x10, 0x20, 0x30, 0x00, 0x00, 0x00, 0x80];

/// Replays the initialization phase.
///
/// Lego is not transparant in the protocol used to communicate with the motors. Therefore the
/// initialization phase during connection with the Lego Control+ Hub was recorded using logic
/// analyzer. This method simply replays that recorded communication.
///
/// Returns whether the initialization was succesful.
#[must_use]
pub fn initialization(bus: &mut uart::Bus) -> bool {
    // Disable uart bus.
    bus.rx_enable(false);
    bus.tx_enable(false);

    // Control Hub turns TX pin on and off 21 times.
    let tx_pin = bus.get_tx_pin();
    gpio::configure(tx_pin, gpio::OutputMode::PushPull(gpio::Speed::Max10MHz).into());
    for _ in 0..21 {
        gpio::write(tx_pin, true);
        delay::millis(19);
        gpio::write(tx_pin, false);
        delay::millis(2);
    }

    // Say hello.
    bus.tx_enable(true);
    bus.write_bytes(&HELLO);

    // Ignore many messages... (unknown protocol).
    delay::millis(220);
    bus.rx_enable(true);

    // Wait until the ACK=POLL from motors.
    let _ = bus.read_byte(); // Flush bus.
    let mut ack = false;
    for _ in 0..100_000 { // Wait for the POLL byte.
        if let Some(POLL) = bus.read_byte() {
            // ACK message received: finalize communication.
            delay::micros(800);
            bus.wait_write_byte(POLL);
            delay::millis(10);
            bus.write_bytes(&CONFIRMED);
            ack = true;
            break;
        }
        delay::micros(1);
    }
    ack
}
