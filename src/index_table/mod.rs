/// This module provides abstraction over different index table implementations.
///
/// It supports both `BTree` and `HashMap` based index tables.
/// The factory module provides utilities for creating these index tables.
pub mod btree;
pub mod hash_map;

pub(crate) mod factory;

use anyhow::Result;

/// Represents an index table which can store key-value pairs, where the key is a string and
/// the value is a tuple of two usize integers.
///
/// This trait provides basic CRUD operations for the index table, along with persistence mechanisms.
pub trait IndexTable: Send + Sync {
    /// Retrieves a value by its key.
    ///
    /// Returns `None` if the key is not present in the table.
    fn get(&self, key: &str) -> Option<(usize, usize)>;

    /// Inserts a key-value pair into the index table.
    ///
    /// Returns a `Result` indicating success or failure of the operation.
    fn insert(&mut self, key: &str, value: (usize, usize)) -> Result<()>;

    /// Deletes a key-value pair from the index table by its key.
    ///
    /// Returns a `Result` indicating success or failure of the operation.
    fn delete(&mut self, key: &str) -> Result<()>;

    /// Loads the index table from its storage.
    ///
    /// Returns a `Result` indicating success or failure of the operation.
    fn load(&mut self) -> Result<()>;

    /// Persists the current state of the index table to its storage.
    ///
    /// Returns a `Result` indicating success or failure of the operation.
    fn persist(&self) -> Result<()>;

    #[cfg(test)]
    /// Returns the type of the index for testing purposes.
    fn index_type(&self) -> &str;
}

#[macro_export]
/// This macro provides a default implementation for persisting an index table.
///
/// It takes in three parameters:
/// - `$self`: the reference to the current index table.
/// - `$file_path`: the path to the file where the index table will be persisted.
/// - `$table`: the actual data table that needs to be persisted.
macro_rules! default_persist {
    ($self:expr, $file_path:expr, $table:expr) => {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open($file_path)?;

        // Lock the file
        file.lock_exclusive()?;

        let writer = std::io::BufWriter::new(&file);
        bincode::serialize_into(writer, &$table)?;

        // Remember to unlock the file when done
        file.unlock()?;
    };
}
