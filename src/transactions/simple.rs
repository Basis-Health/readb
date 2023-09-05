use anyhow::bail;
use std::collections::HashMap;

use crate::databases::db_trait::DatabaseTransactionsIO;
use crate::transactions::{Transaction, TransactionState};
use crate::IndexTable;

pub(crate) struct SimpleTransaction<'a, D>
where
    D: DatabaseTransactionsIO,
{
    database: &'a mut D,
    index_table_snapshot: Box<dyn IndexTable>,

    new_entries: HashMap<String, Vec<u8>>,

    state: TransactionState,
}

impl<'a, D> SimpleTransaction<'a, D>
where
    D: DatabaseTransactionsIO,
{
    pub(crate) fn new(database: &'a mut D) -> Self {
        let index_table_snapshot = database.snapshot();
        Self {
            database,
            index_table_snapshot,
            new_entries: HashMap::new(),
            state: TransactionState::Unknown,
        }
    }
}

impl<'a, D> Transaction for SimpleTransaction<'a, D>
where
    D: DatabaseTransactionsIO,
{
    fn put(&mut self, key: &str, value: &[u8]) -> anyhow::Result<()> {
        if self.state != TransactionState::Unknown {
            bail!("Transaction already {:?}", self.state);
        }

        self.new_entries.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn get(&mut self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        if self.state != TransactionState::Unknown {
            bail!("Transaction already {:?}", self.state);
        }

        if let Some(value) = self.new_entries.get(key) {
            return Ok(Some(value.clone()));
        }

        self.database.get(key)
    }

    fn commit(&mut self) -> anyhow::Result<()> {
        if self.state != TransactionState::Unknown {
            bail!("Transaction already {:?}", self.state);
        }

        // We need to serialize the new entries in order, into a series of bytes
        // To ensure order, we first convert the hashmap into a vector of tuples
        let new_entries: Vec<(String, Vec<u8>)> = self.new_entries.drain().collect();
        let data_series = new_entries
            .iter()
            .flat_map(|(_, v)| v.clone())
            .collect::<Vec<_>>();
        let offset = match self.database.merge_file(data_series.as_slice()) {
            Ok(o) => o,
            Err(e) => {
                self.rollback()?;
                bail!("Failed to write to database: {:?}, rolling back", e);
            }
        };

        let mut running_offset = offset;
        for (key, value) in new_entries {
            let len = value.len();
            match self
                .index_table_snapshot
                .insert(key.as_str(), (running_offset, len))
            {
                Ok(_) => (),
                Err(e) => {
                    self.rollback()?;
                    bail!("Failed to write to database: {:?}, rolling back", e);
                }
            }
            running_offset += len as u64;
        }

        self.state = TransactionState::Commit;
        Ok(())
    }

    fn rollback(&mut self) -> anyhow::Result<()> {
        if self.state == TransactionState::Rollback {
            bail!("Transaction was already rolled back!");
        }

        #[cfg(feature = "error-on-rollback-committed")]
        if self.state == TransactionState::Commit {
            bail!("Transaction was already committed, cannot rollback!");
        }

        self.state = TransactionState::Rollback;
        Ok(())
    }
}
