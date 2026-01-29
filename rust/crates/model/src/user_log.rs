use serde::Deserialize;
use serde_json::Value;

use crate::error::MyError;

pub struct UserLogReq {
    pub target: Option<u32>,
    pub limit: u32,
    // 分类
    pub cat: Option<u32>,
    pub key: String,
}

impl Default for UserLogReq {
    fn default() -> Self {
        Self {
            target: None,
            limit: 10,
            cat: None,
            key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserLogResponse {
    pub log: Vec<UserLogEntry>,
}

impl UserLogResponse {
    pub fn from_json(json: &str) -> Result<Self, MyError> {
        serde_json::from_str(json).map_err(|e| MyError::JsonParse(e.to_string()))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserLogEntry {
    pub id: String,
    pub timestamp: i64,
    pub details: UserLogDetails,
    pub data: UserLogData,
    pub params: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserLogDetails {
    pub id: u32,
    pub title: String,
    pub category: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserLogData {
    pub user: i64,
    pub money: Option<u64>,
    pub total: Option<u64>,
    pub description: Option<String>,
}
