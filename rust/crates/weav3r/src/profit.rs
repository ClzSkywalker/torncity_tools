use crate::favorite::{FavoritesResponse, ProductionItem};

#[derive(Debug, Clone, Default)]
pub struct FavoritesRes {
    pub profit_items_new: Vec<ProfitInfo>,
    pub profit_items_old: Vec<ProfitInfo>,
    pub user_profit_result: Vec<ProfitUserInfo>,
}

/// 用户维度 利润信息
#[derive(Debug, Clone, Default)]
pub struct ProfitUserInfo {
    pub player_id: i32,
    pub player_name: String,
    pub total_value: i32,
    pub profit_total_value: i32,
    pub profit_percentage: f32,
    pub created_on: u64, // 拉取到的时间戳
    pub items: Vec<ProfitInfo>,
}

/// 商品维度 利润信息
#[derive(Debug, Clone, Default)]
pub struct ProfitInfo {
    pub player_id: i32,
    pub player_name: String,
    pub quantity: i32,
    pub price: i32,
    pub total_value: i32,
    pub image: String,
    pub market_profit_percentage: f32,
    pub market_profit_single_value: i32,
    pub market_profit_total_value: i32,
    pub avg_bazaar_profit_percentage: f32,
    pub avg_bazaar_profit_single_value: i32,
    pub avg_bazaar_profit_total_value: i32,
    // 按照最低的数据进行复制
    pub profit_percentage: f32,
    pub profit_single_value: i32,
    pub profit_total_value: i32,

    pub id: i32,
    pub name: String,
    pub market_price: i32,
    pub avg_bazaar_price: i32,
    pub created_on: u64, // 拉取到的时间戳
}

pub fn productions_to_profit_infos(productions: Vec<ProductionItem>) -> Vec<ProfitInfo> {
    let mut res = Vec::new();
    for production in productions {
        for bazaar in production.cheapest_bazaars {
            let profit_info = ProfitInfo {
                player_id: bazaar.player_id,
                player_name: bazaar.player_name,
                quantity: bazaar.quantity,
                price: bazaar.price,
                total_value: bazaar.total_value.parse::<i32>().unwrap(),
                image: production.image.clone(),
                id: production.id,
                name: production.name.clone(),
                market_price: production.market_price,
                avg_bazaar_price: production.avg_bazaar_price,
                ..Default::default()
            };
            res.push(profit_info);
        }
    }
    res
}

pub fn get_bazaar_url(player_id: i32) -> String {
    format!("https://www.torn.com/bazaar.php?userId={}", player_id)
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// 最小利润
    pub min_profit: i32,
    /// 最小利润百分比
    pub min_profit_percentage: f32,
    /// 忽略的物品
    pub ignore_names: Vec<String>,
    /// 单个物品过滤条件
    pub filter_items: Vec<FilterItem>,
}

/// 单个物品过滤条件
#[derive(Debug, Clone, Default)]
pub struct FilterItem {
    pub id: i32,
    pub name: String,
    pub price: i32,
    pub profit_percentage: f32,
}

pub fn filter(favorites_response: FavoritesResponse, filter: Filter) -> Vec<ProfitInfo> {
    let mut items = Vec::new();
    for item in favorites_response.items {
        if filter.ignore_names.contains(&item.name) {
            continue;
        }
        for bazaar in item.cheapest_bazaars {
            let market_profit_percentage =
                ((item.market_price - bazaar.price) as f32 / item.market_price as f32) * 100.;
            let market_profit_total_value = (item.market_price - bazaar.price) * bazaar.quantity;

            let avg_bazaar_percentage =
                (item.avg_bazaar_price - bazaar.price) as f32 / item.avg_bazaar_price as f32 * 100.;
            let avg_bazaar_total_value = (item.avg_bazaar_price - bazaar.price) * bazaar.quantity;

            if let Some(filter_item) = filter
                .filter_items
                .iter()
                .find(|x| x.id == item.id || x.name == item.name)
            {
                if bazaar.price > filter_item.price {
                    continue;
                }
                if market_profit_percentage < filter_item.profit_percentage {
                    continue;
                }
            }
            let profit_percent = if market_profit_percentage > avg_bazaar_percentage {
                avg_bazaar_percentage
            } else {
                market_profit_percentage
            };
            let profit_single_value = if market_profit_percentage > avg_bazaar_percentage {
                item.avg_bazaar_price - bazaar.price
            } else {
                item.market_price - bazaar.price
            };
            let profit_total_value = if market_profit_total_value > avg_bazaar_total_value {
                avg_bazaar_total_value
            } else {
                market_profit_total_value
            };

            if profit_percent >= filter.min_profit_percentage
                && profit_total_value >= filter.min_profit
            {
                let Ok(total_value) = bazaar.total_value.parse::<i32>() else {
                    println!("Failed to parse total value: {}", bazaar.total_value);
                    continue;
                };
                let profit_info = ProfitInfo {
                    player_id: bazaar.player_id,
                    player_name: bazaar.player_name,
                    quantity: bazaar.quantity,
                    price: bazaar.price,
                    total_value,
                    image: item.image.clone(),
                    market_profit_percentage,
                    market_profit_single_value: item.market_price - bazaar.price,
                    market_profit_total_value,
                    avg_bazaar_profit_percentage: avg_bazaar_percentage,
                    avg_bazaar_profit_single_value: item.avg_bazaar_price - bazaar.price,
                    avg_bazaar_profit_total_value: avg_bazaar_total_value,
                    profit_percentage: profit_percent,
                    profit_single_value,
                    profit_total_value,

                    id: item.id,
                    name: item.name.clone(),
                    market_price: item.market_price,
                    avg_bazaar_price: item.avg_bazaar_price,
                    ..Default::default()
                };
                items.push(profit_info);
            }
        }
    }

    items.sort_by(|a, b| {
        b.market_profit_total_value
            .cmp(&a.market_profit_total_value)
    });

    items
}

pub struct SortProfitParams {
    pub recent_sec: u64,
}

/// 按利润排序，前 sec 秒的利润排序，然后是老的利润排序
pub fn sort_profit(
    params: SortProfitParams,
    items: Vec<ProfitUserInfo>,
) -> Vec<ProfitUserInfo> {
    let now =tools::time::get_current_time();
    let recent_sec = now - params.recent_sec;
    let mut recent_items: Vec<ProfitUserInfo> = items
        .clone()
        .into_iter()
        .filter(|x| x.created_on >= recent_sec)
        .collect();
    recent_items.sort_by(|a, b| b.profit_total_value.cmp(&a.profit_total_value));

    let mut old_items: Vec<ProfitUserInfo> = items
        .clone()
        .into_iter()
        .filter(|x| x.created_on < recent_sec)
        .collect();
    old_items.sort_by(|a, b| b.profit_total_value.cmp(&a.profit_total_value));

    recent_items.extend(old_items);
    recent_items
}

// 计算利润
pub fn calc_profit(now: u64, old: FavoritesRes, profit_items_new: Vec<ProfitInfo>) -> (FavoritesRes,bool) {
    let mut user_profit_result: Vec<ProfitUserInfo> = Vec::new();
    let mut has_new = false;

    // 统计数据
    for item in profit_items_new.iter() {
        if let Some(res) = user_profit_result
            .iter_mut()
            .find(|x| x.player_id == item.player_id)
        {
            res.items.push(item.clone());
        } else {
            has_new = true;
            user_profit_result.push(ProfitUserInfo {
                player_id: item.player_id,
                player_name: item.player_name.clone(),
                items: vec![item.clone()],
                created_on: now,
                ..Default::default()
            });
        }
    }

    // 计算单个用户总利润
    for res in user_profit_result.iter_mut() {
        res.total_value = res.items.iter().map(|x| x.total_value).sum();
        res.profit_total_value = res.items.iter().map(|x| x.market_profit_total_value).sum();
        res.profit_percentage = if res.total_value == 0 {
            0.0
        } else {
            res.profit_total_value as f32 / res.total_value as f32 * 100.0
        };
    }

    // 如果是老数据则赋予老的时间戳
    for item in old.profit_items_new.iter() {
        if let Some(res) = user_profit_result
            .iter_mut()
            .find(|x| x.player_id == item.player_id)
        {
            res.created_on = item.created_on;
        }
    }

    (FavoritesRes {
        profit_items_new: profit_items_new.clone(),
        profit_items_old: old.profit_items_new.clone(),
        user_profit_result,
    }, has_new)
}
