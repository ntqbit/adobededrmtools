use base64::{Engine, engine::general_purpose::STANDARD};

pub fn b64(v: &[u8]) -> String {
    STANDARD.encode(v)
}

pub fn unb64(v: &str) -> anyhow::Result<Vec<u8>> {
    Ok(STANDARD.decode(v)?)
}
