use super::rand_bytes;

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use anyhow::Context;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

fn aes128_cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let encryptor = Aes128CbcEnc::new(key.into(), iv.into());
    encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}

fn aes128_cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let decryptor = Aes128CbcDec::new(key.into(), iv.into());
    Ok(decryptor.decrypt_padded_vec_mut::<Pkcs7>(ciphertext)?)
}

const IV_LEN: usize = 16;

fn generate_iv() -> [u8; IV_LEN] {
    rand_bytes()
}

pub fn encrypt_aes(key: &[u8; 16], data: &[u8]) -> Vec<u8> {
    let iv: [u8; IV_LEN] = generate_iv();
    let encrypted = aes128_cbc_encrypt(key, &iv, data);

    let mut output = Vec::new();
    output.extend_from_slice(&iv);
    output.extend_from_slice(&encrypted);
    output
}

pub fn decrypt_aes(key: &[u8; 16], data: &[u8]) -> anyhow::Result<Vec<u8>> {
    if data.len() < IV_LEN {
        return Err(anyhow::anyhow!("ciphertext is too short: {}", data.len()));
    }

    let iv = &data[..IV_LEN];
    let ciphertext = &data[IV_LEN..];
    let decrypted = aes128_cbc_decrypt(key, iv, ciphertext).context("could not decrypt AES128")?;
    Ok(decrypted)
}
