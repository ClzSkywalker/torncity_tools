use godot::{
    classes::{Button, Control, DisplayServer, IControl, SpinBox, TextEdit},
    prelude::*,
};
use tools::{
    cfg::CfgTool,
    node::{INodeFunc, INodeTool},
};
use weav3r::data::Weav3rSettingData;

#[derive(GodotClass)]
#[class(init,base=Control)]
pub struct Weav3rSettingScene {
    #[base]
    base: Base<Control>,
    // @onready var 对应的字段
    interval_edit: Option<Gd<SpinBox>>,
    profit_percent_edit: Option<Gd<SpinBox>>,
    min_profit_edit: Option<Gd<SpinBox>>,
    filter_id_edit: Option<Gd<TextEdit>>,
    office_sell_price_edit: Option<Gd<SpinBox>>,
    office_sell_profit_edit: Option<Gd<SpinBox>>,
    token_edit: Option<Gd<TextEdit>>,
    cookie_edit: Option<Gd<TextEdit>>,
    parse_weav3r_curl_btn: Option<Gd<Button>>,
    save_button: Option<Gd<Button>>,
}

#[godot_api]
impl IControl for Weav3rSettingScene {
    fn ready(&mut self) {
        // 在 ready 中初始化 @onready 变量，类似 GDScript 的 @onready var
        self.interval_edit = self.get_node_as::<SpinBox>("%IntervalEdit");
        self.profit_percent_edit = self.get_node_as::<SpinBox>("%ProfitPercentEdit");
        self.min_profit_edit = self.get_node_as::<SpinBox>("%MinProfitEdit");
        self.filter_id_edit = self.get_node_as::<TextEdit>("%FilterIdEdit");
        self.office_sell_price_edit = self.get_node_as::<SpinBox>("%OfficeSellPriceEdit");
        self.office_sell_profit_edit = self.get_node_as::<SpinBox>("%OfficeSellProfitEdit");
        self.token_edit = self.get_node_as::<TextEdit>("%TokenEdit");
        self.cookie_edit = self.get_node_as::<TextEdit>("%CookieEdit");
        self.parse_weav3r_curl_btn = self.get_node_as::<Button>("%ParseWeav3rCurlBtna");

        self.save_button = self.get_node_as::<Button>("%SaveButton");

        let cfg = match CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rSettingScene: Failed to load {:?}: {:?}",
                    Weav3rSettingData::SETTINGS_PATH,
                    err
                );
                return;
            }
        };

        let setting_data = Weav3rSettingData::new(cfg);

        // 检查节点是否都找到了
        if let Some(interval_edit) = self.interval_edit.as_mut() {
            let interval = setting_data.get_interval();
            interval_edit.set_value(interval);
        } else {
            godot_error!("Weav3rSettingScene: IntervalEdit node not found.");
        }
        if let Some(profit_percent_edit) = self.profit_percent_edit.as_mut() {
            let profit_percent = setting_data.get_profit_percent();
            profit_percent_edit.set_value(profit_percent as f64);
        } else {
            godot_error!("Weav3rSettingScene: ProfitPercentEdit node not found.");
        }
        if let Some(min_profit_edit) = self.min_profit_edit.as_mut() {
            let min_profit = setting_data.get_min_profit();
            min_profit_edit.set_value(min_profit as f64);
        } else {
            godot_error!("Weav3rSettingScene: MinProfitEdit node not found.");
        }
        if let Some(filter_id_edit) = self.filter_id_edit.as_mut() {
            let filter_id = setting_data.get_filter_ids();
            filter_id_edit.set_text(filter_id.as_str());
        } else {
            godot_error!("Weav3rSettingScene: FilterIdEdit node not found.");
        }
        if let Some(edit) = self.office_sell_price_edit.as_mut() {
            let value = setting_data.get_office_sell_price();
            edit.set_value(value as f64);
        } else {
            godot_error!("Weav3rSettingScene: office_sell_price_edit node not found.");
        }
        if let Some(edit) = self.office_sell_profit_edit.as_mut() {
            let value = setting_data.get_office_sell_profit();
            edit.set_value(value as f64);
        } else {
            godot_error!("Weav3rSettingScene: office_sell_profit_edit node not found.");
        }
        if let Some(token_edit) = self.token_edit.as_mut() {
            let token = setting_data.get_next_action();
            token_edit.set_text(token.as_str());
        } else {
            godot_error!("Weav3rSettingScene: TokenEdit node not found.");
        }
        if let Some(cookie_edit) = self.cookie_edit.as_mut() {
            let cookie = setting_data.get_cookie();
            godot_print!("Cookie loaded from config - length: {}", cookie.len());
            cookie_edit.set_text(cookie.as_str());
            godot_print!("Cookie set to TextEdit after load - length: {}", cookie_edit.get_text().len());
        } else {
            godot_error!("Weav3rSettingScene: CookieEdit node not found.");
        }

        if let Some(save_button) = &self.save_button {
            let save_button = save_button.clone();
            save_button
                .signals()
                .pressed()
                .connect_other(self, Self::on_save_pressed);
        } else {
            godot_error!("Weav3rSettingScene: SaveButton node not found.");
        }
        if let Some(save_weav3r_token_btn) = &self.parse_weav3r_curl_btn {
            let save_weav3r_token_btn = save_weav3r_token_btn.clone();
            save_weav3r_token_btn
                .signals()
                .pressed()
                .connect_other(self, Self::on_save_weav3r_token_pressed);
        } else {
            godot_error!("Weav3rSettingScene: SaveWeav3rTokenBtn node not found.");
        }
    }
}

impl INodeFunc for Weav3rSettingScene {
    fn node_path() -> &'static str {
        "res://scenes/settings.tscn"
    }
}

#[godot_api]
impl Weav3rSettingScene {
    #[func]
    pub fn on_save_pressed(&mut self) {
        let cfg = match CfgTool::new(Weav3rSettingData::SETTINGS_PATH) {
            Ok(r) => r,
            Err(err) => {
                godot_error!(
                    "Weav3rSettingScene: Failed to load {:?}: {:?}",
                    Weav3rSettingData::SETTINGS_PATH,
                    err
                );
                return;
            }
        };
        let mut setting_data = Weav3rSettingData::new(cfg);
        if let Some(interval_edit) = &self.interval_edit {
            let interval = interval_edit.get_value();
            setting_data.set_interval(interval);
        }
        if let Some(profit_percent_edit) = &self.profit_percent_edit {
            let profit_percent = profit_percent_edit.get_value() as f32;
            setting_data.set_profit_percent(profit_percent);
        }
        if let Some(min_profit_edit) = &self.min_profit_edit {
            let min_profit = min_profit_edit.get_value().round() as i64;
            setting_data.set_min_profit(min_profit);
        }
        if let Some(filter_id_edit) = &self.filter_id_edit {
            let filter_id = filter_id_edit.get_text().strip_edges(true, true);
            setting_data.set_filter_ids(&filter_id.to_string());
        }
        if let Some(office_sell_price_edit) = &self.office_sell_price_edit {
            let office_sell_price = office_sell_price_edit.get_value() as u64;
            setting_data.set_office_sell_price(office_sell_price);
        }
        if let Some(office_sell_profit_edit) = &self.office_sell_profit_edit {
            let office_sell_profit = office_sell_profit_edit.get_value() as u64;
            setting_data.set_office_sell_profit(office_sell_profit);
        }
        if let Some(token_edit) = &self.token_edit {
            let token = token_edit.get_text().strip_edges(true, true);
            setting_data.set_next_action(&token.to_string());
        }
        if let Some(cookie_edit) = &self.cookie_edit {
            let cookie = cookie_edit.get_text().strip_edges(true, true);
            godot_print!("Cookie to save from TextEdit - length: {}", cookie.len());
            setting_data.set_cookie(&cookie.to_string());
        }

        if let Err(err) = setting_data.save() {
            godot_error!(
                "Weav3rSettingScene: Failed to save {:?}: {:?}",
                Weav3rSettingData::SETTINGS_PATH,
                err
            );
        } else {
            godot_print!("Weav3rSettingScene: Saved successfully");
        }
    }

    #[func]
    pub fn on_save_weav3r_token_pressed(&mut self) {
        let display_server = DisplayServer::singleton();
        let text = display_server.clipboard_get();
        if text.is_empty() {
            godot_warn!("Weav3rSettingScene: Clipboard is empty.");
            return;
        }
        let token = tools::http::HttpTool::from_curl(&text.to_string());
        let Ok(ht) = token else {
            godot_warn!("Weav3rSettingScene: Failed to get token from curl.");
            return;
        };

        let Some(next_action) = ht.headers.get("next-action").cloned() else {
            godot_warn!("Weav3rSettingScene: Failed to get next_action from curl.");
            return;
        };

        if next_action.is_empty() {
            godot_warn!("Weav3rSettingScene: next_action is empty.");
            return;
        }

        let Some(cookie) = ht.headers.get("cookie").cloned() else {
            godot_warn!("Weav3rSettingScene: Failed to get cookie from curl.");
            return;
        };

        if cookie.is_empty() {
            godot_warn!("Weav3rSettingScene: cookie is empty.");
            return;
        }

        godot_print!("Cookie from curl - length: {}", cookie.len());

        if let Some(token_edit) = self.token_edit.as_mut() {
            token_edit.set_text(next_action.as_str());
        }
        if let Some(cookie_edit) = self.cookie_edit.as_mut() {
            cookie_edit.set_text(cookie.as_str());
            godot_print!("Cookie set to TextEdit - length: {}", cookie_edit.get_text().len());
        }
    }
}
