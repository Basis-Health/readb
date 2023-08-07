pub(crate) mod lazy_file;
pub(crate) mod loader;

pub trait Loader {
    fn load(&mut self, line_number: usize) -> anyhow::Result<String>;

    #[cfg(feature = "write")]
    fn add(&mut self, line: &str) -> anyhow::Result<usize>;

    #[cfg(feature = "write")]
    fn persist(&mut self) -> anyhow::Result<()>;
}
