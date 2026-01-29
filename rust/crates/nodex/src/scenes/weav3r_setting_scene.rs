use godot::{
    classes::{Button, Control, IControl, TextEdit},
    prelude::*,
};
use tools::{
    cfg::CfgTool,
    node::{INodeFunc, INodeTool},
};
use weav3r::data::Weav3rSettingData;

use crate::prelude::Weav3rMainScene;

#[derive(GodotClass)]
#[class(init,base=Control)]
pub struct Weav3rSettingScene {
    #[base]
    base: Base<Control>,
    // @onready var 对应的字段
    interval_edit: Option<Gd<TextEdit>>,
    profit_percent_edit: Option<Gd<TextEdit>>,
    min_profit_edit: Option<Gd<TextEdit>>,
    filter_id_edit: Option<Gd<TextEdit>>,
    save_button: Option<Gd<Button>>,
}

#[godot_api]
impl IControl for Weav3rSettingScene {
    fn ready(&mut self) {
        // 在 ready 中初始化 @onready 变量，类似 GDScript 的 @onready var
        self.interval_edit = self.get_node_as::<TextEdit>("%IntervalEdit");
        self.profit_percent_edit = self.get_node_as::<TextEdit>("%ProfitPercentEdit");
        self.min_profit_edit = self.get_node_as::<TextEdit>("%MinProfitEdit");
        self.filter_id_edit = self.get_node_as::<TextEdit>("%FilterIdEdit");
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
            interval_edit.set_text(format!("{:.2}", interval).as_str());
        } else {
            godot_error!("Weav3rSettingScene: IntervalEdit node not found.");
        }
        if let Some(profit_percent_edit) = self.profit_percent_edit.as_mut() {
            let profit_percent = setting_data.get_profit_percent();
            profit_percent_edit.set_text(format!("{:.2}", profit_percent).as_str());
        } else {
            godot_error!("Weav3rSettingScene: ProfitPercentEdit node not found.");
        }
        if let Some(min_profit_edit) = self.min_profit_edit.as_mut() {
            let min_profit = setting_data.get_min_profit();
            min_profit_edit.set_text(format!("{}", min_profit).as_str());
        } else {
            godot_error!("Weav3rSettingScene: MinProfitEdit node not found.");
        }
        if let Some(filter_id_edit) = self.filter_id_edit.as_mut() {
            let filter_id = setting_data.get_filter_ids();
            filter_id_edit.set_text(filter_id.as_str());
        } else {
            godot_error!("Weav3rSettingScene: FilterIdEdit node not found.");
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
    fn on_save_pressed(&mut self) {
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
            let interval = interval_edit.get_text().to_float();
            setting_data.set_interval(interval);
        }
        if let Some(profit_percent_edit) = &self.profit_percent_edit {
            let profit_percent = profit_percent_edit.get_text().to_float();
            setting_data.set_profit_percent(profit_percent);
        }
        if let Some(min_profit_edit) = &self.min_profit_edit {
            let min_profit = min_profit_edit.get_text().to_int();
            setting_data.set_min_profit(min_profit);
        }
        if let Some(filter_id_edit) = &self.filter_id_edit {
            let filter_id = filter_id_edit.get_text().strip_edges(true, true);
            setting_data.set_filter_ids(&filter_id.to_string());
        }
        if let Err(err) = setting_data.save() {
            godot_error!(
                "Weav3rSettingScene: Failed to save {:?}: {:?}",
                Weav3rSettingData::SETTINGS_PATH,
                err
            );
        }
        let Some(mut tree) = self.base().get_tree() else {
            godot_error!("Weav3rSettingScene: SceneTree not found.");
            return;
        };
        let err = tree.change_scene_to_file(Weav3rMainScene::node_path());
        if err != godot::global::Error::OK {
            godot_error!("Weav3rSettingScene: Failed to change scene: {:?}", err);
        }
    }
}
