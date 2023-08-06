use std::fs::File;
use std::io::{self, prelude::*, SeekFrom};
use std::path::PathBuf;

pub struct LazyFile {
    path: PathBuf,
    file: Option<File>,
}

impl LazyFile {
    pub fn new<P: Into<PathBuf>>(path: P) -> LazyFile {
        LazyFile {
            path: path.into(),
            file: None,
        }
    }

    pub fn get_line(&mut self, line_number: usize) -> io::Result<String> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }

        let file = self.file.as_mut().unwrap();

        file.seek(SeekFrom::Start(0))?;
        let reader = io::BufReader::new(file);

        let r = reader.lines().nth(line_number);
        match r {
            Some(Ok(line)) => Ok(line),
            Some(Err(e)) => Err(e),
            None => Err(io::Error::new(io::ErrorKind::Other, "Line not found")),
        }
    }
}
