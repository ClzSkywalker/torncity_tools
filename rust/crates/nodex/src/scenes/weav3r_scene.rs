use godot::{classes::*, prelude::*};
use tools::node::{INodeFunc, INodeTool};
use weav3r::{
    data::Weav3rSettingData,
    profit::{FavoritesRes, ProfitUserInfo},
};

use crate::prelude::{Weav3rHttpRequest, Weav3rItem};

#[derive(GodotClass)]
#[class(init,base=Control)]
pub struct Weav3rScene {
    #[base]
    base: Base<Control>,
    http_request: Option<Gd<Weav3rHttpRequest>>,
    timer: Option<Gd<Timer>>,
    grid_container: Option<Gd<GridContainer>>,
    audio_player: Option<Gd<AudioStreamPlayer>>,
    timer_controller: Option<Gd<Button>>,
    favorites_res: FavoritesRes,
}

#[godot_api]
impl IControl for Weav3rScene {
    fn ready(&mut self) {
        self.http_request = self.get_node_as::<Weav3rHttpRequest>("HTTPRequest");
        self.timer = self.get_node_as::<Timer>("Timer");
        self.grid_container = self.get_node_as::<GridContainer>("%GridContainer");
        self.audio_player = self.get_node_as::<AudioStreamPlayer>("AudioStreamPlayer");
        self.timer_controller = self.get_node_as::<Button>("%TimerController");

        if let Some(http) = &self.http_request {
            let http = http.clone();
            http.signals()
                .request_completed()
                .connect_other(self, Self::on_request_completed);
        } else {
            godot_error!("Weav3rScene: HTTPRequest node not found.");
        }

        let cfg = match tools::cfg::CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rScene: Failed to load {:?}: {:?}",
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
                .connect_other(self, Self::send_request);
        } else {
            godot_error!("Weav3rScene: Timer node not found.");
        }

        // 启动时先请求一次
        self.send_request();
    }
}

impl INodeFunc for Weav3rScene {
    fn node_path() -> &'static str {
        "res://scenes/weav3r.tscn"
    }
}

#[godot_api]
impl Weav3rScene {
    #[func]
    fn send_request(&mut self) {
        let cfg = match tools::cfg::CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rScene: Failed to load {:?}: {:?}",
                    Weav3rSettingData::SETTINGS_PATH,
                    err
                );
                return;
            }
        };
        let setting_data = Weav3rSettingData::new(cfg);
        let filter_id_text = setting_data.get_filter_ids();
        if filter_id_text.trim().is_empty() {
            godot_error!("Weav3rScene: FilterIdEdit is empty.");
            return;
        }
        let target_ids = filter_id_text.split(',').collect::<Vec<&str>>().join(",");

        let Some(http) = self.http_request.as_mut() else {
            godot_error!("Weav3rScene: HTTPRequest node not found.");
            return;
        };
        http.bind_mut().send_request(GString::from(&target_ids));
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
                "Weav3rScene: Failed to get response.code: {}",
                response_code
            );
            return;
        }

        let cfg = match tools::cfg::CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rScene: Failed to load {:?}: {:?}",
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
            godot_error!("Weav3rScene: Failed to parse favorites response.");
            return;
        };

        self.favorites_res.set_new_profit(favorites_response.items);

        if !self.favorites_res.has_new {
            return;
        }

        godot_print!("Weav3rScene: Has new data.");
        if let Some(audio_player) = self.audio_player.as_mut() {
            audio_player.play();
        }
        self.render_list(self.favorites_res.user_profit_result.clone());
    }

    fn render_list(&mut self, items: Vec<ProfitUserInfo>) {
        let Some(grid_container) = self.grid_container.as_mut() else {
            godot_error!("Weav3rScene: GridContainer node not found.");
            return;
        };

        let children = grid_container.get_children();
        for child in children.iter_shared() {
            let mut child = child.clone();
            child.queue_free();
        }

        for item in items {
            let Some(mut weav3r_item) = Weav3rItem::get_scene_instance() else {
                godot_error!("Weav3rScene: Failed to get Weav3rItem");
                continue;
            };
            weav3r_item.bind_mut().set_item(item);
            let child = weav3r_item.upcast::<Node>();
            grid_container.add_child(Some(&child));
        }
    }
}
