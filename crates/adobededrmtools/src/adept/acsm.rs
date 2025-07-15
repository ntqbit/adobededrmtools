use anyhow::Context;
use serde::Deserialize;

use super::xml::deserialize_xml;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "fulfillmentToken")]
struct FulfillmentToken {
    #[serde(rename = "operatorURL")]
    pub operator_url: String,
}

pub struct Acsm {
    parsed: FulfillmentToken,
    raw: String,
}

impl Acsm {
    pub fn from_string(s: String) -> anyhow::Result<Self> {
        let parsed = deserialize_xml(&s).context("failed to deserialize xml")?;
        Ok(Self { raw: s, parsed })
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        Self::from_string(s.to_string())
    }

    pub fn from_file(filepath: &str) -> anyhow::Result<Self> {
        Self::from_string(
            std::fs::read_to_string(filepath).context("could not read acsm from file")?,
        )
    }

    pub fn fulfillment_token(&self) -> &str {
        &self.raw
    }

    pub fn operator_url(&self) -> &str {
        &self.parsed.operator_url
    }
}
