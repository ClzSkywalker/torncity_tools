#[derive(Debug, Clone, Copy, PartialEq, Eq,Default)]
pub enum ToastType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    #[default]
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ToastDuration {
    Short,
    #[default]
    Medium,
    Long,
    Custom(f32),
}

impl ToastDuration {
    pub fn from_seconds(seconds: f32) -> Self {
        Self::Custom(seconds)
    }

    pub fn as_seconds(&self) -> f32 {
        match self {
            Self::Short => 1.5,
            Self::Medium => 3.0,
            Self::Long => 5.0,
            Self::Custom(seconds) => *seconds,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub enable: bool,
    pub duration: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enable: true,
            duration: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueueMode {
    #[default]
    Unlimited,
    FIFO,
}

impl ToastType {
    pub fn default_background_color(&self) -> godot::prelude::Color {
        match self {
            Self::Info => godot::prelude::Color::from_rgba(0.2, 0.5, 1.0, 0.9),
            Self::Success => godot::prelude::Color::from_rgba(0.2, 0.8, 0.3, 0.9),
            Self::Warning => godot::prelude::Color::from_rgba(1.0, 0.7, 0.2, 0.9),
            Self::Error => godot::prelude::Color::from_rgba(0.9, 0.3, 0.3, 0.9),
            Self::Custom => godot::prelude::Color::from_rgba(0.2, 0.2, 0.2, 0.9),
        }
    }
}
