//pub(crate) mod lazy_file;
mod buffered_file;
pub(crate) mod loader;

pub trait Loader {
    fn load(&mut self, offset: u64, length: usize) -> anyhow::Result<Vec<u8>>;

    #[cfg(feature = "write")]
    fn add(&mut self, data: &[u8]) -> anyhow::Result<(u64, usize)>;
    #[cfg(feature = "write")]
    fn persist(&mut self) -> anyhow::Result<()>;

    #[cfg(feature = "garbage-collection")]
    fn read_and_replace<F: FnOnce(&[u8]) -> anyhow::Result<Vec<u8>>>(
        &mut self,
        f: F,
    ) -> anyhow::Result<()>;
}
