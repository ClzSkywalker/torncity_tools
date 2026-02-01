use std::sync::OnceLock;

use model::office_sell::OfficeSellItem;

pub static OFFICE_SELL_LIST: OnceLock<Vec<OfficeSellItem>> = OnceLock::new();

// 加载 office_sell.json 文件
pub fn get_office_sell_list() -> &'static Vec<OfficeSellItem> {
    OFFICE_SELL_LIST.get_or_init(|| {
        // let res = ResourceLoader::singleton().get_resource("res://assets/data/office_sell.json", "JSON").unwrap();
        // let list = ResourceLoader::load("res://assets/data/office_sell.json", "JSON").unwrap();
        // list
        Vec::new()
    })
}
