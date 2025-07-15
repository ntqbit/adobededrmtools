use adobededrmtools_crypto::Pkey;

/// Adobe DRM-protected file content encryption key.
/// Obtain from [`decrypt_adept_encryption_key`].
#[derive(Clone, Copy)]
pub struct AdeptEncryptionKey([u8; 16]);

impl AdeptEncryptionKey {
    pub fn from_raw(raw: [u8; 16]) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> [u8; 16] {
        self.0
    }
}

pub fn decrypt_adept_encryption_key(
    encrypted_key: &[u8],
    private_license_key: &[u8],
) -> anyhow::Result<AdeptEncryptionKey> {
    let pkey = Pkey::from_der(private_license_key)?;
    let mut decrypted_key: [u8; 16] = [0; 16];
    let decrypted = pkey.decrypt(encrypted_key)?;

    if decrypted.len() != 16 {
        return Err(anyhow::anyhow!(
            "decrypted key length is different from 16: {}",
            decrypted.len()
        ));
    }

    decrypted_key.copy_from_slice(&decrypted);
    Ok(AdeptEncryptionKey(decrypted_key))
}
