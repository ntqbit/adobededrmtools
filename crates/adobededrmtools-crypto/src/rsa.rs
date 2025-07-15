use anyhow::Context;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use x509_cert::der::Decode;

use super::rand::rng;

pub fn make_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut rng = rng();
    let privkey =
        rsa::RsaPrivateKey::new(&mut rng, 1024).expect("could not generate RSA private key");
    let pubkey = privkey.to_public_key();
    let pubkey_der = pubkey
        .to_public_key_der()
        .expect("could not convert RSA public key to DER")
        .to_vec();
    let privkey_der = privkey
        .to_pkcs8_der()
        .expect("could not convert RSA private key to DER")
        .as_bytes()
        .to_vec();
    (pubkey_der, privkey_der)
}

pub fn encrypt_with_cert(cert_der: &[u8], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cert = x509_cert::certificate::Certificate::from_der(cert_der)
        .context("could not parse X.509 certificate from DER")?;

    let pubkey = rsa::RsaPublicKey::from_pkcs1_der(
        cert.tbs_certificate
            .subject_public_key_info
            .subject_public_key
            .raw_bytes(),
    )
    .ok()
    .context("could not parse RSA public key from DER")?;

    let mut rng = rng();
    Ok(pubkey
        .encrypt(&mut rng, rsa::Pkcs1v15Encrypt, plaintext)
        .ok()
        .context("could not encrypt RSA")?)
}
