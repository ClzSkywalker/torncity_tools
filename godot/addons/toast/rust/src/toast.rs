use crate::config::ToastConfig;
use godot::classes::{Button, IPanelContainer, Label, PanelContainer, PackedScene, ResourceLoader, StyleBoxFlat};
use godot::prelude::*;

const TOAST_SCENE_PATH: &str = "res://addons/toast/scenes/toast.tscn";

#[derive(GodotClass)]
#[class(init, base=PanelContainer)]
pub struct Toast {
    #[base]
    base: Base<PanelContainer>,

    config: ToastConfig,
    label: Option<Gd<Label>>,
    close_button: Option<Gd<Button>>,
    is_dismissing: bool,
}

#[godot_api]
impl IPanelContainer for Toast {
    fn ready(&mut self) {
        self.label = self
            .base()
            .get_node_or_null("%Label")
            .and_then(|node| node.try_cast::<Label>().ok());
        self.close_button = self
            .base()
            .get_node_or_null("%CloseButton")
            .and_then(|node| node.try_cast::<Button>().ok());

        self.apply_config();
    }
}

#[godot_api]
impl Toast {
    pub fn set_config(&mut self, config: ToastConfig) {
        self.config = config;
        if let Some(label) = self.label.as_mut() {
            label.set_text(&self.config.text);
        }
    }

    pub fn show(&mut self) {
        self.base_mut().set_visible(true);
        self.base_mut()
            .set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));

        if self.config.animation.enable {
            self.play_entrance_animation();
        }

        if self.config.auto_dismiss {
            self.schedule_dismiss();
        }
    }

    pub fn dismiss(&mut self) {
        if self.is_dismissing {
            return;
        }
        self.is_dismissing = true;

        if self.config.animation.enable {
            self.play_exit_animation();
        } else {
            self.base_mut().queue_free();
        }
    }

    fn apply_config(&mut self) {
        let mut style_box = StyleBoxFlat::new_gd();

        style_box.set_bg_color(self.config.get_background_color());
        style_box.set_corner_radius_all(self.config.corner_radius as i32);

        self.base_mut()
            .add_theme_stylebox_override("panel", &style_box);

        let width = self.config.width;
        self.base_mut()
            .set_custom_minimum_size(Vector2::new(width, 50.0));

        if let Some(label) = self.label.as_mut() {
            label.set_text(&self.config.text);
            label.add_theme_color_override("font_color", self.config.text_color);
        }else {
            godot_warn!("Toast: Label node not found.");
        }

        if let Some(close_button) = self.close_button.as_mut() {
            close_button.set_visible(self.config.show_close_button);
            close_button
                .clone()
                .signals()
                .pressed()
                .connect_other(self, Self::dismiss);
        }else {
            godot_warn!("Toast: CloseButton node not found.");
        }
    }

    fn schedule_dismiss(&mut self) {
        let mut tree = self.base().get_tree();
        let callback = self.base().callable("on_timer_timeout");
        let mut timer = tree.create_timer(self.config.duration.as_seconds() as f64);
        let _ = timer.connect("timeout", &callback);
    }

    #[func]
    fn on_timer_timeout(&mut self) {
        self.dismiss();
    }

    fn play_entrance_animation(&mut self) {
        let duration = self.config.animation.duration;
        let mut color = self.base().get_modulate();
        color.a = 0.0;
        self.base_mut().set_modulate(color);

        let callback = self.base().callable("set_alpha_1");
        let mut tree = self.base().get_tree();
        let mut timer = tree.create_timer(duration as f64);
        let _ = timer.connect("timeout", &callback);
    }

    fn play_exit_animation(&mut self) {
        let duration = self.config.animation.duration;

        let callback = self.base().callable("set_alpha_0");
        let complete_callback = self.base().callable("on_exit_animation_complete");

        let mut tree = self.base().get_tree();
        let mut timer: Gd<godot::classes::SceneTreeTimer> = tree.create_timer(duration as f64);
        timer.connect("timeout", &callback);

        let mut timer2: Gd<godot::classes::SceneTreeTimer> = tree.create_timer(duration as f64);
        timer2.connect("timeout", &complete_callback);
    }

    #[func]
    fn set_alpha_0(&mut self) {
        let mut color = self.base().get_modulate();
        color.a = 0.0;
        self.base_mut().set_modulate(color);
    }

    #[func]
    fn set_alpha_1(&mut self) {
        let mut color = self.base().get_modulate();
        color.a = 1.0;
        self.base_mut().set_modulate(color);
    }

    #[func]
    fn on_exit_animation_complete(&mut self) {
        self.base_mut().queue_free();
    }

    pub fn set_position(&mut self, pos: Vector2) {
        self.base_mut().set_position(pos);
    }

    pub fn from_scene() -> Option<Gd<Self>> {
        let scene = ResourceLoader::singleton().load(TOAST_SCENE_PATH);
        let scene = scene.and_then(|res| res.try_cast::<PackedScene>().ok())?;

        let instance = scene.instantiate()?;
        instance.try_cast::<Self>().ok()
    }
}
