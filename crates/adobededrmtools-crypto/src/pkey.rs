use super::rand::rng;
use anyhow::Context;
use rsa::pkcs8::DecodePrivateKey;

#[derive(Clone)]
pub struct Pkey(rsa::RsaPrivateKey);

impl Pkey {
    pub fn from_der(pkey_der: &[u8]) -> anyhow::Result<Self> {
        Ok(Self(
            rsa::RsaPrivateKey::from_pkcs8_der(pkey_der)
                .context("could not parse rsa pkey from pkcs8 der")?,
        ))
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = rng();

        self.0
            .sign_with_rng(&mut rng, rsa::Pkcs1v15Sign::new_unprefixed(), data)
            .expect("could not sign rsa")
    }

    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        Ok(self
            .0
            .decrypt(rsa::Pkcs1v15Encrypt, data)
            .context("could not decrypt rsa")?)
    }
}
