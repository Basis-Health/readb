use std::io::{Read, Seek, Write};
use std::path::Path;
use anyhow::Result;

pub enum CompressionType {
    Uncompressed,

    #[cfg(feature = "remote-brotli-compression")]
    Brotli
}

#[cfg(feature = "remote-brotli-compression")]
const DEFAULT_BROTLI_BUFFER_SIZE: usize = 4096;

pub(crate) fn decompress(path: &Path, compression_type: &CompressionType) -> Result<()> {
    let mut f = std::fs::OpenOptions::new().read(true).write(true).open(path)?;
    decompress_file(&mut f, compression_type)
}

pub(crate) fn decompress_file(f: &mut std::fs::File, compression_type: &CompressionType) -> Result<()> {
    match compression_type {
        CompressionType::Uncompressed => Ok(()),

        #[cfg(feature = "remote-brotli-compression")]
        CompressionType::Brotli => {
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;

            let mut decompressed = Vec::new();
            let mut decoder = brotli::Decompressor::new(&buffer[..], DEFAULT_BROTLI_BUFFER_SIZE);
            decoder.read_to_end(&mut decompressed)?;

            f.seek(std::io::SeekFrom::Start(0))?;
            f.write_all(&decompressed)?;
            Ok(())
        }
    }
}