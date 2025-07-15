use anyhow::Context;

pub struct ParsedPkcs12 {
    pub pkey: Vec<u8>,
    pub cert: Vec<u8>,
}

pub fn parse_pkcs12(pkcs12: &[u8], password: &str) -> anyhow::Result<ParsedPkcs12> {
    let ks = p12_keystore::KeyStore::from_pkcs12(pkcs12, password)
        .context("could not parse pkcs12 keystore")?;

    let (_, chain) = ks
        .private_key_chain()
        .context("no private key chain in pkcs12 keystore")?;

    let key = chain.key();
    if chain.chain().len() != 1 {
        return Err(anyhow::anyhow!(
            "expected chain of length 1 in pkcs12 keystore"
        ));
    }

    let cert = &chain.chain()[0];

    Ok(ParsedPkcs12 {
        pkey: key.to_vec(),
        cert: cert.as_der().to_vec(),
    })
}
