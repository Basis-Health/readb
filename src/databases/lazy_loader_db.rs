use crate::cache::Cache;
use crate::databases::db_trait::{Database, DatabaseSettings};
use crate::index_table::factory::IndexFactory;
use crate::io::loader::LazyLoader;
use crate::io::Loader;
use crate::IndexTable;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "write")]
use crate::cache::Key;
#[cfg(feature = "write")]
use crate::databases::db_trait::DatabaseTransactionsIO;
#[cfg(feature = "write")]
use crate::transactions::{simple::SimpleTransaction, Transaction};
#[cfg(feature = "write")]
use anyhow::bail;

#[cfg(feature = "garbage-collection")]
use crate::garbage_collection::compact_file;

/// The main database structure.
///
/// Represents the core of the database, managing the index table, cache, and data loading.
///
/// # Generic Parameters
/// - `C`: The type representing the cache mechanism. Must implement the `Cache` trait.
/// - `L`: The type responsible for data loading. Must implement the `Loader` trait.
pub struct LLDatabase<C: Cache> {
    index_table: Box<dyn IndexTable>,
    cache: C,
    loader: LazyLoader,
}

impl<C: Cache> LLDatabase<C> {
    // Unwrap was used before 0.4.0, as the creation of a database returned a result, now it'll panic
    // if it cannot create the Database, albeit you can pass the create_directory param to handle the
    // common edge case of not ensuring the path exists before creating the database
    #[deprecated]
    pub fn unwrap(self) -> LLDatabase<C> {
        self
    }
}

impl<C: Cache + Send + Sync> Database for LLDatabase<C> {
    fn new(settings: DatabaseSettings) -> Self {
        if settings.path.is_none() {
            panic!("Path is required");
        }

        let path = settings.path.unwrap();
        if !path.exists() && settings.create_path {
            fs::create_dir(path.clone()).unwrap();
        }

        // path has to be a dictionary
        #[cfg(not(feature = "ignore-path-check"))]
        if !path.is_dir() {
            panic!("Path must be a directory");
        }

        let index_table = IndexFactory::new(settings.index_type)
            .load_or_create(path.clone())
            .unwrap();
        let cache = if settings.cache_size.is_some() {
            C::new(settings.cache_size.unwrap())
        } else {
            C::new_default()
        };

        Self {
            index_table,
            cache,
            loader: LazyLoader::new(path.join("./.rdb.data")),
        }
    }

    fn new_default(location: PathBuf) -> Self {
        Self::new(DatabaseSettings {
            path: Some(location),
            ..Default::default()
        })
    }

    fn get(&mut self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let index = self.index_table.get(key);
        if index.is_none() {
            return Ok(None);
        }
        let index = index.unwrap();

        let cached = self.cache.get(&index);
        if cached.is_some() {
            return Ok(cached);
        }

        let (offset, length) = index;
        let d = self.loader.load(offset, length);
        if d.is_err() {
            println!("Error loading data: {:?}", d.err());
            return Ok(None);
        }
        let d = d.unwrap();

        self.cache.put(index, d.clone());
        Ok(Some(d))
    }

    fn link(&mut self, old: &str, new: &str) -> anyhow::Result<()> {
        let index = self.index_table.get(old);
        if index.is_none() {
            return Err(anyhow::anyhow!("Key not found"));
        }
        let index = index.unwrap();
        self.index_table.insert(new, index)
    }

    fn delete(&mut self, key: &str) -> anyhow::Result<()> {
        self.index_table.delete(key)
    }

    fn persist(&mut self) -> anyhow::Result<()> {
        self.index_table.persist()?;

        #[cfg(feature = "write")]
        self.loader.persist()?;

        Ok(())
    }

    #[cfg(feature = "write")]
    fn put(&mut self, key: &str, value: &[u8]) -> anyhow::Result<()> {
        let index = self.loader.add(value)?;
        self.index_table.insert(key, index)
    }

    #[cfg(feature = "garbage-collection")]
    fn gc(&mut self) -> anyhow::Result<()> {
        self.loader.read_and_replace(|data| {
            let keys = self.index_table.all_key_values();
            let (new_keys, new_data) = compact_file(keys, data);
            self.index_table.replace_all(new_keys)?;

            Ok(new_data)
        })
    }

    #[cfg(feature = "write")]
    fn tx(&mut self) -> anyhow::Result<Box<dyn Transaction + '_>> {
        Ok(Box::new(SimpleTransaction::new(self)))
    }
}

#[cfg(feature = "write")]
impl<C: Cache + Send + Sync> DatabaseTransactionsIO for LLDatabase<C> {
    fn snapshot(&self) -> Box<dyn IndexTable> {
        self.index_table.snapshot()
    }

    fn merge_file(&mut self, new_content: &[u8]) -> anyhow::Result<u64> {
        Ok(self.loader.add(new_content)?.0)
    }

    fn merge_index_table(
        &mut self,
        backup: Box<dyn IndexTable>,
        new_key_values: Vec<(String, Key)>,
    ) -> anyhow::Result<()> {
        for (k, v) in new_key_values {
            match self.index_table.insert(k.as_str(), v) {
                Ok(_) => (),
                Err(e) => {
                    self.rollback(backup).unwrap_or_else(|e2| {
                        panic!(
                            "[FATAL] Failed to apply rollback: {}\nOriginal Error: {}",
                            e2, e
                        )
                    });
                    bail!("Failed to merge index tables: {}", e);
                }
            }
        }

        Ok(())
    }

    fn rollback(&mut self, index_table: Box<dyn IndexTable>) -> anyhow::Result<()> {
        self.index_table = index_table;
        Ok(())
    }
}
