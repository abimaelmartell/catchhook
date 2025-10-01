use catchhook::models::StoredRequest;
use catchhook::storage::Storage;
use tempfile::TempDir;

#[tokio::test]
async fn test_storage_integration() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path(), 5).unwrap();

    assert_eq!(storage.latest(10).unwrap().len(), 0);

    let request = StoredRequest {
        id: 1,
        ts_ms: 1234567890,
        method: "POST".to_string(),
        path: "/webhook".to_string(),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
        body: b"test data".to_vec(),
    };

    storage.insert(&request).unwrap();

    let latest = storage.latest(10).unwrap();
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].id, 1);
    assert_eq!(latest[0].method, "POST");
}
