//! # Readatabase (readb)
//!
//! `readb` is a simple, embedded read-only key-value database implemented in Rust. Its emphasis on performance
//! makes it exceptionally faster than some other databases like `sled` (refer to the benchmark section in the README).
//!
//! ## Key Features:
//! - **Fast and Efficient**: Leverages caching to accelerate frequently accessed reads.
//! - **Modular Design**: Seamlessly switch between various indexing strategies and caching methods.
//! - **Lock-Free Reads**: Enables concurrent reads without locks, improving throughput.
//!
//! ## Limitations:
//! - **Read-Only**: Designed for read operations. Writing is not supported.
//! - **One-Time Indexing**: Data is indexed once at initialization, making it unsuitable for dynamic data additions.
//!
//! ## Main Functionality:
//! ### Base API:
//! - `new`: Constructs a new database.
//! - `get`: Retrieves the value associated with a given key.
//! - `link`: Aliases one key to another.
//! - `delete`: Removes a key. Links and actual data remain unaffected.
//! - `persist`: Saves new links to the database.
//!
//! ### `remote-cloning` Feature:
//! With the `remote-cloning` feature enabled:
//! - `clone_from`: Copies the database from a remote address to a local directory.
//!     - `address`: The remote source.
//!     - `path`: The local destination.
//!     - `compression`: Specifies the type of compression to use during transfer. If `None`, no compression is applied.
//!
//! For a detailed guide and benchmarking, please refer to the README.

mod cache;
pub(crate) mod index_table;
mod io;
pub use index_table::factory::IndexType;

mod api;
mod database;

pub use api::*;

pub use index_table::IndexTable;

#[cfg(feature = "remote-cloning")]
mod remote;
#[cfg(feature = "remote-cloning")]
pub use remote::{cloner::clone_from, compression::CompressionType};

#[cfg(feature = "garbage-collection")]
mod garbage_collection;
