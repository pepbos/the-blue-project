use crate::gpio;

/// Led controller.
///
/// Assumes led is on when pin is low.
#[derive(Debug)]
pub struct Led {
    pin: gpio::Gpio,
    on: bool,
}

impl Led {
    #[inline]
    pub fn new(pin: gpio::Gpio, mode: gpio::OutputMode) -> Self {
        let led = Self { pin, on: false };
        led.update();
        gpio::configure(pin, mode.into());
        led
    }

    #[inline]
    fn update(&self) {
        gpio::write(self.pin, !self.on);
    }

    #[inline]
    pub fn on(&mut self) {
        self.on = true;
        self.update();
    }

    #[inline]
    pub fn off(&mut self) {
        self.on = false;
        self.update();
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.on = !self.on;
        self.update();
    }

    #[inline]
    pub fn write(&mut self, on: bool) {
        if on {
            self.on();
        } else {
            self.off();
        }
    }
}
