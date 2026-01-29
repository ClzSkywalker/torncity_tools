use crate::{
    node::prelude::*, prelude::Weav3rItem,
    scenes::weav3r_setting_scene::Weav3rSettingScene,
};
use godot::{
    classes::{
        AudioStreamPlayer, Button, Control, GridContainer, IControl, Node, PackedScene,
        ResourceLoader, Timer,
    },
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

impl Weav3rMainScene {
    const BUTTON_PATH: &str = "Button";
    const HTTP_REQUEST_PATH: &str = "HTTPRequest";
    const GRID_CONTAINER_PATH: &str = "ScrollContainer/GridContainer";
    const TIMER_PATH: &str = "Timer";
    const SETTINGS_BUTTON_PATH: &str = "%SettingsButton";
}

#[godot_api]
impl IControl for Weav3rMainScene {
    fn ready(&mut self) {
        self.button = self.get_node_as::<Button>(Self::BUTTON_PATH);
        self.timer = self.get_node_as::<Timer>(Self::TIMER_PATH);
        self.http_request = self.get_node_as::<Weav3rHttpRequest>(Self::HTTP_REQUEST_PATH);
        self.grid_container = self.get_node_as::<GridContainer>(Self::GRID_CONTAINER_PATH);
        self.settings_button = self.get_node_as::<Button>(Self::SETTINGS_BUTTON_PATH);

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

        let interval = cfg.read_config_f64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_INTERVAL,
            Weav3rSettingData::DEFAULT_INTERVAL,
        );

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

        if let Some(settings_button) = self.get_node_as::<Button>(Self::SETTINGS_BUTTON_PATH) {
            settings_button
                .signals()
                .pressed()
                .connect_other(self, Self::on_settings_pressed);
        } else {
            godot_error!("Weav3rMainScene: SettingsButton node not found.");
        }
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
        let filter_id_text = cfg.read_config_string(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_FILTER_IDS,
            Weav3rSettingData::DEFAULT_FILTER_IDS,
        );
        if filter_id_text.trim().is_empty() {
            godot_error!("Weav3rMainScene: FilterIdEdit is empty.");
            return;
        }
        let target_ids = filter_id_text.split(',').collect::<Vec<&str>>().join(",");

        let Some(mut http) = self.get_node_as::<Weav3rHttpRequest>(Self::HTTP_REQUEST_PATH) else {
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

        let profit_percentage = cfg.read_config_f64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_PROFIT_PERCENT,
            Weav3rSettingData::DEFAULT_PROFIT_PERCENT,
        );
        let profit_minimum_value = cfg.read_config_i32(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_MIN_PROFIT,
            Weav3rSettingData::DEFAULT_MIN_PROFIT,
        );

        let response_text = String::from_utf8_lossy(body.as_slice());
        let Ok(favorites_response) = weav3r::favorite::parse_favorites_response(&response_text)
        else {
            godot_error!("Weav3rMainScene: Failed to parse favorites response.");
            return;
        };

        let profit_items = weav3r::profit::filter(
            favorites_response,
            weav3r::profit::Filter {
                min_profit: profit_minimum_value,
                min_profit_percentage: profit_percentage as f32,
                ignore_names: vec![],
                filter_items: vec![
                    weav3r::profit::FilterItem {
                        name: "Xanax".to_string(),
                        price: 800000,
                        // profit_percentage: 10.0,
                        ..Default::default()
                    },
                    weav3r::profit::FilterItem {
                        name: "Donator Pack".to_string(),
                        price: 23000000,
                        // profit_percentage: 10.0,
                        ..Default::default()
                    },
                    weav3r::profit::FilterItem {
                        name: "Feathery Hotel Coupon".to_string(),
                        price: 11500000,
                        // profit_percentage: 10.0,
                        ..Default::default()
                    },
                ],
            },
        );

        let now = tools::time::get_current_time();

        let (mut resp, has_new) = weav3r::profit::calc_profit(now, self.favorites_res.clone(), profit_items);
        let sorted_user_profit_result = weav3r::profit::sort_profit(
            weav3r::profit::SortProfitParams { recent_sec: 30 },
            resp.user_profit_result,
        );

        if has_new {
            godot_print!("Weav3rMainScene: Has new data.");
            if let Some(audio_player) = self.audio_player.as_mut() {
                audio_player.play();
            }
        }

        resp.user_profit_result = sorted_user_profit_result.clone();
        self.favorites_res = resp;
        self.render_list(sorted_user_profit_result);
    }

    fn render_list(&mut self, items: Vec<ProfitUserInfo>) {
        let Some(mut grid_container) = self.get_node_as::<GridContainer>(Self::GRID_CONTAINER_PATH)
        else {
            godot_error!("Weav3rMainScene: GridContainer node not found.");
            return;
        };

        let Some(scene) = ResourceLoader::singleton()
            .load("res://scenes/weav3r_item.tscn")
            .and_then(|res| res.try_cast::<PackedScene>().ok())
        else {
            godot_error!("Weav3rMainScene: Failed to load weav3r_item.tscn");
            return;
        };

        let children = grid_container.get_children();
        for child in children.iter_shared() {
            let mut child = child.clone();
            child.queue_free();
        }

        for item in items {
            let Some(node) = scene.instantiate() else {
                godot_error!("Weav3rMainScene: Failed to instance Weav3rItem");
                continue;
            };
            let Ok(mut item_node) = node.try_cast::<Weav3rItem>() else {
                godot_error!("Weav3rMainScene: Instance is not Weav3rItem");
                continue;
            };
            item_node.bind_mut().set_item(item);
            let child = item_node.upcast::<Node>();
            grid_container.add_child(Some(&child));
        }
    }
}
