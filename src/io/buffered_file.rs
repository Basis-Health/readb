use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const DEFAULT_BUFFER_SIZE: usize = 4096; // For example, 4KB

pub(crate) struct BufferedFile {
    path: PathBuf,
    buffer: Vec<u8>,
    file_length: u64,
    file: Option<File>,
}

impl BufferedFile {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();

        let file_length = if path.exists() {
            File::open(&path).unwrap().metadata().unwrap().len()
        } else {
            0
        };

        let file = Some(File::open(&path).unwrap_or_else(|_| {
            OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&path)
                .unwrap()
        }));

        BufferedFile {
            path,
            buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            file_length,
            file,
        }
    }

    pub fn read(&mut self, offset: u64, length: usize) -> Result<Vec<u8>, std::io::Error> {
        if offset >= (self.file_length - self.buffer.len() as u64) {
            let start = (offset - (self.file_length - self.buffer.len() as u64)) as usize;
            let end = start + length;

            return Ok(self.buffer[start..end].to_vec());
        }

        self.ensure_file_open()?;
        let file = self.file.as_mut().unwrap();
        file.seek(SeekFrom::Start(offset))?;

        let mut data = vec![0u8; length];
        file.read_exact(&mut data)?;

        Ok(data)
    }

    pub fn add(&mut self, data: &[u8]) -> Result<(u64, usize), std::io::Error> {
        let offset = self.file_length;

        if self.buffer.len() + data.len() > DEFAULT_BUFFER_SIZE {
            self.persist()?;
        }

        self.buffer.extend_from_slice(data);
        self.file_length += data.len() as u64;

        Ok((offset, data.len()))
    }

    pub fn persist(&mut self) -> Result<(), std::io::Error> {
        self.ensure_file_closed();

        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        file.write_all(&self.buffer)?;

        self.buffer.clear();
        Ok(())
    }

    fn ensure_file_open(&mut self) -> Result<(), std::io::Error> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        Ok(())
    }

    fn ensure_file_closed(&mut self) {
        if let Some(file) = self.file.take() {
            drop(file); // Close the file explicitly
        }
    }

    pub fn read_all(&mut self) -> Result<Vec<u8>, std::io::Error> {
        self.persist()?; // Persist the buffer before reading the file
        self.ensure_file_open()?;

        let file = self.file.as_mut().unwrap();
        file.seek(SeekFrom::Start(0))?;

        let mut data = vec![0u8; self.file_length as usize];
        file.read_exact(&mut data)?;

        Ok(data)
    }

    pub fn replace_with(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.ensure_file_closed();
        let mut file = OpenOptions::new().write(true).open(&self.path)?;
        file.write_all(data)?;
        self.file_length = data.len() as u64;

        // Remove everything after the new data
        file.set_len(self.file_length)?;

        Ok(())
    }
}
