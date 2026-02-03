use std::{collections::HashMap, sync::OnceLock};

use csv::ReaderBuilder;
use godot::{
    classes::{FileAccess, file_access::ModeFlags},
    global::godot_error,
};
use model::items::{CsvItemInfo, ItemInfo};

static OFFICE_ITEM_INFO_LIST: OnceLock<Vec<ItemInfo>> = OnceLock::new();
static OFFICE_ITEM_INFO_MAP: OnceLock<HashMap<i32, ItemInfo>> = OnceLock::new();
static WEAPON_ITEM_INFO_MAP: OnceLock<HashMap<i32, ItemInfo>> = OnceLock::new();

/// 获取物品列表
pub fn get_item_list() -> &'static Vec<ItemInfo> {
    OFFICE_ITEM_INFO_LIST.get_or_init(|| {
        let Some(file) = FileAccess::open("res://assets/data/torncity_items.csv", ModeFlags::READ)
        else {
            godot_error!("Failed to open torncity_items.csv");
            return Vec::new();
        };

        // 读取全部内容
        let length = file.get_length() as i64;
        let buffer = file.get_buffer(length);
        let buffer = buffer.to_vec();
        let content = String::from_utf8_lossy(&buffer);

        // 解析 CSV
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        reader
            .deserialize::<CsvItemInfo>()
            .filter_map(|result| match result {
                Ok(item) => Some(ItemInfo::from(item)),
                Err(e) => {
                    godot_error!("Failed to parse CSV row: {:?}", e);
                    None
                }
            })
            .collect()
    })
}

/// 获取物品 id map
pub fn get_item_info_map() -> &'static HashMap<i32, ItemInfo> {
    OFFICE_ITEM_INFO_MAP.get_or_init(|| {
        get_item_list()
            .iter()
            .map(|item| (item.id, item.clone()))
            .collect()
    })
}

/// 获取武器物品 id map
/// 武器是指 weapon_type 不为空的物品（排除 Temporary 类型）
pub fn weapon_id_map() -> &'static HashMap<i32, ItemInfo> {
    WEAPON_ITEM_INFO_MAP.get_or_init(|| {
        get_item_list()
            .iter()
            .filter(|item| !item.weapon_type.is_empty() && item.weapon_type != "Temporary")
            .map(|item| (item.id, item.clone()))
            .collect()
    })
}
