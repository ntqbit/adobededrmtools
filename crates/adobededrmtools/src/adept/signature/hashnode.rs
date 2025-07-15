use xmltree::Element;

// Reimplementation of XML node hashing scheme from
// https://forge.soutade.fr/soutade/libgourou/src/commit/d3c90f03bba187292c747080592840123f94f285/src/libgourou.cpp#L120

#[repr(u8)]
pub enum AsnTag {
    #[allow(dead_code)]
    None = 0x00,
    NsTag = 0x01,
    Child = 0x02,
    EndTag = 0x03,
    Text = 0x04,
    Attribute = 0x05,
}

pub trait Updater {
    fn update(&mut self, data: &[u8]);
}

pub trait Hasher {
    fn push_tag(&mut self, tag: AsnTag);

    fn push_string(&mut self, s: &str);
}

impl<U: Updater> Hasher for U {
    fn push_tag(&mut self, tag: AsnTag) {
        self.update(&[tag as u8]);
    }

    fn push_string(&mut self, s: &str) {
        self.update(&(s.len() as u16).to_be_bytes());
        self.update(s.as_bytes());
    }
}

pub fn hash_xml<H: Hasher>(hasher: &mut H, xml: &str) -> anyhow::Result<()> {
    let doc = Element::parse(xml.as_bytes())?;
    hash_element_inner(hasher, &HasherElement(&doc));
    Ok(())
}

enum HasherNode<'a> {
    Element(HasherElement<'a>),
    Text(&'a str),
}

struct HasherElement<'a>(&'a Element);

impl<'a> HasherElement<'a> {
    pub fn ns(&self) -> Option<&str> {
        self.0.namespace.as_ref().map(|x| x.as_str())
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn attributes(&self) -> impl Iterator<Item = Attribute> {
        self.0
            .attributes
            .iter()
            .map(|(k, v)| Attribute { name: k, value: v })
    }

    pub fn children(&self) -> impl Iterator<Item = HasherNode> {
        self.0.children.iter().filter_map(|node| match node {
            xmltree::XMLNode::Element(element) => Some(HasherNode::Element(HasherElement(element))),
            xmltree::XMLNode::Text(text) => Some(HasherNode::Text(text)),
            _ => None,
        })
    }
}

struct Attribute<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> Attribute<'a> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn value(&self) -> &str {
        self.value
    }
}

fn should_process_element(element: &HasherElement) -> bool {
    const BLACKLIST: &[&str] = &["hmac", "signature"];
    !BLACKLIST.iter().any(|&v| v == element.name())
}

fn hash_element_inner<H: Hasher>(hasher: &mut H, element: &HasherElement) {
    if !should_process_element(element) {
        return;
    }

    // Push namespace
    if let Some(ns) = element.ns() {
        hasher.push_tag(AsnTag::NsTag);
        hasher.push_string(ns);
    }

    // Push name
    hasher.push_string(element.name());

    // Push attributes
    let mut attrs: Vec<Attribute> = element
        .attributes()
        .into_iter()
        .filter(|attr| !attr.name().starts_with("xmlns"))
        .collect();
    attrs.sort_by(|a, b| a.name().cmp(b.name()));

    for attr in attrs {
        hasher.push_tag(AsnTag::Attribute);
        hasher.push_string("");

        hasher.push_string(attr.name());
        hasher.push_string(attr.value());
    }

    hasher.push_tag(AsnTag::Child);

    for child in element.children() {
        hash_node_inner(hasher, &child);
    }

    hasher.push_tag(AsnTag::EndTag);
}

fn hash_node_inner<H: Hasher>(hasher: &mut H, node: &HasherNode) {
    match node {
        HasherNode::Element(element) => hash_element_inner(hasher, element),
        HasherNode::Text(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                hasher.push_tag(AsnTag::Text);
                hasher.push_string(trimmed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sha1::Digest;

    use super::hash_xml;

    struct Sha1Hasher(sha1::Sha1);

    impl Sha1Hasher {
        pub fn new() -> Self {
            Self(sha1::Sha1::new())
        }

        pub fn digest(self) -> [u8; 20] {
            self.0.finalize().into()
        }
    }

    impl super::Updater for Sha1Hasher {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    #[test]
    fn test_hash_xml() {
        const TEST_CASES: &[(&str, &str)] = &[
            (
                r#"<?xml version="1.0"?>
<adept:activate xmlns:adept="http://ns.adobe.com/adept" requestType="initial">
  <adept:fingerprint>xsXngUfahHAHQpv8brLlYMFbpNk=</adept:fingerprint>
  <adept:deviceType>standalone</adept:deviceType>
  <adept:clientOS>Linux 6.15.6-arch1-1</adept:clientOS>
  <adept:clientLocale>C</adept:clientLocale>
  <adept:clientVersion>Desktop</adept:clientVersion>
  <adept:targetDevice>
    <adept:softwareVersion>10.0.4</adept:softwareVersion>
    <adept:clientOS>Linux 6.15.6-arch1-1</adept:clientOS>
    <adept:clientLocale>C</adept:clientLocale>
    <adept:clientVersion>Desktop</adept:clientVersion>
    <adept:deviceType>standalone</adept:deviceType>
    <adept:fingerprint>xsXngUfahHAHQpv8brLlYMFbpNk=</adept:fingerprint>
  </adept:targetDevice>
  <adept:nonce>j+ePeCI6AAAAAAAA</adept:nonce>
  <adept:expiration>2025-07-14T15:36:35Z</adept:expiration>
  <adept:user>urn:uuid:e9fb5f93-8f17-4b45-b564-c8de69a4051b</adept:user>
</adept:activate>
"#,
                "1ab9a7543c085dbd75cacfbc87c1b93c7e323e6a",
            ),
            (
                r#"<?xml version="1.0"?>
<adept:fulfill xmlns:adept="http://ns.adobe.com/adept">
  <adept:user>urn:uuid:52176b2b-fbdf-40f0-90b4-005c381806bc</adept:user>
  <adept:device>urn:uuid:a310b35a-512e-4054-8a95-7b7288b95f78</adept:device>
  <adept:deviceType>standalone</adept:deviceType>
  <fulfillmentToken fulfillmentType="buy" auth="user" xmlns="http://ns.adobe.com/adept">
    <distributor>urn:uuid:a5fac67c-03f8-43af-94d1-fb894365054d</distributor>
    <operatorURL>dummy</operatorURL>
    <transaction>61777-38641</transaction>
    <purchase>2025-07-13T15:49:52+03:00</purchase>
    <expiration>2025-07-16T15:49:52+03:00</expiration>
    <resourceItemInfo>
      <resource>urn:uuid:5af67d43-61b7-44f0-b827-e41594a40484</resource>
      <resourceItem>1</resourceItem>
      <metadata>
        <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">ΤΟΥ</dc:title>
        <dc:creator xmlns:dc="http://purl.org/dc/elements/1.1/">dummy</dc:creator>
        <dc:publisher xmlns:dc="http://purl.org/dc/elements/1.1/">dummy</dc:publisher>
        <dc:identifier xmlns:dc="http://purl.org/dc/elements/1.1/">dummy</dc:identifier>
        <dc:format xmlns:dc="http://purl.org/dc/elements/1.1/">application/epub+zip</dc:format>
        <dc:language xmlns:dc="http://purl.org/dc/elements/1.1/">el</dc:language>
      </metadata>
      <licenseToken>
        <resource>urn:uuid:5af67d43-61b7-44f0-b827-e41594a40484</resource>
        <permissions>
          <display />
          <excerpt />
          <print />
          <play />
        </permissions>
      </licenseToken>
    </resourceItemInfo>
    <hmac>iFEK7MgV0vZDHfAq9TbD6db8U8M=</hmac>
  </fulfillmentToken>
  <adept:targetDevice>
    <adept:softwareVersion>10.0.4</adept:softwareVersion>
    <adept:clientOS>Linux 6.15.6-arch1-1</adept:clientOS>
    <adept:clientLocale>C</adept:clientLocale>
    <adept:clientVersion>Desktop</adept:clientVersion>
    <adept:deviceType>standalone</adept:deviceType>
    <adept:fingerprint>kjXZLt1DmCGG6WU6YauHLNecTD8=</adept:fingerprint>
    <adept:activationToken>
      <adept:user>urn:uuid:52176b2b-fbdf-40f0-90b4-005c381806bc</adept:user>
      <adept:device>urn:uuid:a310b35a-512e-4054-8a95-7b7288b95f78</adept:device>
    </adept:activationToken>
  </adept:targetDevice>
  <adept:signature>c/ZHjn/YF3N2KPEkXZVB6okfqi4g56kWCCHsidi9oHotHkXe5pjDOYj8/GFcJ2krEoIhmdFJ9rCMH8fHzGuaUCvciPAxh1fNSEQq29iNDr+/h17vFT0Es1g3P/IC6xA6P5pIRcuuMTnWuRRD1kjFKLXsDfQWq0WwjdVqBrabemc=</adept:signature>
</adept:fulfill>"#,
                "32d5c35172f4ac65c6e63f9a88d97c1c70b1eb07",
            ),
        ];

        for &(xml, hash) in TEST_CASES {
            let mut hasher = Sha1Hasher::new();
            hash_xml(&mut hasher, xml).expect("hash_xml failed");
            assert_eq!(hex::encode(hasher.digest()), hash, "hash mismatch");
        }
    }
}
