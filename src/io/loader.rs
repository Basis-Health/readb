use crate::io::buffered_file::BufferedFile;
use crate::io::Loader;
use std::path::PathBuf;

pub struct LazyLoader {
    file: BufferedFile,
}

impl LazyLoader {
    pub fn new<P: Into<PathBuf> + Clone>(path: P) -> Self {
        let file = BufferedFile::new(path);
        Self { file }
    }
}

impl Loader for LazyLoader {
    fn load(&mut self, offset: u64, length: usize) -> anyhow::Result<Vec<u8>> {
        Ok(self.file.read(offset, length)?)
    }

    #[cfg(feature = "write")]
    fn add(&mut self, data: &[u8]) -> anyhow::Result<(u64, usize)> {
        let (offset, length) = self.file.add(data)?;
        Ok((offset, length))
    }

    #[cfg(feature = "write")]
    fn persist(&mut self) -> anyhow::Result<()> {
        Ok(self.file.persist()?)
    }

    #[cfg(feature = "garbage-collection")]
    fn read_and_replace<F: FnOnce(&[u8]) -> anyhow::Result<Vec<u8>>>(
        &mut self,
        f: F,
    ) -> anyhow::Result<()> {
        let data = self.file.read_all()?;
        let new_data = f(&data)?;
        self.file.replace_with(&new_data)?;
        Ok(())
    }
}
