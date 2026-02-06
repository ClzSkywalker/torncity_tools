use std::collections::HashMap;

use godot::global::godot_error;
use model::{items::ItemInfo, weav3r::favorites::ProductionItem};
use tools::order_change::{ContentHashable, ContentHash, hash::StableHasher};

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
            let user_price = user_bazaar.price;

            let selected = compute_profit(
                in_target_ids,
                user_price as u64,
                user_bazaar.quantity,
                market_price,
                avg_bazaar_price,
                office_sell_price,
            );
            let Some(final_profit) = selected.final_profit else {
                continue;
            };

            let profit_info = ProfitInfo {
                player_id: user_bazaar.player_id,
                player_name: user_bazaar.player_name.clone(),
                quantity: user_bazaar.quantity,
                single_recyle_price: user_bazaar.price as u64,
                image: product.image.clone(),
                market_profit: selected.market,
                avg_bazaar_profit: selected.bazaar,
                office_profit: selected.office,
                final_profit,
                id: product.id,
                name: product.name.clone(),
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
            if !in_target {
                if let Some(office_item) = filter.office_item_map.get(&item.id)
                    && (!office_item.tradeable
                        || item.single_recyle_price > office_item.sell_price
                        || office_item.sell_price < filter.office_sell_price
                        || item.final_profit.total_profit_value < filter.office_sell_profit as i64)
                {
                    continue;
                }
                items.push(item.clone());
                continue;
            }

            if item.final_profit.percentage >= filter.min_profit_percentage
                && item.final_profit.total_profit_value >= filter.min_profit
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
            item.items.sort_by(|a, b| {
                b.final_profit
                    .total_profit_value
                    .cmp(&a.final_profit.total_profit_value)
            })
        });

        // 计算单个用户总利润
        for res in user_profit_result.iter_mut() {
            res.total_recyle_price = res
                .items
                .iter()
                .map(|x| x.total_recyle_price())
                .sum::<u64>();
            res.total_profit_price = res
                .items
                .iter()
                .map(|x| x.final_profit.total_profit_value)
                .sum::<i64>();
            res.profit_percentage = if res.total_recyle_price == 0 {
                0.0
            } else {
                res.total_profit_price as f32 / res.total_recyle_price as f32 * 100.0
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
        recent_items.sort_by(|a, b| b.total_profit_price.cmp(&a.total_profit_price));

        let mut old_items: Vec<ProfitUserInfo> = items
            .clone()
            .into_iter()
            .filter(|x| x.created_on < recent_sec)
            .collect();
        old_items.sort_by(|a, b| b.total_profit_price.cmp(&a.total_profit_price));

        // 子项中按利润排序
        recent_items.extend(old_items);
        recent_items.iter_mut().for_each(|x| {
            x.items.sort_by(|a, b| {
                b.final_profit
                    .total_profit_value
                    .cmp(&a.final_profit.total_profit_value)
            })
        });

        recent_items
    }
}

/// 用户维度 利润信息
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProfitUserInfo {
    pub player_id: i32,
    pub player_name: String,
    pub total_recyle_price: u64,
    pub total_profit_price: i64,
    pub profit_percentage: f32,
    pub created_on: u64, // 拉取到的时间戳
    pub items: Vec<ProfitInfo>,
}

/// 商品维度 利润信息
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProfitInfo {
    pub player_id: i32,
    pub player_name: String,
    pub quantity: i32,
    /// 单个回收价格
    pub single_recyle_price: u64,
    pub image: String,
    /// 市场利润
    pub market_profit: ProfitMetrics,
    /// 平均 bazaar 利润
    pub avg_bazaar_profit: ProfitMetrics,
    /// 官方利润
    pub office_profit: Option<ProfitMetrics>,
    // 按照最低的数据进行复制
    pub final_profit: ProfitMetrics,

    pub id: i32,
    pub name: String,
    pub created_on: u64, // 拉取到的时间戳
}

impl ProfitInfo {
    /// 商品回收总价格
    pub fn total_recyle_price(&self) -> u64 {
        self.quantity as u64 * self.single_recyle_price
    }

    /// 商品最终总售价
    pub fn total_sell_price(&self) -> u64 {
        self.quantity as u64 * self.final_profit.single_sell_price
    }
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

        res.market_profit = res.market_profit.build(total_quantity);
        res.avg_bazaar_profit = res.avg_bazaar_profit.build(total_quantity);
        res.final_profit = res.final_profit.build(total_quantity);

        res.quantity = total_quantity;
        res.single_recyle_price = (res.single_recyle_price * self.quantity as u64
            + data.single_recyle_price * data.quantity as u64)
            / (self.quantity as u64 + data.quantity as u64);
        res
    }
}

/// 利润计算结果
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProfitMetrics {
    /// 利润百分比
    pub percentage: f32,
    /// 单个利润
    pub single_profit_value: i64,
    /// 总利润
    pub total_profit_value: i64,
    /// 二次出售
    /// 单个售价
    pub single_sell_price: u64,
    /// 总售价
    pub total_sell_price: u64,
    /// 是否在官方售卖
    pub is_office: bool,
}

impl ProfitMetrics {
    fn single_value(&self, quantity: i32) -> Self {
        Self {
            percentage: self.percentage,
            single_profit_value: self.single_profit_value,
            total_profit_value: self.total_profit_value / quantity as i64,
            single_sell_price: self.single_sell_price,
            total_sell_price: self.total_sell_price / quantity as u64,
            is_office: self.is_office,
        }
    }

    fn combine(&self, data: Self) -> Self {
        Self {
            percentage: (self.percentage + data.percentage) / 2.,
            single_profit_value: (self.single_profit_value + data.single_profit_value) / 2,
            total_profit_value: (self.total_profit_value + data.total_profit_value) / 2,
            single_sell_price: (self.single_sell_price + data.single_sell_price) / 2,
            total_sell_price: (self.total_sell_price + data.total_sell_price) / 2,
            is_office: self.is_office,
        }
    }

    fn build(&self, quantity: i32) -> Self {
        Self {
            percentage: self.percentage,
            single_profit_value: self.single_profit_value,
            total_profit_value: self.total_profit_value * quantity as i64,
            single_sell_price: self.single_sell_price,
            total_sell_price: self.total_sell_price * quantity as u64,
            is_office: self.is_office,
        }
    }
}

/// 选中的利润计算结果
#[derive(Debug, Clone)]
struct SelectedProfit {
    final_profit: Option<ProfitMetrics>,
    market: ProfitMetrics,
    bazaar: ProfitMetrics,
    office: Option<ProfitMetrics>,
}

/// 统一的利润计算函数
/// - 在 target_ids 中：使用 market_price 和 avg_bazaar_price 计算利润，取较低的百分比
/// - 不在 target_ids 中：使用官方价格计算利润，如果没有官方价格则返回 None
fn compute_profit(
    in_target_ids: bool,
    user_price: u64,
    quantity: i32,
    market_price: i64,
    avg_bazaar_price: i64,
    office_sell_price: Option<i64>,
) -> SelectedProfit {
    let q = quantity as i64;

    let market_diff = market_price - user_price as i64;
    let market = ProfitMetrics {
        percentage: market_diff as f32 / market_price as f32 * 100.0,
        single_profit_value: market_diff,
        total_profit_value: market_diff * q,
        single_sell_price: market_price as u64,
        total_sell_price: market_price as u64 * q as u64,
        is_office: false,
    };

    let bazaar_diff = avg_bazaar_price - user_price as i64;
    let bazaar = ProfitMetrics {
        percentage: bazaar_diff as f32 / avg_bazaar_price as f32 * 100.0,
        single_profit_value: bazaar_diff,
        total_profit_value: bazaar_diff * q,
        single_sell_price: avg_bazaar_price as u64,
        total_sell_price: avg_bazaar_price as u64 * q as u64,
        is_office: false,
    };

    let office = office_sell_price.map(|office_price| ProfitMetrics {
        percentage: (office_price - user_price as i64) as f32 / office_price as f32 * 100.0,
        single_profit_value: office_price - user_price as i64,
        total_profit_value: (office_price - user_price as i64) * q,
        single_sell_price: office_price as u64,
        total_sell_price: office_price as u64 * q as u64,
        is_office: true,
    });

    let mut pick_market = if market.percentage <= bazaar.percentage {
        market.clone()
    } else {
        bazaar.clone()
    };

    if in_target_ids {
        if let Some(office_profit) = &office
            && office_profit.percentage >= pick_market.percentage
        {
            pick_market = office_profit.clone()
        }
        SelectedProfit {
            final_profit: Some(pick_market),
            market,
            bazaar,
            office,
        }
    } else {
        let Some(office_profit) = office.clone() else {
            return SelectedProfit {
                final_profit: None,
                market,
                bazaar,
                office,
            };
        };
        SelectedProfit {
            final_profit: Some(office_profit),
            market,
            bazaar,
            office,
        }
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
    /// 官方售卖利润阀值，低于这个利润的物品不走官方售卖逻辑
    pub office_sell_profit: u64,
    /// 官方售卖价格列表
    pub office_item_map: HashMap<i32, ItemInfo>,
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

impl ContentHashable for ProfitInfo {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_i32(self.id);
        hasher.write_str(&self.name);
        hasher.write_i32(self.quantity);
        hasher.write_u64(self.single_recyle_price);
        hasher.write_str(&self.image);
        hasher.write_u64(self.created_on);
        
        let final_profit_hash = self.final_profit.content_hash().0;
        hasher.write_u64(final_profit_hash);
        
        hasher.finish()
    }
}

impl ContentHashable for ProfitMetrics {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_f32(self.percentage);
        hasher.write_i64(self.single_profit_value);
        hasher.write_i64(self.total_profit_value);
        hasher.write_u64(self.single_sell_price);
        hasher.write_u64(self.total_sell_price);
        hasher.write_u64(if self.is_office { 1 } else { 0 });
        hasher.finish()
    }
}

impl ContentHashable for ProfitUserInfo {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_i32(self.player_id);
        hasher.write_str(&self.player_name);
        hasher.write_u64(self.total_recyle_price);
        hasher.write_i64(self.total_profit_price);
        hasher.write_f32(self.profit_percentage);
        hasher.write_u64(self.created_on);
        
        let items_hash = self.items.content_hash().0;
        hasher.write_u64(items_hash);
        
        hasher.finish()
    }
}
