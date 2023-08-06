// Eventually hide the implementation of the index table
pub mod hash_map;
pub mod btree;

pub(crate) mod factory;

use anyhow::Result;

pub trait IndexTable {
    fn get(&self, key: &str) -> Option<&usize>;
    fn insert(&mut self, key: String, value: usize) -> Result<()>;
    fn delete(&mut self, key: &str) -> Result<()>;
    fn load(&mut self) -> Result<()>;
    fn persist(&self) -> Result<()>;

    #[cfg(feature = "fuzzy-search")]
    fn keys(&self) -> Vec<String>;

    #[cfg(test)]
    fn index_type(&self) -> &str;
}

#[macro_export]
macro_rules! default_persist {
    ($self:expr, $file_path:expr, $table:expr) => {
        let file = OpenOptions::new().write(true).truncate(true).open($file_path)?;

        // Lock the file
        file.lock_exclusive()?;

        let writer = BufWriter::new(&file);
        serialize_into(writer, &$table)?;

        // Remember to unlock the file when done
        file.unlock()?;

    };
}