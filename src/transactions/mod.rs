pub trait Transaction: Send + Sync {
    fn put(&mut self, key: &str, value: &[u8]) -> anyhow::Result<()>;

    fn get(&mut self, key: &str) -> anyhow::Result<Option<Vec<u8>>>;

    fn commit(&mut self) -> anyhow::Result<()>;

    fn rollback(&mut self) -> anyhow::Result<()>;
}

pub(crate) mod simple;

#[derive(Debug, PartialEq, Eq)]
enum TransactionState {
    Unknown,
    Commit,
    Rollback,
}
