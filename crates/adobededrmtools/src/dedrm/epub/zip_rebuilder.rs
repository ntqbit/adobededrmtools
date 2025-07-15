use std::io::{Read, Seek, Write};

use anyhow::Context;
use zip::{ZipArchive, read::ZipFile};

pub trait ZipRebuilder {
    fn init<Z: ZipReader>(&mut self, zip: &mut Z) -> anyhow::Result<()>;

    fn process_file<Z: ZipFileW>(&mut self, file: &mut Z) -> anyhow::Result<ZipFileDisposition>;
}

pub enum ZipFileDisposition {
    Delete,
    Retain,
    Modify(Vec<u8>),
}

pub trait ZipFileW {
    fn name(&self) -> &str;

    fn read_file(&mut self) -> anyhow::Result<Vec<u8>>;
}

pub trait ZipReader {
    fn read_file(&mut self, name: &str) -> anyhow::Result<Option<Vec<u8>>>;
}

impl<'a, R: Read + Seek> ZipFileW for ZipFile<'a, R> {
    fn name(&self) -> &str {
        self.name()
    }

    fn read_file(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

impl<R: Read + Seek> ZipReader for ZipArchive<R> {
    fn read_file(&mut self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        match self.by_name(name) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
            Err(zip::result::ZipError::FileNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

pub fn rebuild_zip<R, W, B>(input: R, output: W, mut rebuilder: B) -> anyhow::Result<W>
where
    R: Read + Seek,
    W: Write + Seek,
    B: ZipRebuilder,
{
    let mut archive = zip::read::ZipArchive::new(input).context("read archive failed")?;
    let mut out_arhive = zip::write::ZipWriter::new(output);

    rebuilder
        .init(&mut archive)
        .context("rebuilder init failed")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("could not get file by index")?;

        let disposition = rebuilder
            .process_file(&mut file)
            .context("failed to process file")?;

        match disposition {
            ZipFileDisposition::Delete => {}
            ZipFileDisposition::Retain => {
                out_arhive.raw_copy_file(file)?;
            }
            ZipFileDisposition::Modify(data) => {
                out_arhive.start_file(
                    file.name(),
                    file.options()
                        .compression_method(zip::CompressionMethod::default()),
                )?;
                out_arhive.write_all(&data)?;
            }
        }
    }

    Ok(out_arhive.finish().context("zip finish failed")?)
}
