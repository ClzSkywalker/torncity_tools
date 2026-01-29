use crate::{
    node::prelude::*, prelude::Weav3rItem, scenes::weav3r_setting_scene::Weav3rSettingScene,
};
use godot::{
    classes::{AudioStreamPlayer, Button, Control, GridContainer, IControl, Node, Timer},
    global::Error,
    prelude::*,
};
use tools::node::{INodeFunc, INodeTool};
use weav3r::{
    data::Weav3rSettingData,
    profit::{FavoritesRes, ProfitUserInfo},
};

#[derive(GodotClass)]
#[class(init,base=Control)]
pub struct Weav3rMainScene {
    #[base]
    base: Base<Control>,
    favorites_res: FavoritesRes,
    button: Option<Gd<Button>>,
    timer: Option<Gd<Timer>>,
    http_request: Option<Gd<Weav3rHttpRequest>>,
    grid_container: Option<Gd<GridContainer>>,
    settings_button: Option<Gd<Button>>,
    audio_player: Option<Gd<AudioStreamPlayer>>,
}

#[godot_api]
impl IControl for Weav3rMainScene {
    fn ready(&mut self) {
        self.button = self.get_node_as::<Button>("Button");
        self.timer = self.get_node_as::<Timer>("Timer");
        self.http_request = self.get_node_as::<Weav3rHttpRequest>("HTTPRequest");
        self.grid_container = self.get_node_as::<GridContainer>("ScrollContainer/GridContainer");
        self.settings_button = self.get_node_as::<Button>("%SettingsButton");

        if let Some(button) = &self.button {
            let button = button.clone();
            button
                .signals()
                .pressed()
                .connect_other(self, Self::on_button_pressed);
        }

        if let Some(http) = &self.http_request {
            let http = http.clone();
            http.signals()
                .request_completed()
                .connect_other(self, Self::on_request_completed);
        } else {
            godot_error!("Weav3rMainScene: HTTPRequest node not found.");
        }

        let cfg = match tools::cfg::CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rMainScene: Failed to load {:?}: {:?}",
                    Weav3rSettingData::SETTINGS_PATH,
                    err
                );
                return;
            }
        };

        let setting_data = Weav3rSettingData::new(cfg);
        let interval = setting_data.get_interval();

        if let Some(timer) = self.timer.as_mut() {
            let mut timer = timer.clone();
            timer.set_wait_time(interval);
            timer
                .signals()
                .timeout()
                .connect_other(self, Self::on_timer_timeout);
        } else {
            godot_error!("Weav3rMainScene: Timer node not found.");
        }

        if let Some(settings_button) = self.settings_button.as_mut() {
            let settings_button = settings_button.clone();
            settings_button
                .signals()
                .pressed()
                .connect_other(self, Self::on_settings_pressed);
        } else {
            godot_error!("Weav3rMainScene: SettingsButton node not found.");
        }

        // 启动时先请求一次
        self.on_button_pressed();
    }
}

impl INodeFunc for Weav3rMainScene {
    fn node_path() -> &'static str {
        "res://scenes/main.tscn"
    }
}

#[godot_api]
impl Weav3rMainScene {
    #[func]
    fn on_button_pressed(&mut self) {
        let cfg = match tools::cfg::CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rMainScene: Failed to load {:?}: {:?}",
                    Weav3rSettingData::SETTINGS_PATH,
                    err
                );
                return;
            }
        };
        let setting_data = Weav3rSettingData::new(cfg);
        let filter_id_text = setting_data.get_filter_ids();
        if filter_id_text.trim().is_empty() {
            godot_error!("Weav3rMainScene: FilterIdEdit is empty.");
            return;
        }
        let target_ids = filter_id_text.split(',').collect::<Vec<&str>>().join(",");

        let Some(http) = self.http_request.as_mut() else {
            godot_error!("Weav3rMainScene: HTTPRequest node not found.");
            return;
        };
        http.bind_mut().send_request(GString::from(&target_ids));
    }

    #[func]
    fn on_timer_timeout(&mut self) {
        self.on_button_pressed();
    }

    #[func]
    fn on_settings_pressed(&mut self) {
        let Some(mut tree) = self.base().get_tree() else {
            godot_error!("Weav3rMainScene: SceneTree not found.");
            return;
        };
        let err = tree.change_scene_to_file(Weav3rSettingScene::node_path());
        if err != Error::OK {
            godot_error!("Weav3rMainScene: Failed to open settings scene: {:?}", err);
        }
    }

    #[func]
    fn on_request_completed(
        &mut self,
        _result: i64,
        response_code: i64,
        _headers: PackedStringArray,
        body: PackedByteArray,
    ) {
        if response_code != 200 {
            godot_error!(
                "Weav3rMainScene: Failed to get response.code: {}",
                response_code
            );
            return;
        }

        let cfg = match tools::cfg::CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rMainScene: Failed to load {:?}: {:?}",
                    Weav3rSettingData::SETTINGS_PATH,
                    err
                );
                return;
            }
        };

        let setting_data = Weav3rSettingData::new(cfg);
        let profit_percentage = setting_data.get_profit_percent();
        let profit_minimum_value = setting_data.get_min_profit();

        self.favorites_res.filter.min_profit = profit_minimum_value;
        self.favorites_res.filter.min_profit_percentage = profit_percentage as f32;

        let response_text = String::from_utf8_lossy(body.as_slice());
        let Ok(favorites_response) =
            model::weav3r::favorites::FavoritesResponse::from_text(&response_text)
        else {
            godot_error!("Weav3rMainScene: Failed to parse favorites response.");
            return;
        };

        self.favorites_res.set_new_profit(favorites_response.items);

        if !self.favorites_res.has_new {
            return;
        }

        godot_print!("Weav3rMainScene: Has new data.");
        if let Some(audio_player) = self.audio_player.as_mut() {
            audio_player.play();
        }
        self.render_list(self.favorites_res.user_profit_result.clone());
    }

    fn render_list(&mut self, items: Vec<ProfitUserInfo>) {
        let Some(grid_container) = self.grid_container.as_mut() else {
            godot_error!("Weav3rMainScene: GridContainer node not found.");
            return;
        };

        let children = grid_container.get_children();
        for child in children.iter_shared() {
            let mut child = child.clone();
            child.queue_free();
        }

        for item in items {
            let Some(mut weav3r_item) = Weav3rItem::get_scene_instance() else {
                godot_error!("Weav3rMainScene: Failed to get Weav3rItem");
                continue;
            };
            weav3r_item.bind_mut().set_item(item);
            let child = weav3r_item.upcast::<Node>();
            grid_container.add_child(Some(&child));
        }
    }
}
