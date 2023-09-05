//! # Readatabase (readb) 🗄️
//!
//! `readb` is a high-performance, embedded key-value database engineered in Rust. Designed with a read-first approach,
//! it's optimized for read-heavy workloads and demonstrates superior speed compared to certain databases like `sled` (for benchmarks, refer to the README).
//!
//! ## 🌟 Features
//! - **Swift & Efficient**: Uses smart caching to speed up recurrent reads.
//! - **Modular Design**: Easily switch between various indexing and caching strategies.
//! - **Lock-Free Reads**: Conducts concurrent reads without locks, ensuring optimal throughput.
//! - **Minimalistic**: Few dependencies and a tiny footprint, weighing under 1KB.
//!
//! ## ❗ Characteristics
//! - **Read-Centric**: While write and delete operations are supported, `readb` is optimized for reading.
//! - **Append-Only Strategy**: Adds new data to the end, aiding in its rapid read capabilities.
//! - **Light on Deletion**: `readb` isn't structured around deletes, and unused data remains until garbage collection is triggered.
//!
//! ## 🔧 Core API
//! - `new`: Initialize a new database.
//! - `get`: Fetch the value paired with a particular key.
//! - `link`: Set up an alias between two keys.
//! - `delete`: Eliminate a key from the index; actual data remains untouched.
//! - `persist`: Make certain the recent changes are stored permanently.
//!
//! ## 🌐 `remote-cloning` Feature
//! Upon activating the `remote-cloning` feature:
//! - `clone_from`: Transfers the database from a specified remote source to a local directory.
//!   - `address`: The source address.
//!   - `path`: The target directory on the local machine.
//!   - `compression`: Dictates the compression type during the transfer, defaulting to `None` for no compression.
//!
//! Consult the README for a comprehensive guide, feature details, and performance benchmarks.

mod cache;
pub(crate) mod index_table;
mod io;
pub use index_table::factory::IndexType;

mod api;
mod databases;

pub use api::*;

pub use index_table::IndexTable;

#[cfg(feature = "remote-cloning")]
mod remote;
#[cfg(feature = "remote-cloning")]
pub use remote::{cloner::clone_from, compression::CompressionType};

#[cfg(feature = "garbage-collection")]
mod garbage_collection;

#[cfg(feature = "write")]
mod transactions;
