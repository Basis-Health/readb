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

    #[cfg(test)]
    fn index_type(&self) -> &str;
}
