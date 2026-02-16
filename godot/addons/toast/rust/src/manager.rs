use crate::config::ToastConfig;
use crate::toast::Toast;
use crate::types::{QueueMode, ToastPosition, ToastType};
use crate::ToastDuration;
use godot::classes::control::{LayoutPreset, MouseFilter};
use godot::classes::Control;
use godot::classes::VBoxContainer;
use godot::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct SingletonWrapper(Gd<ToastManager>);

unsafe impl Sync for SingletonWrapper {}
unsafe impl Send for SingletonWrapper {}

static TOAST_MANAGER: OnceLock<SingletonWrapper> = OnceLock::new();

const TOAST_SPACING: f32 = 10.0;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct ToastManager {
    #[base]
    base: Base<Node>,

    position_containers: HashMap<ToastPosition, Gd<VBoxContainer>>,
    toasts: Vec<Gd<Toast>>,
    queue: Vec<ToastConfig>,
    max_toasts: usize,
    default_position: ToastPosition,
    queue_mode: QueueMode,
    pending_toasts: Vec<ToastConfig>,
}

#[godot_api]
impl INode for ToastManager {
    fn ready(&mut self) {
        self.create_container();
        self.register_singleton();
    }

    fn process(&mut self, _delta: f64) {
        self.cleanup_dismissed_toasts();
        self.process_queue();
        self.process_pending_toasts();
    }
}

#[godot_api]
impl ToastManager {
    pub fn setup_toast_manager() {
        let engine = godot::classes::Engine::singleton();
        let main_loop = engine.get_main_loop();

        if let Some(main_loop_obj) = main_loop {
            if let Ok(scene_tree) = main_loop_obj.try_cast::<godot::classes::SceneTree>() {
                if let Some(mut root) = scene_tree.get_root() {
                    if get_toast_manager().is_none() {
                        let mut toast_manager = ToastManager::new_alloc();
                        toast_manager.set_name("ToastManager");
                        root.call_deferred("add_child", &[toast_manager.to_variant()]);
                    }
                }
            }
        }
    }

    #[func]
    pub fn show(&mut self, text: String) {
        let config = ToastConfig::new(text);
        self.show_with_config(config);
    }

    #[func]
    pub fn show_with_type(&mut self, text: String, toast_type: i32) {
        let mut config = ToastConfig::new(text);
        config.toast_type = match toast_type {
            0 => ToastType::Info,
            1 => ToastType::Success,
            2 => ToastType::Warning,
            3 => ToastType::Error,
            _ => ToastType::Custom,
        };
        self.show_with_config(config);
    }

    #[func]
    pub fn show_with_position(&mut self, text: String, position: i32) {
        let mut config = ToastConfig::new(text);
        config.position = Self::int_to_position(position);
        self.show_with_config(config);
    }

    #[func]
    pub fn show_with_duration(&mut self, text: String, duration: f32) {
        let mut config = ToastConfig::new(text);
        config.duration = ToastDuration::from_seconds(duration);
        self.show_with_config(config);
    }

    pub fn show_with_config(&mut self, config: ToastConfig) {
        if !self.base().is_inside_tree() || self.position_containers.is_empty() {
            self.pending_toasts.push(config);
            return;
        }

        if self.toasts.len() >= self.max_toasts {
            match self.queue_mode {
                QueueMode::Unlimited => {}
                QueueMode::FIFO => {
                    self.queue.push(config);
                    return;
                }
            }
        }

        self.create_toast(config);
    }

    #[func]
    fn process_pending_toasts(&mut self) {
        let configs: Vec<ToastConfig> = self.pending_toasts.drain(..).collect();

        for config in configs {
            if self.toasts.len() >= self.max_toasts {
                match self.queue_mode {
                    QueueMode::Unlimited => {}
                    QueueMode::FIFO => {
                        self.queue.push(config);
                        continue;
                    }
                }
            }

            self.create_toast(config);
        }
    }

    fn create_container(&mut self) {
        let positions = [
            ToastPosition::TopLeft,
            ToastPosition::TopCenter,
            ToastPosition::TopRight,
            ToastPosition::MiddleLeft,
            ToastPosition::MiddleCenter,
            ToastPosition::MiddleRight,
            ToastPosition::BottomLeft,
            ToastPosition::BottomCenter,
            ToastPosition::BottomRight,
        ];

        for position in positions {
            if self.position_containers.contains_key(&position) {
                continue;
            }

            let mut container = VBoxContainer::new_alloc();
            container.set_name(&format!("ToastContainer_{:?}", position));
            container.set_mouse_filter(MouseFilter::IGNORE);
            container.set_z_index(1000);
            container.add_theme_constant_override("separation", TOAST_SPACING as i32);

            let (anchors, offsets) = Self::calculate_container_layout(position, 300.0, 20.0);

            container.set_anchor(Side::LEFT, anchors.0);
            container.set_anchor(Side::TOP, anchors.1);
            container.set_anchor(Side::RIGHT, anchors.2);
            container.set_anchor(Side::BOTTOM, anchors.3);
            container.set_offset(Side::LEFT, offsets.0);
            container.set_offset(Side::TOP, offsets.1);
            container.set_offset(Side::RIGHT, offsets.2);
            container.set_offset(Side::BOTTOM, offsets.3);

            let container_clone = container.clone();
            self.base_mut().add_child(&container_clone.upcast::<Node>());
            self.position_containers.insert(position, container);
        }
    }

    fn register_singleton(&mut self) {
        if set_toast_manager(self.to_gd()).is_err() {
            godot_error!("ToastManager: failed to set singleton, already registered");
        }
    }

    #[func]
    pub fn dismiss_all(&mut self) {
        for toast in self.toasts.iter_mut() {
            toast.bind_mut().dismiss();
        }
    }

    #[func]
    pub fn set_max_toasts(&mut self, max: i32) {
        self.max_toasts = max as usize;
    }

    #[func]
    pub fn get_max_toasts(&self) -> i32 {
        self.max_toasts as i32
    }

    #[func]
    pub fn set_default_position(&mut self, position: i32) {
        self.default_position = Self::int_to_position(position);
    }

    #[func]
    pub fn get_default_position(&self) -> i32 {
        Self::position_to_int(self.default_position)
    }

    #[func]
    pub fn set_queue_mode(&mut self, mode: i32) {
        self.queue_mode = match mode {
            0 => QueueMode::Unlimited,
            1 => QueueMode::FIFO,
            _ => QueueMode::Unlimited,
        };
    }

    #[func]
    pub fn get_queue_mode(&self) -> i32 {
        match self.queue_mode {
            QueueMode::Unlimited => 0,
            QueueMode::FIFO => 1,
        }
    }

    fn create_toast(&mut self, config: ToastConfig) {
        let mut toast = Toast::from_scene().unwrap_or_else(|| {
            godot_warn!("Failed to load toast scene, using fallback");
            Toast::new_alloc()
        });
        toast.bind_mut().set_config(config.clone());

        self.position_toast(toast.clone(), config.position);

        toast.bind_mut().show();
        self.toasts.push(toast);
    }

    fn position_toast(&mut self, toast: Gd<Toast>, position: ToastPosition) {
        if let Some(container) = self.position_containers.get_mut(&position) {
            let toast_upcasted = toast.clone().upcast::<Node>();
            container.add_child(&toast_upcasted);

            godot_print!(
                "ToastManager: Added toast to container at position {:?}, child count: {}",
                position,
                container.get_child_count()
            );

            if let Ok(mut toast_control) = toast_upcasted.try_cast::<Control>() {
                toast_control.set_anchors_and_offsets_preset(LayoutPreset::TOP_LEFT);
                toast_control.set_position(Vector2::ZERO);
                godot_print!("ToastManager: Reset toast anchors and position");
            }

            let is_top_position = matches!(
                position,
                ToastPosition::TopLeft | ToastPosition::TopCenter | ToastPosition::TopRight
            );

            if is_top_position {
                let child_count = container.get_child_count();
                if child_count > 0 {
                    container.move_child(&toast.upcast::<Node>(), 0);
                    godot_print!("ToastManager: Moved toast to top position");
                }
            }
        } else {
            godot_error!(
                "ToastManager: no container found for position {:?}",
                position
            );
        }
    }

    fn cleanup_dismissed_toasts(&mut self) {
        self.toasts.retain(|toast| toast.is_instance_valid());
    }

    fn process_queue(&mut self) {
        if self.toasts.len() < self.max_toasts && !self.queue.is_empty() {
            let config = self.queue.remove(0);
            self.create_toast(config);
        }
    }

    fn int_to_position(position: i32) -> ToastPosition {
        match position {
            0 => ToastPosition::TopLeft,
            1 => ToastPosition::TopCenter,
            2 => ToastPosition::TopRight,
            3 => ToastPosition::MiddleLeft,
            4 => ToastPosition::MiddleCenter,
            5 => ToastPosition::MiddleRight,
            6 => ToastPosition::BottomLeft,
            7 => ToastPosition::BottomCenter,
            8 => ToastPosition::BottomRight,
            _ => ToastPosition::TopCenter,
        }
    }

    fn position_to_int(position: ToastPosition) -> i32 {
        match position {
            ToastPosition::TopLeft => 0,
            ToastPosition::TopCenter => 1,
            ToastPosition::TopRight => 2,
            ToastPosition::MiddleLeft => 3,
            ToastPosition::MiddleCenter => 4,
            ToastPosition::MiddleRight => 5,
            ToastPosition::BottomLeft => 6,
            ToastPosition::BottomCenter => 7,
            ToastPosition::BottomRight => 8,
        }
    }

    #[allow(clippy::type_complexity)]
    fn calculate_container_layout(
        position: ToastPosition,
        toast_width: f32,
        margin: f32,
    ) -> ((f32, f32, f32, f32), (f32, f32, f32, f32)) {
        let half_width = toast_width / 2.0;
        let full_width = toast_width + margin;

        let anchors = match position {
            ToastPosition::TopLeft => (0.0, 0.0, 0.0, 0.0),
            ToastPosition::TopCenter => (0.5, 0.0, 0.5, 0.0),
            ToastPosition::TopRight => (1.0, 0.0, 1.0, 0.0),
            ToastPosition::MiddleLeft => (0.0, 0.5, 0.0, 0.5),
            ToastPosition::MiddleCenter => (0.5, 0.5, 0.5, 0.5),
            ToastPosition::MiddleRight => (1.0, 0.5, 1.0, 0.5),
            ToastPosition::BottomLeft => (0.0, 1.0, 0.0, 1.0),
            ToastPosition::BottomCenter => (0.5, 1.0, 0.5, 1.0),
            ToastPosition::BottomRight => (1.0, 1.0, 1.0, 1.0),
        };

        let offsets = match position {
            ToastPosition::TopLeft => (margin, margin, -margin, -margin),
            ToastPosition::TopCenter => (-half_width, margin, half_width, -margin),
            ToastPosition::TopRight => (-full_width, margin, -margin, -margin),
            ToastPosition::MiddleLeft => (margin, -half_width, margin + toast_width, half_width),
            ToastPosition::MiddleCenter => (-half_width, -half_width, half_width, half_width),
            ToastPosition::MiddleRight => (-full_width, -half_width, -margin, half_width),
            ToastPosition::BottomLeft => (margin, -half_width, margin + toast_width, -margin),
            ToastPosition::BottomCenter => (-half_width, -half_width, half_width, -margin),
            ToastPosition::BottomRight => (-full_width, -half_width, -margin, -margin),
        };

        (anchors, offsets)
    }
}

pub fn get_toast_manager() -> Option<Gd<ToastManager>> {
    TOAST_MANAGER.get().map(|wrapper| wrapper.0.clone())
}

pub fn set_toast_manager(manager: Gd<ToastManager>) -> Result<(), SingletonWrapper> {
    TOAST_MANAGER.set(SingletonWrapper(manager))
}
