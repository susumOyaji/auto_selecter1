use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug)]
pub struct ScraperError(pub String);

impl std::fmt::Display for ScraperError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ScraperError {}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct StockData {
    pub code: String,
    pub name: String,
    pub price: String,
    pub change: String,
    pub change_percent: String,
    pub update_time: String,
}
