use model::weav3r::favorites::ProductionItem;

#[derive(Debug, Clone, Default)]
pub struct FavoritesRes {
    pub filter: Filter,
    pub sort: SortProfitParams,
    pub profit_items_new: Vec<ProfitInfo>,
    pub profit_items_old: Vec<ProfitInfo>,
    pub user_profit_result: Vec<ProfitUserInfo>,
    /// 是否有新增用户
    pub has_new: bool,
}

impl FavoritesRes {
    pub fn set_new_profit(&mut self, productions: Vec<ProductionItem>) {
        let profit_items: Vec<ProfitInfo> = productions
            .iter()
            .flat_map(|x| product_to_profit_info(x.clone()))
            .collect();
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

    /// 过滤利润信息
    pub fn filter(data: Vec<ProfitInfo>, filter: Filter) -> Vec<ProfitInfo> {
        let mut items = Vec::new();
        for item in data.iter() {
            if filter.ignore_names.contains(&item.name) {
                continue;
            }

            if let Some(filter_item) = filter
                .filter_items
                .iter()
                .find(|x| x.id == item.id || x.name == item.name)
                && item.price > filter_item.price
            {
                continue;
            }

            if item.profit_percentage >= filter.min_profit_percentage
                && item.profit_total_value >= filter.min_profit
            {
                items.push(item.clone());
            }
        }

        items
    }

    // 计算用户维度利润
    pub fn calc_user_profit(profit_items_new: Vec<ProfitInfo>) -> Vec<ProfitUserInfo> {
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
                .sort_by(|a, b| b.profit_total_value.cmp(&a.profit_total_value))
        });

        // 计算单个用户总利润
        for res in user_profit_result.iter_mut() {
            res.total_value = res.items.iter().map(|x| x.total_value).sum::<u64>();
            res.profit_total_value = res
                .items
                .iter()
                .map(|x| {
                    if x.market_profit_total_value > x.avg_bazaar_profit_total_value {
                        x.avg_bazaar_profit_total_value
                    } else {
                        x.market_profit_total_value
                    }
                })
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
    pub fn diff_user_profit(
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
    pub fn sort_profit(
        params: SortProfitParams,
        items: Vec<ProfitUserInfo>,
    ) -> Vec<ProfitUserInfo> {
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
    pub price: i32,
    pub total_value: u64,
    pub image: String,
    pub market_profit_percentage: f32,
    pub market_profit_single_value: i32,
    pub market_profit_total_value: i64,
    pub avg_bazaar_profit_percentage: f32,
    pub avg_bazaar_profit_single_value: i32,
    pub avg_bazaar_profit_total_value: i64,
    // 按照最低的数据进行复制
    pub profit_percentage: f32,
    pub profit_single_value: i32,
    pub profit_total_value: i64,

    pub id: i32,
    pub name: String,
    pub market_price: i32,
    pub avg_bazaar_price: i32,
    pub created_on: u64, // 拉取到的时间戳
}

pub type ProfitInfoVec = Vec<ProfitInfo>;

fn product_to_profit_info(product: ProductionItem) -> Vec<ProfitInfo> {
    let mut res = Vec::new();
    for user_bazaar in product.cheapest_bazaars.iter() {
        let market_profit_percentage = ((product.market_price - user_bazaar.price) as f32
            / product.market_price as f32)
            * 100.;
        let market_profit_total_value =
            ((product.market_price - user_bazaar.price) as i64) * user_bazaar.quantity as i64;

        let avg_bazaar_percentage = (product.avg_bazaar_price - user_bazaar.price) as f32
            / product.avg_bazaar_price as f32
            * 100.;
        let avg_bazaar_total_value =
            ((product.avg_bazaar_price - user_bazaar.price) as i64) * user_bazaar.quantity as i64;

        let profit_percent = if market_profit_percentage > avg_bazaar_percentage {
            avg_bazaar_percentage
        } else {
            market_profit_percentage
        };
        let profit_single_value = if market_profit_percentage > avg_bazaar_percentage {
            product.avg_bazaar_price - user_bazaar.price
        } else {
            product.market_price - user_bazaar.price
        };
        let profit_total_value = if market_profit_total_value > avg_bazaar_total_value {
            avg_bazaar_total_value
        } else {
            market_profit_total_value
        };

        let Ok(total_value) = user_bazaar.total_value.parse::<u64>() else {
            eprintln!(
                "Failed to parse total value: \'{}\'",
                user_bazaar.total_value
            );
            continue;
        };
        let profit_info = ProfitInfo {
            player_id: user_bazaar.player_id,
            player_name: user_bazaar.player_name.clone(),
            quantity: user_bazaar.quantity,
            price: user_bazaar.price,
            total_value,
            image: product.image.clone(),
            market_profit_percentage,
            market_profit_single_value: product.market_price - user_bazaar.price,
            market_profit_total_value,
            avg_bazaar_profit_percentage: avg_bazaar_percentage,
            avg_bazaar_profit_single_value: product.avg_bazaar_price - user_bazaar.price,
            avg_bazaar_profit_total_value: avg_bazaar_total_value,
            profit_percentage: profit_percent,
            profit_single_value,
            profit_total_value,

            id: product.id,
            name: product.name.clone(),
            market_price: product.market_price,
            avg_bazaar_price: product.avg_bazaar_price,
            ..Default::default()
        };
        res.push(profit_info);
    }
    res
}

pub fn get_bazaar_url(player_id: i32) -> String {
    format!("https://www.torn.com/bazaar.php?userId={}", player_id)
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// 最小利润
    pub min_profit: i64,
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

#[derive(Debug, Clone, Default)]
pub struct SortProfitParams {
    pub recent_sec: u64,
}
