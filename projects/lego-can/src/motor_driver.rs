//! Three FullBridge motor drivers.
//!
//! Board contains three motors, which are controlled by three full bridge drivers.
//! Each bridge consists of four N-MOSFETs, in an H-bridge configuration.
//! Voltage control is achieved using PWM on the high level MOSFETs.
//! Direction is controlled by correctly combining which MOSFETs to turn on when.

use bluepill::gpio;
use bluepill::timer;
use bluepill::timer::pwm;

/// Hardware layout:

/// Enable H-bridge:
const ENABLE_MOTOR: gpio::Gpio = gpio::PA4;

/// Left low FET gate:
const LOW_FET_LEFT: [gpio::Gpio; 3] = [gpio::PA1, gpio::PB14, gpio::PB12];

/// Right low FET gate:
const LOW_FET_RIGHT: [gpio::Gpio; 3] = [gpio::PA5, gpio::PB2, gpio::PB13];

/// Left PWM Channel:
const PWM_CH_LEFT: [timer::Channel; 3] =
    [timer::Channel::C1, timer::Channel::C3, timer::Channel::C1];

/// Right PWM Channel:
const PWM_CH_RIGHT: [timer::Channel; 3] =
    [timer::Channel::C2, timer::Channel::C4, timer::Channel::C2];

/// Left and right PWM timers:
const PWM_TIM: [timer::Timer; 3] = [timer::TIM3, timer::TIM3, timer::TIM1];

/// All used Timers:
const TIM: [timer::Timer; 2] = [timer::TIM1, timer::TIM3];

/// PWM timer auto reset register:
const ARR: u16 = i16::MAX as u16;

/// PWM timer prescaler:
const PSC: u16 = 0;

/// PWM configuration:
const PWM_MODE: pwm::Mode = pwm::Mode::Pwm1;
const PWM_POLARITY: pwm::Polarity = pwm::Polarity::ActiveHigh;

/// GPIO output mode:
const GPIO_MODE: gpio::OutputMode = gpio::OutputMode::PushPull(gpio::Speed::Max50MHz);

/// Motors PWM driver.
///
/// Consists of three H-bridge motor drivers.
///
/// PWM frequency is set to 2048Hz, with [i16::MAX] steps.
///
/// Must call [Motors::enable()] to enable.
pub struct Motors {
    /// Three [FullBridge] drivers, one for each motor.
    motors: [FullBridge; 3],
}

impl Motors {
    /// New motor driver.
    ///
    /// Initializes with the high level MOSFETs off, and low level MOSFETs on.
    pub fn new() -> Self {
        // Disable the GATE drivers.
        gpio::configure(
            ENABLE_MOTOR,
            gpio::Mode::OuputPushPull(gpio::Speed::Max2MHz),
        );
        gpio::write(ENABLE_MOTOR, false);

        // Enable the timers for PWM.
        let config = pwm::Config { psc: PSC, arr: ARR };
        let mut pwm = TIM.map(|tim| config.make(tim));

        let motors = [
            FullBridge::new(
                HalfBridge::new(PWM_TIM[0], PWM_CH_LEFT[0], LOW_FET_LEFT[0]),
                HalfBridge::new(PWM_TIM[0], PWM_CH_RIGHT[0], LOW_FET_RIGHT[0]),
            ),
            FullBridge::new(
                HalfBridge::new(PWM_TIM[1], PWM_CH_LEFT[1], LOW_FET_LEFT[1]),
                HalfBridge::new(PWM_TIM[1], PWM_CH_RIGHT[1], LOW_FET_RIGHT[1]),
            ),
            FullBridge::new(
                HalfBridge::new(PWM_TIM[2], PWM_CH_LEFT[2], LOW_FET_LEFT[2]),
                HalfBridge::new(PWM_TIM[2], PWM_CH_RIGHT[2], LOW_FET_RIGHT[2]),
            ),
        ];

        pwm.iter_mut().for_each(|pwm| pwm.enable());

        let mut out = Self { motors };
        out.off_ground();

        out
    }

    /// Set the enable pin of all gate drivers.
    pub fn enable(&mut self, enable: bool) {
        gpio::write(ENABLE_MOTOR, enable);
    }

    /// Turns off all FETs.
    #[allow(unused)]
    pub fn off(&mut self) {
        self.motors.iter_mut().for_each(|m| m.off());
    }

    /// Turns off, by connecting all legs to ground.
    pub fn off_ground(&mut self) {
        self.motors.iter_mut().for_each(|m| m.off_ground());
    }

    /// Set PWM from raw command.
    ///
    /// Buffer must contain atleast six bytes, representing three i16 in big endian format.
    ///
    /// Each i16 represents the pwm value of the corresponding motor.
    pub fn set_raw_pwm(&mut self, raw_pwm: &[u8]) {
        for i in 0..3 {
            let j = i * 2;
            let pwm = i16::from_be_bytes([raw_pwm[j], raw_pwm[j + 1]]);
            self.motors[i].pwm(pwm);
        }
    }

    /// Access the H-bridge drivers.
    #[allow(unused)]
    pub fn get_mut(&mut self) -> &mut [FullBridge; 3] {
        &mut self.motors
    }
}

/// Full H-bridge motor driver.
pub struct FullBridge {
    legs: [HalfBridge; 2],
}

impl FullBridge {
    fn new(left: HalfBridge, right: HalfBridge) -> Self {
        Self {
            legs: [left, right],
        }
    }

    /// Turns off all FETs.
    pub fn off(&mut self) {
        self.legs.iter_mut().for_each(|leg| leg.off());
    }

    /// Turns off, by connecting both legs to ground.
    pub fn off_ground(&mut self) {
        self.legs.iter_mut().for_each(|leg| leg.ground());
    }

    /// Set PWM value.
    ///
    /// - Positive PWM: left leg positive.
    /// - Negative PWM: right leg positive.
    /// - Zero PWM: off.
    ///
    /// when off, both legs are connected to ground.
    pub fn pwm(&mut self, pwm: i16) {
        let ccr = pwm.abs() as u16;
        if ccr == 0 {
            self.off_ground();
            return;
        }
        let direction = pwm > 0;
        let (pwm_leg, gnd_leg) = if direction { (0, 1) } else { (1, 0) };
        self.legs[gnd_leg].ground();
        self.legs[pwm_leg].pwm(ccr);
    }
}

/// One leg of the FullBridge.
///
/// Contains two MOSFETs:
/// - Low FET connected to gpio pin.
/// - High FET connected to pwm pin.
struct HalfBridge {
    gnd: gpio::Gpio,
    pwm: pwm::Channel,
}

impl HalfBridge {
    fn new(timer: timer::Timer, channel: timer::Channel, gnd: gpio::Gpio) -> Self {
        // Low FET configuration.
        gpio::write(gnd, false);
        gpio::configure(gnd, GPIO_MODE.into());

        // PWM configuration.
        let mut pwm = pwm::Channel::new(timer, channel);
        pwm.configure(PWM_MODE, PWM_POLARITY, GPIO_MODE.as_af());

        // High FET configuration.
        gpio::write(pwm.gpio(), false);
        gpio::configure(pwm.gpio(), GPIO_MODE.into());
        Self { gnd, pwm }
    }

    /// Turns off both FETs.
    fn off(&mut self) {
        gpio::write(self.gnd, false);
        gpio::configure(self.pwm.gpio(), GPIO_MODE.into());
    }

    /// Low FET on, high FET off.
    fn ground(&mut self) {
        gpio::configure(self.pwm.gpio(), GPIO_MODE.into());
        gpio::write(self.gnd, true);
    }

    /// Low FET off, high FET pwm.
    fn pwm(&mut self, pwm: u16) {
        gpio::write(self.gnd, false);
        gpio::configure(self.pwm.gpio(), GPIO_MODE.as_af().into());
        self.pwm.write_ccr(pwm);
    }
}
