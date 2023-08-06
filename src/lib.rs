mod cache;
mod io;
pub(crate) mod index_table;
pub use index_table::factory::IndexType;

mod api;
mod database;

pub use api::*;

#[cfg(feature = "index-write")]
mod index_api;
#[cfg(feature = "index-write")]
pub use index_api::*;
#[cfg(feature = "index-write")]
pub use index_table::IndexTable;

#[cfg(feature = "remote-cloning")]
mod remote;
#[cfg(feature = "remote-cloning")]
pub use remote::{cloner::clone_from, compression::CompressionType};
