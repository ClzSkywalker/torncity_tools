use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeSellItem {
    pub id: i32,
    pub name: String,
    pub sell: i64,
    pub stack: bool,
    pub in_shop: bool,
}
