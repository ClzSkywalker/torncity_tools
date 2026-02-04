use std::collections::HashMap;

use godot::global::godot_error;
use model::{items::ItemInfo, weav3r::favorites::ProductionItem};

#[derive(Debug, Clone, Default)]
pub struct FavoritesData {
    pub filter: Filter,
    pub sort: SortProfitParams,
    pub profit_items_new: Vec<ProfitInfo>,
    pub profit_items_old: Vec<ProfitInfo>,
    pub user_profit_result: Vec<ProfitUserInfo>,
    /// 是否有新增用户
    pub has_new: bool,
}

impl FavoritesData {
    pub fn set_new_profit(&mut self, productions: Vec<ProductionItem>) {
        let profit_items: Vec<ProfitInfo> = productions
            .iter()
            .flat_map(|x| self.product_to_profit_info(x.clone()))
            .collect();
        let profit_items = Self::combine(profit_items);
        let profit_items = Self::filter(profit_items, self.filter.clone());
        let user_profit_result = Self::calc_user_profit(profit_items.clone());
        let (user_profit_result, has_new) =
            Self::diff_user_profit(&self.user_profit_result, user_profit_result);
        let user_profit_result = Self::sort_profit(self.sort.clone(), user_profit_result);

        self.profit_items_old = self.profit_items_new.clone();
        self.profit_items_new = profit_items;
        self.user_profit_result = user_profit_result;
        self.has_new = has_new;
    }

    fn product_to_profit_info(&self, product: ProductionItem) -> Vec<ProfitInfo> {
        let mut res = Vec::new();

        let Some(market_price) = product.market_price else {
            return res;
        };
        if market_price == 0 {
            return res;
        }

        let Some(avg_bazaar_price) = product.avg_bazaar_price else {
            return res;
        };
        if avg_bazaar_price == 0 {
            return res;
        }

        let in_target_ids = self.filter.target_ids.contains(&product.id);

        let office_sell_price = if !in_target_ids {
            self.filter
                .office_item_map
                .get(&product.id)
                .map(|item| item.sell_price as i64)
        } else {
            None
        };

        for user_bazaar in product.cheapest_bazaars.iter() {
            let Ok(total_value) = user_bazaar.total_value.parse::<u64>() else {
                eprintln!(
                    "Failed to parse total value: \'{}\'",
                    user_bazaar.total_value
                );
                continue;
            };

            let user_price = user_bazaar.price;

            let Some(selected) = compute_profit(
                in_target_ids,
                user_price,
                user_bazaar.quantity,
                market_price,
                avg_bazaar_price,
                office_sell_price,
            ) else {
                continue;
            };

            let final_profit = if selected.market.percentage <= selected.bazaar.percentage {
                selected.market.clone()
            } else {
                selected.bazaar.clone()
            };

            let profit_info = ProfitInfo {
                player_id: user_bazaar.player_id,
                player_name: user_bazaar.player_name.clone(),
                quantity: user_bazaar.quantity,
                price: user_bazaar.price as u64,
                total_value,
                image: product.image.clone(),
                market_profit: selected.market,
                avg_bazaar_profit: selected.bazaar,
                final_profit,
                id: product.id,
                name: product.name.clone(),
                market_price: market_price as u64,
                avg_bazaar_price: avg_bazaar_price as u64,
                is_office: selected.used_office,
                ..Default::default()
            };
            res.push(profit_info);
        }
        res
    }

    /// 将相同用户下相同商品的 id 组合在一起，如果价格不一致，用平均值处理
    /// 部分商品是拆开售卖的，这边利润计算进行统一处理
    fn combine(data: Vec<ProfitInfo>) -> Vec<ProfitInfo> {
        let mut data_map: HashMap<(i32, i32), ProfitInfo> = HashMap::new();
        for ele in data.iter() {
            match data_map.get(&(ele.player_id, ele.id)) {
                Some(r) => {
                    let r = r.combine_other(ele);
                    data_map.insert((ele.player_id, ele.id), r);
                }
                None => {
                    data_map.insert((ele.player_id, ele.id), ele.clone());
                }
            }
        }
        data_map.into_values().collect()
    }

    /// 用户维度，过滤利润信息
    fn filter(data: Vec<ProfitInfo>, filter: Filter) -> Vec<ProfitInfo> {
        let mut items = Vec::new();
        for item in data.iter() {
            if filter.ignore_names.contains(&item.name) {
                continue;
            }

            let in_target = filter.target_ids.contains(&item.id);

            // 官方售卖价格过滤
            if !in_target
                && let Some(office_item) = filter.office_item_map.get(&item.id)
                && (!office_item.tradeable || item.price > office_item.sell_price)
            {
                continue;
            }

            if item.final_profit.percentage >= filter.min_profit_percentage
                && item.final_profit.total_value >= filter.min_profit
            {
                items.push(item.clone());
            }
        }

        items
    }

    // 计算用户维度利润
    fn calc_user_profit(profit_items_new: Vec<ProfitInfo>) -> Vec<ProfitUserInfo> {
        let mut user_profit_result: Vec<ProfitUserInfo> = Vec::new();

        // 统计数据
        for item in profit_items_new.iter() {
            if let Some(res) = user_profit_result
                .iter_mut()
                .find(|x| x.player_id == item.player_id)
            {
                res.items.push(item.clone());
            } else {
                user_profit_result.push(ProfitUserInfo {
                    player_id: item.player_id,
                    player_name: item.player_name.clone(),
                    items: vec![item.clone()],
                    ..Default::default()
                });
            }
        }

        user_profit_result.iter_mut().for_each(|item| {
            item.items
                .sort_by(|a, b| b.final_profit.total_value.cmp(&a.final_profit.total_value))
        });

        // 计算单个用户总利润
        for res in user_profit_result.iter_mut() {
            res.total_value = res.items.iter().map(|x| x.total_value).sum::<u64>();
            res.profit_total_value = res
                .items
                .iter()
                .map(|x| x.final_profit.total_value)
                .sum::<i64>();
            res.profit_percentage = if res.total_value == 0 {
                0.0
            } else {
                res.profit_total_value as f32 / res.total_value as f32 * 100.0
            };
        }

        user_profit_result
    }

    /// 新老数据比较，给新的用户增加时间戳，老用户不改变时间戳，返回是否有新增用户
    fn diff_user_profit(
        old: &[ProfitUserInfo],
        mut new: Vec<ProfitUserInfo>,
    ) -> (Vec<ProfitUserInfo>, bool) {
        let now = tools::time::get_current_time();
        let mut has_new = false;
        for ele in new.iter_mut() {
            if let Some(res) = old.iter().find(|x| x.player_id == ele.player_id) {
                ele.created_on = res.created_on;
            } else {
                ele.created_on = now;
                has_new = true;
            }
        }
        (new, has_new)
    }

    /// 按利润排序，前 sec 秒的利润排序，然后是老的利润排序
    fn sort_profit(params: SortProfitParams, items: Vec<ProfitUserInfo>) -> Vec<ProfitUserInfo> {
        let now = tools::time::get_current_time();
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
}

/// 用户维度 利润信息
#[derive(Debug, Clone, Default)]
pub struct ProfitUserInfo {
    pub player_id: i32,
    pub player_name: String,
    pub total_value: u64,
    pub profit_total_value: i64,
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
    pub price: u64,
    pub total_value: u64,
    pub image: String,
    pub market_profit: ProfitMetrics,
    pub avg_bazaar_profit: ProfitMetrics,
    // 按照最低的数据进行复制
    pub final_profit: ProfitMetrics,

    pub id: i32,
    pub name: String,
    pub market_price: u64,
    pub avg_bazaar_price: u64,
    pub created_on: u64, // 拉取到的时间戳

    /// 是否在官方售卖
    pub is_office: bool,
}

impl ProfitInfo {
    /// 计算单个的利润
    fn single_value(&self) -> ProfitInfo {
        let mut cp = self.clone();
        cp.market_profit = cp.market_profit.single_value(self.quantity);
        cp.avg_bazaar_profit = cp.avg_bazaar_profit.single_value(self.quantity);
        cp.final_profit = cp.final_profit.single_value(self.quantity);
        cp
    }

    /// 组合相同的类型
    fn combine_other(&self, data: &ProfitInfo) -> ProfitInfo {
        if self.id != data.id {
            godot_error!("ProfitInfo id not equal:{}:{}", self.id, data.id);
            return self.clone();
        }
        let total_quantity = self.quantity + data.quantity;
        let e = data.single_value();
        let mut res = self.clone();
        res.market_profit = res.market_profit.combine(e.market_profit);
        res.avg_bazaar_profit = res.avg_bazaar_profit.combine(e.avg_bazaar_profit);
        res.final_profit = res.final_profit.combine(e.final_profit);
        res.market_profit.total_value *= total_quantity as i64;
        res.avg_bazaar_profit.total_value *= total_quantity as i64;
        res.final_profit.total_value *= total_quantity as i64;
        res.quantity = total_quantity;
        res
    }
}

/// 利润计算结果
#[derive(Debug, Clone, Default)]
pub struct ProfitMetrics {
    pub percentage: f32,
    pub single_value: i64,
    pub total_value: i64,
}

impl ProfitMetrics {
    fn single_value(&self, quantity: i32) -> Self {
        Self {
            percentage: self.percentage,
            single_value: self.single_value,
            total_value: self.total_value / quantity as i64,
        }
    }

    fn combine(&self, data: Self) -> Self {
        Self {
            percentage: (self.percentage + data.percentage) / 2.,
            single_value: (self.single_value + data.single_value) / 2,
            total_value: (self.total_value + data.total_value) / 2,
        }
    }
}

/// 选中的利润计算结果
#[derive(Debug, Clone)]
struct SelectedProfit {
    _profit: ProfitMetrics,
    market: ProfitMetrics,
    bazaar: ProfitMetrics,
    used_office: bool,
}

/// 统一的利润计算函数
/// - 在 target_ids 中：使用 market_price 和 avg_bazaar_price 计算利润，取较低的百分比
/// - 不在 target_ids 中：使用官方价格计算利润，如果没有官方价格则返回 None
fn compute_profit(
    in_target_ids: bool,
    user_price: i64,
    quantity: i32,
    market_price: i64,
    avg_bazaar_price: i64,
    office_sell_price: Option<i64>,
) -> Option<SelectedProfit> {
    let q = quantity as i64;

    let market_diff = market_price - user_price;
    let market = ProfitMetrics {
        percentage: market_diff as f32 / market_price as f32 * 100.0,
        single_value: market_diff,
        total_value: market_diff * q,
    };

    let bazaar_diff = avg_bazaar_price - user_price;
    let bazaar = ProfitMetrics {
        percentage: bazaar_diff as f32 / avg_bazaar_price as f32 * 100.0,
        single_value: bazaar_diff,
        total_value: bazaar_diff * q,
    };

    if in_target_ids {
        let pick_market = market.percentage <= bazaar.percentage;
        let chosen = if pick_market { &market } else { &bazaar };
        Some(SelectedProfit {
            _profit: ProfitMetrics {
                percentage: chosen.percentage,
                single_value: chosen.single_value,
                total_value: chosen.total_value,
            },
            market,
            bazaar,
            used_office: false,
        })
    } else {
        let ref_price = office_sell_price?;
        let diff = ref_price - user_price;
        let pct = if ref_price > 0 {
            diff as f32 / ref_price as f32 * 100.0
        } else {
            0.0
        };
        Some(SelectedProfit {
            _profit: ProfitMetrics {
                percentage: pct,
                single_value: diff,
                total_value: diff * q,
            },
            market,
            bazaar,
            used_office: true,
        })
    }
}

pub fn get_bazaar_url(player_id: i32) -> String {
    format!("https://www.torn.com/bazaar.php?userId={}", player_id)
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// 用户自定义目标 id
    pub target_ids: Vec<i32>,
    /// 最小利润
    pub min_profit: i64,
    /// 最小利润百分比
    pub min_profit_percentage: f32,
    /// 忽略的物品
    pub ignore_names: Vec<String>,
    /// 单个物品过滤条件
    pub filter_items: Vec<FilterItem>,
    /// 官方最低售卖价格，低于这个价格的物品不走官方售卖逻辑
    pub office_sell_price: u64,
    /// 官方售卖价格列表
    pub office_item_map: HashMap<i32, ItemInfo>,
    /// 武器物品 id map
    pub weapon_item_map: HashMap<i32, ItemInfo>,
}

/// 单个物品过滤条件
#[derive(Debug, Clone, Default)]
pub struct FilterItem {
    pub id: i32,
    pub name: String,
    pub price: u64,
    pub profit_percentage: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SortProfitParams {
    pub recent_sec: u64,
}
