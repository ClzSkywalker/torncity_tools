use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    a: String,
    f: String,
    b: String,
    q: String,
    i: bool,
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

#[derive(Debug, Clone)]
pub struct FavoritesResponse {
    pub metadata: Metadata,
    pub items: Vec<ProductionItem>,
}

impl fmt::Display for FavoritesResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "元数据:")?;
        writeln!(f, "  a: {}", self.metadata.a)?;
        writeln!(f, "  b: {}", self.metadata.b)?;
        writeln!(f, "  i: {}", self.metadata.i)?;
        writeln!(f, "\n物品列表 ({} 个):", self.items.len())?;
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

pub fn parse_favorites_response(
    response_text: &str,
) -> Result<FavoritesResponse, Box<dyn std::error::Error>> {
    let mut metadata: Option<Metadata> = None;
    let mut items: Option<Vec<ProductionItem>> = None;

    for line in response_text.lines() {
        if let Some(colon_pos) = line.find(':') {
            let index = &line[..colon_pos];
            let json_part = &line[colon_pos + 1..];

            match index {
                "0" => {
                    metadata = Some(serde_json::from_str(json_part)?);
                }
                "1" => {
                    items = Some(serde_json::from_str(json_part)?);
                }
                _ => {
                    println!("警告: 未知的索引 {}", index);
                }
            }
        }
    }

    let metadata = metadata.ok_or("缺少元数据")?;
    let items = items.ok_or("缺少物品列表")?;

    // 构建响应结构
    let favorites_response = FavoritesResponse { metadata, items };
    Ok(favorites_response)
}
