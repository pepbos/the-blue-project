/// GPIO pin mode.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    OuputPushPull(Speed),
    OutputOpenDrain(Speed),
    AlternateFunctionOutputPushPull(Speed),
    AlternateFunctionOutputOpenDrain(Speed),
    AnalogInput,
    FloatingInput,
    InputPullDown,
    InputPullUp,
}

/// GPIO switching speed.
///
/// Lower speeds can reduce ringing.
#[derive(Clone, Copy, Debug)]
pub enum Speed {
    Max10MHz = 1,
    Max2MHz = 2,
    Max50MHz = 3,
}

/// GPIO alternate function mode.
///
/// Subset of the GPIO [modes][Mode].
#[derive(Clone, Copy, Debug)]
pub enum AlternateFunctionOutputMode {
    PushPull(Speed),
    OpenDrain(Speed),
}


/// GPIO input mode.
///
/// Subset of the GPIO [modes][Mode].
#[derive(Clone, Copy, Debug)]
pub enum InputMode {
    FloatingInput,
    InputPullDown,
    InputPullUp,
}


/// GPIO output mode.
///
/// Subset of the GPIO [modes][Mode].
#[derive(Clone, Copy, Debug)]
pub enum OutputMode {
    PushPull(Speed),
    OpenDrain(Speed),
}

impl Into<Mode> for AlternateFunctionOutputMode {
    #[inline]
    fn into(self) -> Mode {
        match self {
            Self::PushPull(speed) => Mode::AlternateFunctionOutputPushPull(speed),
            Self::OpenDrain(speed) => Mode::AlternateFunctionOutputOpenDrain(speed),
        }
    }
}

impl Into<Mode> for InputMode {
    #[inline]
    fn into(self) -> Mode {
        match self {
            Self::FloatingInput => Mode::FloatingInput,
            Self::InputPullDown => Mode::InputPullDown,
            Self::InputPullUp => Mode::InputPullUp,
        }
    }
}

impl Into<Mode> for OutputMode {
    #[inline]
    fn into(self) -> Mode {
        match self {
            Self::PushPull(speed) => Mode::OuputPushPull(speed),
            Self::OpenDrain(speed) => Mode::OutputOpenDrain(speed),
        }
    }
}

impl OutputMode {
    /// Convert to equivalent [Alternate Function mode][AlternateFunctionOutputMode].
    #[inline]
    pub fn as_af(self) -> AlternateFunctionOutputMode {
        match self {
            Self::PushPull(speed) => AlternateFunctionOutputMode::PushPull(speed),
            Self::OpenDrain(speed) => AlternateFunctionOutputMode::OpenDrain(speed),
        }
    }
}
