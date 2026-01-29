use std::{collections::HashMap, sync::OnceLock};

use model::user_log::UserLogEntry;

pub const TORN_LOG_CATEGORY_TRADE: u32 = 94;
// 有金钱结果
pub const TORN_LOG_TRADE_ADD: &str = "Trade money add";
pub const TORN_LOG_TRADE_REMOVE: &str = "Trade money remove";
// 没有金钱结果
pub const TORN_LOG_TRADE_INIT: &str = "Trade initiate outgoing";
pub const TORN_LOG_TRADE_EXPIRE: &str = "Trade expire";
pub const TORN_LOG_TRADE_CANCEL_INCOME: &str = "Trade cancel incoming";
pub const TORN_LOG_TRADE_CANCEL_OUTGOING: &str = "Trade cancel outgoing";

pub static TORN_LOG_TRADE_MAP: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();

pub fn torn_log_trade_map() -> &'static HashMap<u32, &'static str> {
    TORN_LOG_TRADE_MAP.get_or_init(|| {
        HashMap::from([
            (4400, TORN_LOG_TRADE_INIT),
            (4410, TORN_LOG_TRADE_CANCEL_INCOME),
            (4442, TORN_LOG_TRADE_ADD),
            (4443, TORN_LOG_TRADE_REMOVE),
            (4420, TORN_LOG_TRADE_EXPIRE),
        ])
    })
}

pub fn get_torn_coin_value(logs: &[UserLogEntry]) -> Option<u64> {
    let id_map = torn_log_trade_map();
    for log in logs.iter() {
        if let Some(name) = id_map.get(&log.details.id) {
            match *name {
                TORN_LOG_TRADE_ADD | TORN_LOG_TRADE_REMOVE => match log.data.total {
                    Some(r) => return Some(r),
                    None => return None,
                },
                TORN_LOG_TRADE_CANCEL_INCOME
                | TORN_LOG_TRADE_CANCEL_OUTGOING
                | TORN_LOG_TRADE_EXPIRE => return None,
                _ => continue,
            }
        }
    }
    None
}
