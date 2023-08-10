use crate::io::lazy_file::LazyFile;
use crate::io::Loader;
use std::path::PathBuf;

pub struct LazyLoader {
    file: LazyFile,
    #[cfg(feature = "write")]
    current_offset: usize,
}

impl LazyLoader {
    pub fn new<P: Into<PathBuf> + Clone>(path: P) -> Self {
        #[cfg(not(feature = "write"))]
        {
            return LazyLoader {
                files: HashMap::new(),
            };
        }

        #[cfg(feature = "write")] // TODO: This should grab the last file and the last offset
        {
            let file = LazyFile::new(path.clone().into());
            let current_offset = file.current_offset();

            LazyLoader {
                file,
                current_offset,
            }
        }
    }
}

impl Loader for LazyLoader {
    fn load(&mut self, offset: usize, length: usize) -> anyhow::Result<Vec<u8>> {
        self.file.read(offset, length)
    }

    #[cfg(feature = "write")]
    fn add(&mut self, data: &[u8]) -> anyhow::Result<(usize, usize)> {
        let (offset, length) = self.file.add(data)?;
        self.current_offset = offset;
        Ok((offset, length))
    }

    #[cfg(feature = "write")]
    fn persist(&mut self) -> anyhow::Result<()> {
        self.file.persist()
    }
}
