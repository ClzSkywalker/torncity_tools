use godot::classes::file_access::ModeFlags;
use godot::{classes::FileAccess, global::godot_error};
use std::sync::OnceLock;

use model::office_sell::OfficeSellItem;

pub static OFFICE_SELL_LIST: OnceLock<Vec<OfficeSellItem>> = OnceLock::new();

/// 加载官方售卖价格配置
pub fn get_office_sell_list() -> &'static Vec<OfficeSellItem> {
    OFFICE_SELL_LIST.get_or_init(|| {
        let Some(file) = FileAccess::open("res://assets/data/office_sell.json", ModeFlags::READ)
        else {
            godot_error!("Failed to open office_sell.json");
            return Vec::new();
        };

        let text = file.get_as_text();
        let text = text.to_string();

        // 反序列化 JSON
        let list: Vec<OfficeSellItem> = serde_json::from_str(&text).unwrap_or_else(|err| {
            godot_error!("Failed to parse office_sell.json: {:?}", err);
            Vec::new()
        });

        list
    })
}
