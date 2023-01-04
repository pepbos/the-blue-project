mod initialization;
mod sample;
mod telemetry_source;

use initialization::initialization;
pub use telemetry_source::{TelemetrySource, DataFrame, FRAME_LEN};
pub use sample::Sample;

use bluepill::uart;

pub const POLL: u8 = 0x04;

pub struct LegoMotorPoller {
    bus: uart::Bus,
}

impl LegoMotorPoller {
    /// Initialize Lego motor communication.
    pub fn new(mut bus: uart::Bus) -> Option<Self> {
        let ok = initialization(&mut bus);
        bus.set_intterupts_mask(!ok);
        bus.rx_interrupt_enable(ok);
        Some(Self{bus}).filter(|_| ok)
    }

    /// Polls the Lego motor.
    ///
    /// Motor should be polled every 100ms.
    #[allow(unused)]
    pub fn poll(&mut self) {
        self.bus.wait_write_byte(POLL);
    }
}
