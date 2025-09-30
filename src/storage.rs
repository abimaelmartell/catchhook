use crate::models::StoredRequest;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::{
    path::Path as FsPath,
    sync::{atomic::AtomicU64, Arc},
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
