mod encryption_file;
mod zip_rebuilder;

use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
};

use adobededrmtools_crypto::decrypt_aes;

use anyhow::Context;
use encryption_file::{
    Algorithm, CompressionAlgorithm, EncryptionAlgorithm, parse_encryption_file,
};

use zip_rebuilder::{ZipFileDisposition, ZipFileW, ZipReader, ZipRebuilder, rebuild_zip};

use super::AdeptEncryptionKey;

fn decrypt_file(
    encryption_key: &AdeptEncryptionKey,
    data: &[u8],
    algorithm: &EncryptionAlgorithm,
) -> anyhow::Result<Vec<u8>> {
    match algorithm {
        EncryptionAlgorithm::Aes128Cbc => {
            Ok(decrypt_aes(&encryption_key.raw(), data).context("failed to decrypt aes128cbc")?)
        }
    }
}

fn decompress_file(data: &[u8], algorithm: &CompressionAlgorithm) -> anyhow::Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Deflate => {
            let mut decoder = flate2::read::DeflateDecoder::new(data);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out).context("deflate io error")?;
            Ok(out)
        }
    }
}

fn decode_file(
    encryption_key: &AdeptEncryptionKey,
    data: &[u8],
    algorithm: &Algorithm,
) -> anyhow::Result<Vec<u8>> {
    let decrypted = decrypt_file(encryption_key, data, &algorithm.encryption)
        .context("failed to decrypt file")?;
    let decompressed =
        decompress_file(&decrypted, &algorithm.compression).context("failed to decompress file")?;
    Ok(decompressed)
}

struct EpubDecryptRebuilder {
    encryption_key: AdeptEncryptionKey,
    encrypted_files: HashMap<String, Algorithm>,
}

impl EpubDecryptRebuilder {
    const ENCRYPTION_FILEPATH: &str = "META-INF/encryption.xml";

    fn should_ship_file(&self, filename: &str) -> bool {
        filename == Self::ENCRYPTION_FILEPATH
    }
}

impl ZipRebuilder for EpubDecryptRebuilder {
    fn init<Z: ZipReader>(&mut self, zip: &mut Z) -> anyhow::Result<()> {
        let Some(encryption_file_contents) = zip.read_file(Self::ENCRYPTION_FILEPATH)? else {
            log::warn!(
                "No META-INF/encryption.xml file in the EPUB archive. Not decrypting anything. Maybe the EPUB is DRM-free?"
            );
            return Ok(());
        };

        let encryption_file_contents = String::from_utf8(encryption_file_contents)
            .ok()
            .context("encryption.xml file is not UTF8")?;

        let encrypted_data = parse_encryption_file(&encryption_file_contents)
            .context("could not parse encryption file")?;

        self.encrypted_files
            .extend(encrypted_data.into_iter().map(|x| (x.path, x.algorithm)));
        log::debug!("encrypted files: {:?}", self.encrypted_files);
        Ok(())
    }

    fn process_file<Z: ZipFileW>(&mut self, file: &mut Z) -> anyhow::Result<ZipFileDisposition> {
        if self.should_ship_file(file.name()) {
            log::debug!("skipping file: {}", file.name());
            return Ok(ZipFileDisposition::Delete);
        }

        let Some(algorithm) = self.encrypted_files.get(file.name()) else {
            log::debug!("copying raw file: {}", file.name());
            return Ok(ZipFileDisposition::Retain);
        };

        log::debug!(
            "decrypting file {} with algorithm {:?}",
            file.name(),
            algorithm
        );

        let data = file.read_file()?;
        let decoded_file = decode_file(&self.encryption_key, &data, algorithm)?;
        Ok(ZipFileDisposition::Modify(decoded_file))
    }
}

pub fn dedrm_epub<R: Read + Seek, W: Write + Seek>(
    input: R,
    output: W,
    encryption_key: AdeptEncryptionKey,
) -> anyhow::Result<W> {
    rebuild_zip(
        input,
        output,
        EpubDecryptRebuilder {
            encryption_key,
            encrypted_files: HashMap::new(),
        },
    )
    .context("could not rebuild zip")
}
