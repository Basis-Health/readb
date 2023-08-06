use crate::cache::lfu::LfuCache;
use crate::io::loader::LazyLoader;

pub type DefaultDatabase = Database<LfuCache, LazyLoader>;

pub use crate::database::DatabaseSettings;
pub use crate::database::Database;