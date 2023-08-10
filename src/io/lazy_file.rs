use std::fs::File;

#[cfg(feature = "write")]
use std::fs::OpenOptions;
use std::io::{self, prelude::*, SeekFrom};
use std::path::PathBuf;

pub struct LazyFile {
    path: PathBuf,
    file: Option<File>,

    #[cfg(feature = "write")]
    current_offset: usize,
}

impl LazyFile {
    pub fn new<P: Into<PathBuf>>(path: P) -> LazyFile {
        LazyFile {
            path: path.into(),
            file: None,

            #[cfg(feature = "write")]
            current_offset: 0,
        }
    }

    fn open_file(&mut self) -> io::Result<()> {
        if self.file.is_none() {
            #[cfg(not(feature = "write"))]
            {
                self.file = Some(File::open(&self.path)?);
            }

            #[cfg(feature = "write")]
            {
                // check if the file exists
                if self.path.exists() {
                    {
                        let f = OpenOptions::new().read(true).write(true).open(&self.path)?;
                        self.file = Some(f);
                    }

                    // Count the number of lines in the file
                    let file = self.file.as_mut().unwrap();
                    let reader = io::BufReader::new(file);

                    // read the amount of bytes in the file
                    let mut bytes = 0;
                    for line in reader.lines() {
                        bytes += line.unwrap().len() + 1;
                    }
                    bytes -= 1; // remove last newline

                    self.current_offset = bytes;
                } else {
                    self.file = Some(File::create(&self.path)?);
                }
            }
        }
        Ok(())
    }

    pub fn read(&mut self, offset: usize, len: usize) -> anyhow::Result<Vec<u8>> {
        self.open_file()?;

        let file = self.file.as_mut().unwrap();

        file.seek(SeekFrom::Start(offset as u64))?;

        let mut reader = io::BufReader::new(file);
        let mut buf = vec![0; len];
        reader.read_exact(&mut buf)?;

        Ok(buf)
    }

    #[cfg(feature = "write")]
    pub(crate) fn add(&mut self, data: &[u8]) -> anyhow::Result<(usize, usize)> {
        self.open_file()?;

        let file = self.file.as_mut().unwrap();

        file.seek(SeekFrom::Start(self.current_offset as u64))?;
        file.write_all(data)?;

        self.current_offset += data.len();

        Ok((self.current_offset - data.len(), data.len()))
    }

    #[cfg(feature = "write")]
    pub(crate) fn persist(&mut self) -> anyhow::Result<()> {
        self.open_file()?;

        let file = self.file.as_ref().unwrap();

        file.sync_all()?;
        Ok(())
    }

    #[cfg(feature = "write")]
    pub(crate) fn current_offset(&self) -> usize {
        self.current_offset
    }
}
