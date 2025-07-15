pub mod serde_base64 {
    use adobededrmtools_crypto::{b64, unb64};
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S>(bytes: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if s.is_human_readable() {
            s.serialize_str(&b64(bytes))
        } else {
            s.serialize_bytes(bytes)
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if d.is_human_readable() {
            let s: String = Deserialize::deserialize(d)?;
            unb64(&s).map_err(|_| D::Error::custom("could not decode base64"))
        } else {
            let buf = serde_bytes::ByteBuf::deserialize(d)?;
            Ok(buf.into_vec())
        }
    }
}
