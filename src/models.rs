use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredRequest {
    pub id: u64,
    pub ts_ms: i64,
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Serialize)]
pub struct LatestResponse {
    pub count: usize,
    pub items: Vec<StoredRequest>,
}
