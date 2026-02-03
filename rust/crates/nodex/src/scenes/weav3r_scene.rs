use godot::{classes::*, prelude::*};
use model::weav3r::favorites::FavoritesResponse;
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
    /// 每个 item 的期望宽度（用于计算列数）
    #[init(val = 300.0)]
    item_width: f32,
}

#[godot_api]
impl IControl for Weav3rScene {
    fn ready(&mut self) {
        self.favorites_res.filter.office_item_map = torn_logic::item::get_item_info_map().clone();

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

        if let Some(timer_controller) = self.timer_controller.as_mut() {
            timer_controller
                .clone()
                .signals()
                .pressed()
                .connect_other(self, Self::on_timer_controller_pressed);
        }

        self.base()
            .clone()
            .signals()
            .resized()
            .connect_other(self, Self::on_resized);

        // 启动时先请求一次
        self.send_request();
        self.update_columns();
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

        let f_target_ids = filter_id_text
            .split(',')
            .filter_map(|x| x.trim().parse::<i32>().ok())
            .collect::<Vec<i32>>();
        self.favorites_res.filter.target_ids = f_target_ids.clone();

        let target_ids = torn_logic::item::get_item_list()
            .iter()
            .filter(|x| x.tradeable)
            .map(|x| x.id)
            .chain(f_target_ids)
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let next_action = setting_data.get_next_action();

        let Some(http) = self.http_request.as_mut() else {
            godot_error!("Weav3rScene: HTTPRequest node not found.");
            return;
        };
        http.bind_mut()
            .send_request(GString::from(&target_ids), next_action);
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
                "Weav3rScene: Failed to get response.code: {}, body: {}",
                response_code,
                String::from_utf8_lossy(body.as_slice()).to_string()
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

        self.favorites_res.filter.min_profit = setting_data.get_min_profit();
        self.favorites_res.filter.min_profit_percentage = setting_data.get_profit_percent();
        self.favorites_res.filter.office_sell_price = setting_data.get_office_sell_price();

        let response_text = String::from_utf8_lossy(body.as_slice());
        let favorites_response = match FavoritesResponse::from_text(&response_text) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rScene: Failed to parse favorites response. Error: {:?}",
                    err,
                );
                godot_error!(
                    "Weav3rScene: Original response text (first 500 chars): {}",
                    if response_text.len() > 500 {
                        format!("{}...", &response_text[..500])
                    } else {
                        response_text.to_string()
                    }
                );
                return;
            }
        };
        self.favorites_res.set_new_profit(favorites_response.items);
        if let Some(audio_player) = self.audio_player.as_mut()
            && self.favorites_res.has_new
            && setting_data.get_audio_switch()
        {
            godot_print!("Weav3rScene: Has new data.");
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

    #[func]
    fn on_resized(&mut self) {
        self.update_columns();
    }

    fn update_columns(&mut self) {
        let available_width = self.base().get_size().x;
        let columns = (available_width / self.item_width).max(1.0) as i32;

        let Some(grid_container) = self.grid_container.as_mut() else {
            return;
        };
        grid_container.set_columns(columns);
    }

    fn on_timer_controller_pressed(&mut self) {
        if let Some(timer) = self.timer.as_mut() {
            let is_paused = timer.is_paused();
            timer.set_paused(!is_paused);
            if let Some(timer_controller) = self.timer_controller.as_mut() {
                timer_controller.set_text(if timer.is_paused() {
                    "Start Request"
                } else {
                    "Stop Request"
                });
            }
        }
    }
}
