use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub effect: String,
    pub requirement: String,
    #[serde(rename = "type")]
    pub t: String,
    pub weapon_type: String,
    pub buy_price: u64,
    pub sell_price: u64,
    pub market_value: u64,
    pub circulation: u64,
    pub image: String,
    pub tradeable: bool,
}

// 临时结构体用于反序列化（因为 type 是 Rust 关键字）
#[derive(Debug, Deserialize)]
pub struct CsvItemInfo {
    id: i32,
    name: String,
    description: String,
    effect: String,
    requirement: String,
    #[serde(rename = "type")]
    t: String,
    weapon_type: String,
    buy_price: u64,
    sell_price: u64,
    market_value: u64,
    circulation: u64,
    image: String,
    #[serde(default)]
    tradeable: String, // 先读作字符串，再转换
}

impl From<CsvItemInfo> for ItemInfo {
    fn from(item: CsvItemInfo) -> Self {
        ItemInfo {
            id: item.id,
            name: item.name,
            description: item.description,
            effect: item.effect,
            requirement: item.requirement,
            t: item.t,
            weapon_type: item.weapon_type,
            buy_price: item.buy_price,
            sell_price: item.sell_price,
            market_value: item.market_value,
            circulation: item.circulation,
            image: item.image,
            tradeable: item.tradeable.to_uppercase() == "TRUE",
        }
    }
}
