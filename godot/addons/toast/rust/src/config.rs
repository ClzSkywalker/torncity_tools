use godot::prelude::*;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct ToastConfig {
    pub toast_type: ToastType,
    pub text: String,
    pub duration: ToastDuration,
    pub position: ToastPosition,
    pub width: f32,
    pub background_color: Option<Color>,
    pub text_color: Color,
    pub font_size: i32,
    pub corner_radius: f32,
    pub auto_dismiss: bool,
    pub show_close_button: bool,
    pub animation: AnimationConfig,
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            toast_type: ToastType::default(),
            text: String::new(),
            duration: ToastDuration::default(),
            position: ToastPosition::default(),
            width: 300.0,
            background_color: None,
            text_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            font_size: 16,
            corner_radius: 8.0,
            auto_dismiss: true,
            show_close_button: true,
            animation: AnimationConfig::default(),
        }
    }
}

impl ToastConfig {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    pub fn with_type(mut self, toast_type: ToastType) -> Self {
        self.toast_type = toast_type;
        self
    }

    pub fn with_duration(mut self, duration: ToastDuration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_font_size(mut self, size: i32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_auto_dismiss(mut self, auto: bool) -> Self {
        self.auto_dismiss = auto;
        self
    }

    pub fn with_show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    pub fn with_animation(mut self, animation: AnimationConfig) -> Self {
        self.animation = animation;
        self
    }

    pub fn get_background_color(&self) -> Color {
        self.background_color.unwrap_or_else(|| self.toast_type.default_background_color())
    }
}
