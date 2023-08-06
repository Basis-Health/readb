use std::path::PathBuf;
use crate::cache::Cache;
use crate::index_table::factory::{IndexFactory, IndexType};
use crate::index_table::IndexTable;
use crate::io::Loader;
use crate::io::loader::LazyLoader;

pub struct Database<C: Cache, L: Loader> {
    index_table: Box<dyn IndexTable>,
    cache: C,
    loader: L,
}

pub struct DatabaseSettings {
    pub path: Option<PathBuf>,
    pub cache_size: Option<usize>,
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

    pub fn new_default(location: PathBuf) -> Self {
        let settings = DatabaseSettings {
            path: Some(location),
            ..Default::default()
        };
        Self::new(settings).unwrap()
    }

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

    pub fn link(&mut self, old: &str, new: &str) -> anyhow::Result<()> {
        let index = self.index_table.get(old);
        if index.is_none() {
            return Err(anyhow::anyhow!("Key not found"));
        }
        let index = index.unwrap();
        self.index_table.insert(new.to_string(), *index)
    }

    pub fn delete(&mut self, key: &str) -> anyhow::Result<()> {
        self.index_table.delete(key)
    }

    pub fn persist(&mut self) -> anyhow::Result<()> {
        self.index_table.persist()
    }
}