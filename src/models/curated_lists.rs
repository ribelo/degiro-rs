use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CuratedLists {
    pub last_updated: DateTime<Utc>,
    pub most_traded_weekly: Vec<u64>,
    pub most_traded_daily: Vec<u64>,
    pub largest_world_etfs: Vec<u64>,
    pub most_held: Vec<u64>,
}
