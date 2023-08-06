use std::path::PathBuf;
use crate::io::lazy_file::LazyFile;
use crate::io::Loader;

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
}
