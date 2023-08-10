pub(crate) mod lazy_file;
pub(crate) mod loader;

pub trait Loader {
    fn load(&mut self, offset: usize, length: usize) -> anyhow::Result<Vec<u8>>;

    #[cfg(feature = "write")]
    fn add(&mut self, data: &[u8]) -> anyhow::Result<(usize, usize)>;
    #[cfg(feature = "write")]
    fn persist(&mut self) -> anyhow::Result<()>;
}
