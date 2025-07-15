use std::io::Cursor;

use anyhow::Context;

mod encryption_key;
pub mod epub;

pub use encryption_key::{AdeptEncryptionKey, decrypt_adept_encryption_key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Epub,
    // TODO: add support for PDF
}

impl ResourceType {
    pub fn from_item_type(item_type: &str) -> Option<Self> {
        match item_type {
            "application/epub+zip" => Some(Self::Epub),
            _ => None,
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ResourceType::Epub => "epub",
        }
    }
}

pub fn dedrm_resource(
    resource_type: ResourceType,
    encrypted_key: &[u8],
    private_license_key: &[u8],
    encrypted_resource: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let encryption_key = decrypt_adept_encryption_key(encrypted_key, private_license_key)
        .context("could not decrypt adept encryption key")?;

    match resource_type {
        ResourceType::Epub => Ok(dedrm_epub_resource(encryption_key, encrypted_resource)?),
    }
}

pub fn dedrm_epub_resource(
    encryption_key: AdeptEncryptionKey,
    encrypted_resource: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let decrypted_resource = epub::dedrm_epub(
        Cursor::new(encrypted_resource),
        Cursor::new(Vec::new()),
        encryption_key,
    )
    .context("could not decrypt epub")?
    .into_inner();

    Ok(decrypted_resource)
}
