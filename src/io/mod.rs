pub(crate) mod lazy_file;
pub(crate) mod loader;

pub trait Loader {
    fn load(&mut self, line_number: usize) -> anyhow::Result<String>;
}
