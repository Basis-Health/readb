use crate::index_table::factory::IndexType;
use crate::index_table::IndexTable;
use anyhow::bail;
use std::path::PathBuf;

pub fn new_index_table(
    location: PathBuf,
    table_type: IndexType,
) -> anyhow::Result<Box<dyn IndexTable>> {
    Ok(match table_type {
        IndexType::HashMap => Box::new(crate::index_table::hash_map::HashMapIndexTable::new(
            location,
        )?),
        IndexType::BTreeMap => Box::new(crate::index_table::btree::BTreeMapIndexTable::new(
            location,
        )?),
        IndexType::Auto => bail!("Cannot create index with type Auto"),
    })
}
