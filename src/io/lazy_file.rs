use std::fs::File;

#[cfg(feature = "write")]
use std::fs::OpenOptions;
use std::io::{self, prelude::*, SeekFrom};
use std::path::PathBuf;

pub struct LazyFile {
    path: PathBuf,
    file: Option<File>,

    #[cfg(feature = "write")]
    line_count: usize,
}

impl LazyFile {
    pub fn new<P: Into<PathBuf>>(path: P) -> LazyFile {
        LazyFile {
            path: path.into(),
            file: None,

            #[cfg(feature = "write")]
            line_count: 0,
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
                    self.line_count = reader.lines().count();
                } else {
                    self.file = Some(File::create(&self.path)?);
                }
            }
        }
        Ok(())
    }

    pub fn get_line(&mut self, line_number: usize) -> io::Result<String> {
        self.open_file()?;

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

    #[cfg(feature = "write")]
    pub(crate) fn add(&mut self, line: &str) -> anyhow::Result<usize> {
        self.open_file()?;

        let file = self.file.as_mut().unwrap();

        file.seek(SeekFrom::End(0))?;
        let mut writer = io::BufWriter::new(file);

        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;

        self.line_count += 1;
        println!("Line count: {}", self.line_count - 1);
        Ok(self.line_count - 1)
    }

    #[cfg(feature = "write")]
    pub(crate) fn persist(&mut self) -> anyhow::Result<()> {
        self.open_file()?;

        let file = self.file.as_ref().unwrap();

        file.sync_all()?;
        Ok(())
    }
}
