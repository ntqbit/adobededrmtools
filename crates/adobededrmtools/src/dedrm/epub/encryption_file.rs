use serde::Deserialize;

// Example of META-INF/encryption.xml file contents:
// <?xml version="1.0" encoding="UTF-8" standalone="no"?><encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
//   <EncryptedData xmlns="http://www.w3.org/2001/04/xmlenc#">
//     <EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes128-cbc"/>
//     <KeyInfo xmlns="http://www.w3.org/2000/09/xmldsig#">
//       <resource xmlns="http://ns.adobe.com/adept">urn:uuid:a37ec574-e73a-4d2c-a41a-72fa9904abdc</resource>
//     </KeyInfo>
//     <CipherData>
//       <CipherReference URI="OEBPS/Fonts/ub-helbi.otf"/>
//     </CipherData>
//   </EncryptedData>

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Algorithm {
    pub encryption: EncryptionAlgorithm,
    pub compression: CompressionAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    Aes128Cbc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Deflate,
}

pub struct EncryptedData {
    pub path: String,
    pub algorithm: Algorithm,
}

#[derive(Deserialize)]
#[serde(rename = "encryption")]
struct Encryption {
    #[serde(rename = "EncryptedData")]
    encrypted_data: Vec<InnerEncryptedData>,
}

#[derive(Deserialize)]
struct InnerEncryptedData {
    #[serde(rename = "EncryptionMethod")]
    encryption_method: EncryptionMethod,
    #[serde(rename = "CipherData")]
    cipher_data: CipherData,
}

#[derive(Deserialize)]
struct EncryptionMethod {
    #[serde(rename = "@Algorithm")]
    algorithm: String,
}

#[derive(Deserialize)]
struct CipherData {
    #[serde(rename = "CipherReference")]
    reference: CipherReference,
}

#[derive(Deserialize)]
struct CipherReference {
    #[serde(rename = "@URI")]
    uri: String,
}

pub fn parse_encryption_file(s: &str) -> anyhow::Result<Vec<EncryptedData>> {
    let encryption: Encryption = quick_xml::de::from_str(s)?;
    Ok(encryption
        .encrypted_data
        .into_iter()
        .map(|d| {
            let algorithm = match d.encryption_method.algorithm.as_str() {
                "http://www.w3.org/2001/04/xmlenc#aes128-cbc" => Algorithm {
                    encryption: EncryptionAlgorithm::Aes128Cbc,
                    compression: CompressionAlgorithm::Deflate,
                },
                "http://ns.adobe.com/adept/xmlenc#aes128-cbc-uncompressed" => Algorithm {
                    encryption: EncryptionAlgorithm::Aes128Cbc,
                    compression: CompressionAlgorithm::None,
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "unsupported encryption algorithm: {}",
                        d.encryption_method.algorithm
                    ));
                }
            };

            Ok(EncryptedData {
                path: d.cipher_data.reference.uri,
                algorithm,
            })
        })
        .collect::<Result<Vec<_>, _>>()?)
}
