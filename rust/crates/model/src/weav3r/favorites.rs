use crate::error::MyError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct FavoritesResponse {
    pub items: Vec<ProductionItem>,
}

impl FavoritesResponse {
    pub fn from_text(text: &str) -> Result<Self, MyError> {
        for line in text.lines() {
            if let Some(colon_pos) = line.find(':') {
                let index = &line[..colon_pos];
                let json_part = &line[colon_pos + 1..];

                if index != "1" {
                    continue;
                }
                let item: Vec<ProductionItem> = serde_json::from_str(json_part)
                    .map_err(|e| MyError::JsonParse(e.to_string()))?;
                return Ok(Self { items: item });
            }
        }
        Ok(Self::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionItem {
    pub id: i32,
    pub name: String,
    pub image: String,
    pub market_price: i32,
    pub avg_bazaar_price: i32,
    pub cheapest_bazaars: Vec<BazaarPriceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BazaarPriceInfo {
    pub player_id: i32,
    pub player_name: String,
    pub quantity: i32,
    pub price: i32,
    pub total_value: String,
}

impl fmt::Display for FavoritesResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "物品列表 ({} 个):", self.items.len())?;
        for item in &self.items {
            writeln!(f, "\n物品 ID: {}", item.id)?;
            writeln!(f, "  名称: {}", item.name)?;
            writeln!(f, "  市场价: {}", item.market_price)?;
            writeln!(f, "  平均集市价: {}", item.avg_bazaar_price)?;
            writeln!(f, "  最便宜的集市 (前5个):")?;
            for (idx, bazaar) in item.cheapest_bazaars.iter().take(5).enumerate() {
                writeln!(
                    f,
                    "    {}. {} (ID: {}) - 价格: {}, 数量: {}, 总价值: {}",
                    idx + 1,
                    bazaar.player_name,
                    bazaar.player_id,
                    bazaar.price,
                    bazaar.quantity,
                    bazaar.total_value
                )?;
            }
        }
        Ok(())
    }
}
