use serde::{Serialize, de::DeserializeOwned};

pub fn serialize_xml<T: Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(quick_xml::se::to_string(value)?)
}

pub fn deserialize_xml<T: DeserializeOwned>(s: &str) -> anyhow::Result<T> {
    Ok(quick_xml::de::from_str(s)?)
}
