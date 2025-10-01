use crate::models::StoredRequest;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::{
    path::Path as FsPath,
    sync::{Arc, atomic::AtomicU64},
};

const REQS: TableDefinition<u64, &[u8]> = TableDefinition::new("reqs");

#[derive(Clone)]
pub struct Storage {
    db: Arc<Database>,
    next_id: Arc<AtomicU64>,
    max_reqs: usize,
}

impl Storage {
    pub fn new(data_dir: &FsPath, max_reqs: usize) -> anyhow::Result<Self> {
        let (db, last_id) = Self::open_db(data_dir)?;
        Ok(Self {
            db,
            next_id: Arc::new(AtomicU64::new(last_id)),
            max_reqs,
        })
    }

    pub fn next_id(&self) -> &Arc<AtomicU64> {
        &self.next_id
    }

    fn open_db(data_dir: &FsPath) -> anyhow::Result<(Arc<Database>, u64)> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("requests.redb");
        let db = if db_path.exists() {
            Database::open(db_path)?
        } else {
            Database::create(db_path)?
        };

        let mut last_id = 0;

        {
            let tx = db.begin_write()?;
            {
                let _table = tx.open_table(REQS)?;
            }
            tx.commit()?;
        }

        {
            let tx = db.begin_read()?;
            let table = tx.open_table(REQS)?;
            if let Some(Ok((k, _))) = table.range(0..)?.last() {
                last_id = k.value();
            }
        }

        Ok((Arc::new(db), last_id))
    }

    pub fn insert(&self, req: &StoredRequest) -> anyhow::Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(REQS)?;
            let blob = serde_json::to_vec(req)?;
            table.insert(req.id, blob.as_slice())?;
        }
        tx.commit()?;

        let tx_prune = self.db.begin_write()?;
        {
            let mut table = tx_prune.open_table(REQS)?;
            let len: u64 = table.len()?;
            if (len as usize) > self.max_reqs {
                let key_to_remove = {
                    let mut it = table.range(0..)?;
                    it.next()
                        .and_then(|result| result.ok())
                        .map(|(k, _)| k.value())
                };
                if let Some(key) = key_to_remove {
                    table.remove(key)?;
                }
            }
        }
        tx_prune.commit()?;
        Ok(())
    }

    pub fn latest(&self, limit: usize) -> anyhow::Result<Vec<StoredRequest>> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(REQS)?;
        let out = table
            .range(0..)?
            .rev()
            .take(limit)
            .filter_map(|row| {
                row.ok()
                    .and_then(|(_, v)| serde_json::from_slice::<StoredRequest>(v.value()).ok())
            })
            .collect();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use tempfile::TempDir;

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path(), 100).unwrap();
        (storage, temp_dir)
    }

    fn create_test_request(id: u64) -> StoredRequest {
        StoredRequest {
            id,
            ts_ms: 1234567890 + id as i64,
            method: "POST".to_string(),
            path: format!("/webhook/{}", id),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: format!("{{\"test\": {}}}", id).into_bytes(),
        }
    }

    #[test]
    fn test_storage_creation() {
        let (storage, _temp_dir) = create_test_storage();
        assert_eq!(storage.next_id().load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_insert_and_retrieve() {
        let (storage, _temp_dir) = create_test_storage();
        let request = create_test_request(1);

        storage.insert(&request).unwrap();

        let latest = storage.latest(10).unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].id, 1);
        assert_eq!(latest[0].method, "POST");
        assert_eq!(latest[0].path, "/webhook/1");
    }

    #[test]
    fn test_multiple_inserts() {
        let (storage, _temp_dir) = create_test_storage();

        for i in 1..=5 {
            let request = create_test_request(i);
            storage.insert(&request).unwrap();
        }

        let latest = storage.latest(10).unwrap();
        assert_eq!(latest.len(), 5);

        assert_eq!(latest[0].id, 5);
        assert_eq!(latest[4].id, 1);
    }

    #[test]
    fn test_pruning_when_exceeding_max_reqs() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path(), 3).unwrap();

        for i in 1..=5 {
            let request = create_test_request(i);
            storage.insert(&request).unwrap();
        }

        let latest = storage.latest(10).unwrap();
        assert!(latest.len() <= 3);

        let ids: Vec<u64> = latest.iter().map(|r| r.id).collect();
        assert!(ids.contains(&5));
        assert!(ids.contains(&4));
        assert!(ids.contains(&3));
    }

    #[test]
    fn test_empty_storage() {
        let (storage, _temp_dir) = create_test_storage();
        let latest = storage.latest(10).unwrap();
        assert_eq!(latest.len(), 0);
    }

    #[test]
    fn test_limit_functionality() {
        let (storage, _temp_dir) = create_test_storage();

        for i in 1..=10 {
            let request = create_test_request(i);
            storage.insert(&request).unwrap();
        }

        let latest = storage.latest(5).unwrap();
        assert_eq!(latest.len(), 5);

        let ids: Vec<u64> = latest.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![10, 9, 8, 7, 6]);
    }
}
