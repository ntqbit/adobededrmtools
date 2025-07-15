use anyhow::Context;

use super::{Pkey, b64};

pub struct Signer(Pkey);

impl Signer {
    pub fn new(pkey: Pkey) -> Self {
        Self(pkey)
    }

    pub fn sign(&self, data: &[u8]) -> String {
        b64(&self.0.sign(data))
    }
}

pub fn make_signer(private_auth_key: &[u8]) -> anyhow::Result<Signer> {
    let pkey = Pkey::from_der(private_auth_key).context("could not construct user pkey")?;
    Ok(Signer::new(pkey))
}
