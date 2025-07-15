use adobededrmtools_crypto::{Sha1, Signer};
use serde::Serialize;

mod hashnode;

use super::xml::serialize_xml;

pub trait SetSignature {
    fn set_signature(&mut self, signature: String);
}

pub fn compute_signature<T>(signer: &Signer, mut value: T) -> anyhow::Result<T>
where
    T: SetSignature + Serialize,
{
    let serialized = serialize_xml(&value)?;
    let signature = compute_signature_raw(signer, &serialized)?;
    value.set_signature(signature);
    Ok(value)
}

pub struct Sha1Hasher(Sha1);
impl hashnode::Updater for Sha1Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

pub fn compute_signature_raw(signer: &Signer, serialized: &str) -> anyhow::Result<String> {
    let mut hasher = Sha1Hasher(Sha1::new());
    hashnode::hash_xml(&mut hasher, serialized)?;
    Ok(signer.sign(&hasher.0.finalize()))
}

macro_rules! impl_set_signature {
    ($typ:ty, $signature:ident) => {
        impl $crate::adept::signature::SetSignature for $typ {
            fn set_signature(&mut self, signature: String) {
                self.$signature.replace(signature);
            }
        }
    };
}

pub(crate) use impl_set_signature;
