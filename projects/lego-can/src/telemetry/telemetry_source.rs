use core::sync::atomic::{AtomicU32, Ordering};
use super::Sample;

// Length of DataFrame is 10 bytes.
pub const FRAME_LEN: usize = 10;

// Each frame starts with this byte.
pub const START: u8 = 0xD8;

/// Dataframe received from LEGO motor.
///
/// - Byte[0]:    Start byte = 0xD8
/// - Bytes[1]:   Rotation speed [%] = [-125...125] : i8
/// - Bytes[2:6]: Accumulated angle [deg] : i32
/// - Bytes[6:8]: Absolute angle position [deg] : i16
/// - Byte[8]:    Filler = 0
/// - Byte[9]:    Checksum8 = NOT(XOR( of previously transmitted bytes)).
///
/// Bytes are in little endian order.
pub type DataFrame = [u8; FRAME_LEN];

/// Status definitions for async friendly reading and writing to the buffer.
const IDLE: u32 = 0; // Buffer is ready for writing.
const WRITING: u32 = 1; // Buffer is busy being written.
const DONEWRITING: u32 = 2; // Buffer is ready for reading.
const READING: u32 = 3; // Buffer is busy being read.

/// Async friendly buffer for telemetry feedback.
///
/// Intended use:
///
/// Define a global TelemetrySource variable.
/// Use the UART-RX interrupt trigger to push bytes using `TelemetrySource::write_byte(...)`.
/// In the main loop, use `TelemetrySource::try_read_sample()` to obtain the latest telemetry sample.
///
/// Make sure that `try_read_sample` is polled faster than the max telemetry feedback rate = 250Hz,
/// or risc missing samples.
pub struct TelemetrySource {
    /// Used to sync reading and writing of the buffer.
    ///
    /// 0 = IDLE
    /// 1 = WRITING
    /// 2 = DONEWRITING
    /// 3 = READING
    status: AtomicU32,
    /// Buffer for holding the data frame.
    data: DataFrame,
    /// Index of byte currently being written.
    write_index: u32,
}

impl TelemetrySource {
    pub const fn new() -> Self {
        Self {
            status: AtomicU32::new(0),
            data: [0u8; FRAME_LEN],
            write_index: 0,
        }
    }

    /// Push byte to the buffer.
    ///
    /// This method will lock the buffer, preventing reading the buffer.
    /// If a data frame is completed, the lock is released.
    ///
    /// This method returns an error if:
    /// - the previous sample was not read when starting a new sample,
    /// - the first byte does not equal `START`,
    pub fn write_byte(&mut self, byte: u8) -> Result<(), ()> {
        if let Err(status) =
            self.status
                .compare_exchange(IDLE, WRITING, Ordering::Acquire, Ordering::Relaxed)
        {
            if status != WRITING {
                // Status must have been either DONEWRITING or READING.
                // In this case the reader is too slow in readng the data.
                // Reset the counter such that the START byte is picked up.
                self.write_index = 0;
                return Err(());
            }
        }

        // Store the byte in the buffer.
        let i = self.write_index as usize;
        self.data[i] = byte;

        // Check start byte.
        let start_failed = (i == 0) && (byte != START);
        if start_failed {
            self.write_index = 0;
            self.status.store(IDLE, Ordering::Relaxed);
            return Err(());
        }

        // Update the byte index.
        self.write_index = ((i + 1) % FRAME_LEN) as u32;

        if self.write_index == 0 {
            self.status.store(DONEWRITING, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Read the telemetry sample, if available.
    ///
    /// Returns None if the buffer is locked.
    /// Returns Error if the buffer was already locked for reading.
    ///
    /// This method locks the buffer while reading, and releases the lock when complete.
    pub fn try_read_sample(&self) -> Result<Option<Sample>, ()> {
        // Try to lock the buffer for reading.
        match self.status.compare_exchange(
            DONEWRITING,
            READING,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                // Succes! Read the sample, and reset the status.
                let sample = Sample::from_dataframe(&self.data)?;
                self.status.store(IDLE, Ordering::Relaxed);
                Ok(Some(sample))
            }
            Err(READING) => {
                // Something went very very wrong.
                self.status.store(IDLE, Ordering::Relaxed);
                Err(())
            }
            _ => {
                // Buffer is busy or empty. Lets try later.
                Ok(None)
            }
        }
    }
}
