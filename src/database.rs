use std::path::PathBuf;
use crate::cache::Cache;
use crate::index_table::factory::{IndexFactory, IndexType};
use crate::index_table::IndexTable;
use crate::io::Loader;
use crate::io::loader::LazyLoader;

/// The main database structure.
///
/// Represents the core of the database, managing the index table, cache, and data loading.
///
/// # Generic Parameters
/// - `C`: The type representing the cache mechanism. Must implement the `Cache` trait.
/// - `L`: The type responsible for data loading. Must implement the `Loader` trait.
pub struct Database<C: Cache, L: Loader> {
    index_table: Box<dyn IndexTable>,
    cache: C,
    loader: L,
}

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
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        DatabaseSettings {
            path: None,
            cache_size: None,
            index_type: IndexType::HashMap,
        }
    }
}

impl<C: Cache> Database<C, LazyLoader> {
    /// Constructs a new `Database` instance with the specified settings.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The path is not provided.
    /// - The provided path isn't a directory (unless the "ignore-path-check" feature is enabled).
    pub fn new(settings: DatabaseSettings) -> anyhow::Result<Self> {
        if settings.path.is_none() {
            return Err(anyhow::anyhow!("Path is required"));
        }

        // path has to be a dictionary
        #[cfg(not(feature = "ignore-path-check"))]
        if !settings.path.as_ref().unwrap().is_dir() {
            return Err(anyhow::anyhow!("Path must be a directory"));
        }

        let path = settings.path.clone().unwrap();

        let index_table = IndexFactory::new(settings.index_type).load_or_create(path.clone())?;
        let cache = if settings.cache_size.is_some() {
            C::new(settings.cache_size.unwrap())
        } else {
            C::new_default()
        };

        Ok(Database {
            index_table,
            cache,
            loader: LazyLoader::new(path.join("./.rdb.data"))
        })

    }

    /// Constructs a new `Database` instance with default settings.
    ///
    /// # Parameters
    /// - `location`: Path to the database's directory.
    pub fn new_default(location: PathBuf) -> Self {
        let settings = DatabaseSettings {
            path: Some(location),
            ..Default::default()
        };
        Self::new(settings).unwrap()
    }

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
    pub fn get(&mut self, key: &str) -> anyhow::Result<Option<String>> {
        let index = self.index_table.get(key);
        if index.is_none() {
            return Ok(None);
        }
        let index = index.unwrap();

        let cached = self.cache.get(index);
        if cached.is_some() {
            return Ok(cached);
        }

        let d = self.loader.load(*index);
        if d.is_err() {
            return Ok(None);
        }
        let d = d.unwrap();

        self.cache.put(*index, d.clone());
        Ok(Some(d))
    }

    /// Associates an existing key with a new key.
    ///
    /// This effectively creates an alias for the old key.
    ///
    /// # Parameters
    /// - `old`: The existing key.
    /// - `new`: The new alias for the key.
    ///
    /// # Errors
    /// Returns an error if the old key is not found in the index table.
    pub fn link(&mut self, old: &str, new: &str) -> anyhow::Result<()> {
        let index = self.index_table.get(old);
        if index.is_none() {
            return Err(anyhow::anyhow!("Key not found"));
        }
        let index = index.unwrap();
        self.index_table.insert(new.to_string(), *index)
    }

    /// Deletes a key and its associated value from the database.
    ///
    /// # Parameters
    /// - `key`: The key to be removed.
    pub fn delete(&mut self, key: &str) -> anyhow::Result<()> {
        self.index_table.delete(key)
    }

    /// Commits the current state of the database, ensuring data persistence.
    /// Note, this only commits the index table.
    pub fn persist(&mut self) -> anyhow::Result<()> {
        self.index_table.persist()
    }
}