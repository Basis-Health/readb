use crate::index_table::btree::BTreeMapIndexTable;
use crate::index_table::hash_map::HashMapIndexTable;
use crate::index_table::IndexTable;
use anyhow::{bail, Result};
use fs2::FileExt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

const TYPE_EXTENSION: &str = "type";
const DEFAULT_INDEX_NAME: &str = ".rdb.index";

#[derive(PartialEq)]
pub enum IndexType {
    HashMap,
    BTreeMap,
    // Determined at runtime, or HashMap if not specified
    Auto,
}

pub struct IndexFactory {
    index_type: IndexType,
}

impl IndexFactory {
    pub fn new(index_type: IndexType) -> Self {
        IndexFactory { index_type }
    }

    fn path2path(path: PathBuf) -> (PathBuf, PathBuf) {
        let path = if path.is_dir() {
            path.join(DEFAULT_INDEX_NAME)
        } else {
            path
        };
        let type_path = path.with_extension(TYPE_EXTENSION);
        (path, type_path)
    }

    pub fn create(&self, path: PathBuf) -> Result<Box<dyn IndexTable>> {
        let (path, type_path) = IndexFactory::path2path(path);

        // Create the file
        let file = File::create(type_path)?;
        file.lock_exclusive()?;

        // Write the type to the file
        let mut writer = BufWriter::new(&file);
        match self.index_type {
            IndexType::HashMap => writer.write_all(b"HashMap")?,
            IndexType::BTreeMap => writer.write_all(b"BTreeMap")?,
            IndexType::Auto => bail!("Cannot create index with type Auto"),
        }

        // Create the appropriate index table
        let index_table: Box<dyn IndexTable> = match self.index_type {
            IndexType::HashMap => Box::new(HashMapIndexTable::new_default(path)?),
            IndexType::BTreeMap => Box::new(BTreeMapIndexTable::new_default(path)?),
            IndexType::Auto => unreachable!(),
        };

        file.unlock()?;
        Ok(index_table)
    }

    pub fn load(&mut self, path: PathBuf) -> Result<Box<dyn IndexTable>> {
        let (path, type_path) = IndexFactory::path2path(path);

        let file = File::open(type_path)?;

        // Lock the file
        file.lock_exclusive()?;

        let mut reader = BufReader::new(&file);
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;

        // Check if the type in the file matches the specified type
        let file_type = match first_line.trim() {
            "HashMap" => IndexType::HashMap,
            "BTreeMap" => IndexType::BTreeMap,
            _ => bail!("Unknown index type {} in file", first_line),
        };

        if file_type != self.index_type {
            if self.index_type == IndexType::Auto {
                self.index_type = file_type;

                // If the index type is Auto, then use the type in the file
                file.unlock()?;
                return self.load(path);
            }

            bail!("Index type in file does not match specified type");
        }

        // Create the appropriate index table
        let mut index_table: Box<dyn IndexTable> = match self.index_type {
            IndexType::HashMap => Box::new(HashMapIndexTable::new(path)?),
            IndexType::BTreeMap => Box::new(BTreeMapIndexTable::new(path)?),
            IndexType::Auto => unreachable!(),
        };

        index_table.load()?;
        file.unlock()?;
        Ok(index_table)
    }

    pub fn load_or_create(&mut self, path: PathBuf) -> Result<Box<dyn IndexTable>> {
        let (_, type_path) = IndexFactory::path2path(path.clone());

        // Check if the file exists
        if type_path.exists() {
            return self.load(path);
        }

        self.create(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;

    #[test]
    fn test_create() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let index_path = temp_dir.path().join("index.bin");

        // Test - create a HashMap index table
        let index_factory = IndexFactory::new(IndexType::HashMap);
        index_factory.create(index_path.clone())?;

        // Assert - verify the type file is correct
        let type_path = index_path.with_extension(TYPE_EXTENSION);
        let mut file = File::open(&type_path)?;
        let mut type_string = String::new();
        file.read_to_string(&mut type_string)?;
        assert_eq!(type_string, "HashMap");

        // Cleanup
        temp_dir.close()?;

        Ok(())
    }

    #[test]
    fn test_load() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let index_path = temp_dir.path().join("index.bin");

        // Setup - create a HashMap index table
        let mut index_factory = IndexFactory::new(IndexType::HashMap);
        index_factory.create(index_path.clone())?;

        // Test - load the HashMap index table
        let loaded_table = index_factory.load(index_path)?;

        // Assert - verify the loaded table is correct
        assert_eq!(loaded_table.index_type(), "hash_map");

        // Cleanup
        temp_dir.close()?;

        Ok(())
    }
}
