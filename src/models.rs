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

#[derive(Serialize, Deserialize)]
pub struct LatestResponse {
    pub count: usize,
    pub items: Vec<StoredRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_request_serialization() {
        let request = StoredRequest {
            id: 1,
            ts_ms: 1234567890,
            method: "POST".to_string(),
            path: "/webhook".to_string(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: b"test data".to_vec(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: StoredRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.path, deserialized.path);
        assert_eq!(request.body, deserialized.body);
    }

    #[test]
    fn test_latest_response_creation() {
        let requests = vec![
            StoredRequest {
                id: 1,
                ts_ms: 1234567890,
                method: "GET".to_string(),
                path: "/test".to_string(),
                headers: vec![],
                body: vec![],
            },
            StoredRequest {
                id: 2,
                ts_ms: 1234567891,
                method: "POST".to_string(),
                path: "/webhook".to_string(),
                headers: vec![],
                body: vec![],
            },
        ];

        let response = LatestResponse {
            count: requests.len(),
            items: requests.clone(),
        };

        assert_eq!(response.count, 2);
        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].id, 1);
        assert_eq!(response.items[1].id, 2);
    }
}
