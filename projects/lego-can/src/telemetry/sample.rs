use super::{FRAME_LEN, DataFrame};

/// Lego telemetry sample.
///
/// Lego motor transmits this information as feedback over UART.
#[derive(Copy, Clone, Debug, Default)]
pub struct Sample {
    /// Rotation speed [%] = [-125...125]
    pub speed: i8,
    /// Accumulated angle [deg]
    pub angle: i32,
    /// Absolute angle position [deg]
    pub position: i16,
}

impl Sample {
    /// Constructs [Sample] from [DataFrame].
    ///
    /// Returns [Err] if the crc fails.
    pub fn from_dataframe(bytes: &DataFrame) -> Result<Self, ()> {
        if checksum_checker(bytes) {
            Ok(Self::from_be_bytes(&bytes[1..]))
        } else {
            Err(())
        }
    }

    /// Construct self from raw bytes.
    ///
    /// Buffer must be atleast 7 bytes long.
    ///
    /// Corresponds to bytes 1:8 from the [DataFrame].
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        Self {
            speed: i8::from_be_bytes([bytes[0]]),
            angle: i32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
            position: i16::from_be_bytes([bytes[5], bytes[6]]),
        }
    }

    /// Write self to buffer as raw bytes.
    ///
    /// Buffer must be atleast 7 bytes long.
    ///
    /// Corresponds to bytes 1:8 from the [DataFrame].
    pub fn write_be_bytes(&self, bytes: &mut [u8]) {
        let speed = self.speed.to_be_bytes();
        let angle = self.angle.to_be_bytes();
        let position = self.position.to_be_bytes();
        bytes[0] = speed[0];
        bytes[1..5].copy_from_slice(&angle);
        bytes[5..7].copy_from_slice(&position);
    }
}

/// Checksum as designed by Lego.
///
/// Checksum8 = NOT(XOR( of previously transmitted bytes)).
fn checksum_checker(buffer: &DataFrame) -> bool {
    let mut xor = buffer[0];
    for i in 1..FRAME_LEN - 1 {
        xor = xor ^ buffer[i];
    }
    buffer[FRAME_LEN - 1] == !xor
}
