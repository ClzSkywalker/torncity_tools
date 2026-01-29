use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Network Request Failed, code: {0}, url: {1}")]
    NetworkCode(i64,String),
    #[error("Json Parse Failed, error: {0}")]
    JsonParse(String),
}
