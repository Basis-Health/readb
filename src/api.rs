use crate::cache::lfu::LfuCache;
use crate::databases::lazy_loader_db::LLDatabase;

pub type DefaultDatabase = LLDatabase<LfuCache>;

pub use crate::databases::db_trait::Database;
pub use crate::databases::db_trait::DatabaseSettings;
