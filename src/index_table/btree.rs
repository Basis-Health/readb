use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use bincode::{serialize_into, deserialize_from};
use fs2::FileExt;
use crate::default_persist;
use crate::index_table::IndexTable;

#[repr(C)]
pub struct BTreeMapIndexTable {
    table: BTreeMap<String, usize>,
    file_path: PathBuf,
}

impl BTreeMapIndexTable {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(&path)?;

        // Lock the file
        file.lock_exclusive()?;

        let reader = BufReader::new(&file);
        let table: BTreeMap<String, usize> = match deserialize_from(reader) {
            Ok(table) => table,
            Err(_) => BTreeMap::new(),
        };

        // Remember to unlock the file when done
        file.unlock()?;

        Ok(Self { table, file_path: path })
    }

    pub fn new_default(path: PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            table: BTreeMap::new(),
            file_path: path,
        })
    }
}

impl IndexTable for BTreeMapIndexTable {
    fn get(&self, key: &str) -> Option<&usize> {
        self.table.get(key)
    }

    fn insert(&mut self, key: String, value: usize) -> anyhow::Result<()> {
        self.table.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: &str) -> anyhow::Result<()> {
        self.table.remove(key);
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        let file = File::open(&self.file_path)?;

        // Lock the file
        file.lock_exclusive()?;

        let reader = BufReader::new(&file);
        self.table = match deserialize_from(reader) {
            Ok(table) => table,
            Err(_) => BTreeMap::new(),
        };

        // Remember to unlock the file when done
        file.unlock()?;

        Ok(())
    }

    fn persist(&self) -> anyhow::Result<()> {
        default_persist!(self, self.file_path.clone(), self.table);
        Ok(())
    }


    #[cfg(feature = "fuzzy-search")]
    fn keys(&self) -> Vec<String> {
        self.table.keys().cloned().collect()
    }


    #[cfg(test)]
    fn index_type(&self) -> &str {
        "btree"
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() -> anyhow::Result<()> {
        // Setup - create a temporary directory
        let temp_dir = tempfile::tempdir()?;
        let index_path = temp_dir.path().join("index.bin");

        // Create a new index table
        let mut index_table = BTreeMapIndexTable::new(index_path.clone())?;

        // Test - insert an item
        let key = "test".to_string();
        let value = 123;
        index_table.insert(key.clone(), value)?;

        // Assert - get returns the correct value
        assert_eq!(index_table.get(&key), Some(&value));

        // Test - persist to disk
        index_table.persist()?;

        // Assert - load from disk
        let mut loaded_table = BTreeMapIndexTable::new(index_path)?;
        loaded_table.load()?;
        assert_eq!(loaded_table.get(&key), Some(&value));

        // Cleanup
        temp_dir.close()?;

        Ok(())
    }
}
