use crate::{
    scenes::{weav3r_scene::Weav3rScene, weav3r_setting_scene::Weav3rSettingScene},
};
use godot::{classes::*, prelude::*};
use tools::node::{INodeFunc, INodeTool};

#[derive(GodotClass)]
#[class(init,base=Node)]
pub struct MainScene {
    #[base]
    base: Base<Node>,
    weav3r_page: Option<Gd<Weav3rScene>>,
    settings_page: Option<Gd<Weav3rSettingScene>>,
    home_button: Option<Gd<Button>>,
    settings_button: Option<Gd<Button>>,
}

#[godot_api]
impl INode for MainScene {
    fn ready(&mut self) {
        self.weav3r_page = self.get_node_as::<Weav3rScene>("%Weav3rPage");
        self.settings_page = self.get_node_as::<Weav3rSettingScene>("%Weav3rSettingPage");
        self.home_button = self.get_node_as::<Button>("%Home");
        self.settings_button = self.get_node_as::<Button>("%Setting");
        if let Some(home_button) = self.home_button.as_mut() {
            home_button
                .clone()
                .signals()
                .pressed()
                .connect_other(self, Self::on_home_button_pressed);
        }
        if let Some(settings_button) = self.settings_button.as_mut() {
            settings_button
                .clone()
                .signals()
                .pressed()
                .connect_other(self, Self::on_settings_button_pressed);
        }
    }
}

impl INodeFunc for MainScene {
    fn node_path() -> &'static str {
        "res://scenes/main_scene.tscn"
    }
}

impl MainScene {
    fn on_home_button_pressed(&mut self) {
        if let Some(weav3r_page) = self.weav3r_page.as_mut() {
            weav3r_page.show();
        }
        if let Some(settings_page) = self.settings_page.as_mut() {
            settings_page.hide();
        }
    }

    fn on_settings_button_pressed(&mut self) {
        if let Some(weav3r_page) = self.weav3r_page.as_mut() {
            weav3r_page.hide();
        }
        if let Some(settings_page) = self.settings_page.as_mut() {
            settings_page.show();
        }
    }
}
