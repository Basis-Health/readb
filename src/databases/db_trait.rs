use crate::cache::Key;
use crate::{IndexTable, IndexType};
use std::path::PathBuf;

#[cfg(feature = "write")]
use crate::transactions::Transaction;

/// Configuration settings required to initialize a [`Database`].
///
/// It provides customizable settings like the storage path, cache size, and the type of index.
pub struct DatabaseSettings {
    /// Path to the database's directory.
    pub path: Option<PathBuf>,
    /// Size of the cache.
    pub cache_size: Option<usize>,
    /// Type of the index table.
    pub index_type: IndexType,

    /// Whether or not to create the path if it doesn't exist
    pub create_path: bool,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        DatabaseSettings {
            path: None,
            cache_size: None,
            index_type: IndexType::HashMap,
            create_path: false,
        }
    }
}

pub trait Database: Send + Sync {
    /// Constructs a new `Database` instance with the specified settings.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The path is not provided.
    /// - The provided path isn't a directory (unless the "ignore-path-check" feature is enabled).
    fn new(settings: DatabaseSettings) -> Self
    where
        Self: Sized;

    /// Constructs a new `Database` instance with default settings.
    ///
    /// # Parameters
    /// - `location`: Path to the database's directory.
    fn new_default(location: PathBuf) -> Self
    where
        Self: Sized;

    /// Retrieves the value associated with a given key.
    ///
    /// This method will first check the index, then the cache, and finally loads from disk if necessary.
    ///
    /// # Parameters
    /// - `key`: The key for which the value should be fetched.
    ///
    /// # Returns
    /// - `Some(String)` if the key is found.
    /// - `None` if the key doesn't exist or data loading fails.
    fn get(&mut self, key: &str) -> anyhow::Result<Option<Vec<u8>>>;

    /// Associates an existing key with a new key.
    ///
    /// This effectively creates an alias for the old key. Note, that removing the old key, will **not**
    /// change this key. If the garbage collection feature is enabled, this alias counts as an individual
    /// reference.
    ///
    /// # Parameters
    /// - `old`: The existing key.
    /// - `new`: The new alias for the key.
    ///
    /// # Errors
    /// Returns an error if the old key is not found in the index table.
    fn link(&mut self, old: &str, new: &str) -> anyhow::Result<()>;

    /// Deletes a key  from the index-table.
    /// Note, this does not delete the data from disk. Once all references to the data are removed,
    /// it might be garbage collected if the "garbage-collection" feature is enabled.
    ///
    /// # Parameters
    /// - `key`: The key to be removed.
    fn delete(&mut self, key: &str) -> anyhow::Result<()>;

    /// Commits the current state of the database, ensuring data persistence.
    /// Note, this only commits the index table.
    fn persist(&mut self) -> anyhow::Result<()>;

    /// Adds a new key-value pair to the database.
    /// Note: This method is only available if the "write" feature is enabled.
    #[cfg(feature = "write")]
    fn put(&mut self, key: &str, value: &[u8]) -> anyhow::Result<()>;

    /// Performs garbage collection on the database.
    /// Note: This method is only available if the "garbage-collection" feature is enabled.
    ///
    /// This method requires a full scan of the index table, and is therefore very slow. It is
    /// also not thread-safe.
    #[cfg(feature = "garbage-collection")]
    fn gc(&mut self) -> anyhow::Result<()>;

    /// Create a new transaction.
    /// Note: This method is only available if the "write" feature is enabled.
    ///
    /// Creates a new transaction, which can be used to batch write operations. The transaction
    /// must be committed to be persisted. If the transaction is dropped without being committed,
    /// all changes will be discarded.
    ///
    /// Note: The database is already batching writes, so this method is only useful if you want
    /// to perform lots of writes, or if you want to write multiple large values.
    ///
    /// # Example
    /// ```
    /// use readb::{Database, DefaultDatabase};
    ///
    /// let mut db = DefaultDatabase::new_default("./test_db".into());
    /// let mut transaction = db.tx().unwrap();
    /// transaction.put("key1", "value1".as_bytes()).unwrap();
    /// transaction.put("key2", "value2".as_bytes()).unwrap();
    /// transaction.commit().unwrap();
    /// ```
    #[cfg(feature = "write")]
    fn tx(&mut self) -> anyhow::Result<Box<dyn Transaction + '_>>;
}

pub(crate) trait DatabaseTransactionsIO: Database {
    // Perform a snapshot of the index table
    fn snapshot(&self) -> Box<dyn IndexTable>;

    // Merges the content to the file returning the offset of the first byte placed
    fn merge_file(&mut self, new_content: &[u8]) -> anyhow::Result<u64>;

    fn merge_index_table(
        &mut self,
        backup: Box<dyn IndexTable>,
        new_key_values: Vec<(String, Key)>,
    ) -> anyhow::Result<()>;

    fn rollback(&mut self, index_table: Box<dyn IndexTable>) -> anyhow::Result<()>;
}
