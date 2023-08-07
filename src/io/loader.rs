use crate::io::lazy_file::LazyFile;
use crate::io::Loader;
use std::path::PathBuf;

pub struct LazyLoader {
    file: LazyFile,
}

impl LazyLoader {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        LazyLoader {
            file: LazyFile::new(path),
        }
    }
}

impl Loader for LazyLoader {
    fn load(&mut self, line_number: usize) -> anyhow::Result<String> {
        Ok(self.file.get_line(line_number)?)
    }

    #[cfg(feature = "write")]
    fn add(&mut self, line: &str) -> anyhow::Result<usize> {
        self.file.add(line)
    }

    #[cfg(feature = "write")]
    fn persist(&mut self) -> anyhow::Result<()> {
        self.file.persist()
    }
}
