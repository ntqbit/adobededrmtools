use adobededrmtools_crypto::Signer;
use serde::{Deserialize, Serialize};

use super::ADEPT_XMLNS;
use super::request::{make_post, make_post_serialized};
use super::response::parse_response;
use super::signature::{SetSignature, compute_signature_raw, impl_set_signature};
use super::{HttpClient, xml::serialize_xml};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "adept:credentials")]
pub struct FulfillmentCredentials {
    #[serde(rename = "@xmlns:adept")]
    pub adept_xmlns: &'static str,
    #[serde(rename = "adept:user")]
    pub user: String,
    #[serde(rename = "adept:certificate")]
    pub certificate: String,
    #[serde(rename = "adept:licenseCertificate")]
    pub license_certificate: String,
    #[serde(rename = "adept:authenticationCertificate")]
    pub authentication_certificate: String,
}

pub struct FulfillmentAuthData {
    pub user: String,
    pub certificate: String,
    pub license_certificate: String,
    pub authentication_certificate: String,
}

pub async fn fulfillment_auth<H: HttpClient>(
    http_client: &H,
    operator_url: &str,
    data: FulfillmentAuthData,
) -> anyhow::Result<()> {
    let req = FulfillmentCredentials {
        adept_xmlns: ADEPT_XMLNS,
        user: data.user,
        certificate: data.certificate,
        license_certificate: data.license_certificate,
        authentication_certificate: data.authentication_certificate,
    };

    let response = parse_response(
        http_client
            .request(make_post(operator_url, "/Auth", &req)?)
            .await?,
    )?;

    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "adept:fulfill")]
struct Fulfill {
    #[serde(rename = "@xmlns:adept")]
    pub adept_xmlns: &'static str,
    #[serde(rename = "adept:user")]
    pub user: String,
    #[serde(rename = "adept:device")]
    pub device: String,
    #[serde(rename = "adept:deviceType")]
    pub device_type: String,
    pub fulfillment_token_placeholder: (),
    #[serde(rename = "adept:targetDevice")]
    pub target_device: FulfillmentTargetDevice,
    #[serde(rename = "adept:signature")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentTargetDevice {
    #[serde(rename = "adept:softwareVersion")]
    pub software_version: String,
    #[serde(rename = "adept:clientOS")]
    pub client_os: String,
    #[serde(rename = "adept:clientLocale")]
    pub client_locale: String,
    #[serde(rename = "adept:clientVersion")]
    pub client_version: String,
    #[serde(rename = "adept:deviceType")]
    pub device_type: String,
    #[serde(rename = "adept:fingerprint")]
    pub fingerprint: String,
    #[serde(rename = "adept:activationToken")]
    pub activation_token: FulfillmentActivationToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "adept:activationToken")]
pub struct FulfillmentActivationToken {
    #[serde(rename = "adept:user")]
    pub user: String,
    #[serde(rename = "adept:device")]
    pub device: String,
}

impl_set_signature!(Fulfill, signature);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "envelope")]
pub struct Envelope {
    #[serde(rename = "fulfillmentResult")]
    pub fulfillmen_result: FulfillmentResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "fulfillmentResult")]
pub struct FulfillmentResult {
    pub fulfillment: String,
    pub returnable: bool,
    pub initial: bool,
    pub notify: Vec<FulfillmentNotify>,
    #[serde(rename = "resourceItemInfo")]
    pub resources: Vec<ResourceItemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "resourceItemInfo")]
pub struct ResourceItemInfo {
    pub resource: String,
    #[serde(rename = "resourceItem")]
    pub resource_item: u32,
    pub metadata: (),
    pub src: String,
    #[serde(rename = "downloadType")]
    pub download_type: String,
    #[serde(rename = "licenseToken")]
    pub license_token: LicenseToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "licenseToken")]
pub struct LicenseToken {
    pub user: String,
    pub resource: String,
    #[serde(rename = "resourceItemType")]
    pub resource_item_type: String,
    #[serde(rename = "deviceType")]
    pub device_type: String,
    pub device: String,
    pub voucher: String,
    #[serde(rename = "licenseURL")]
    pub license_url: String,
    #[serde(rename = "operatorURL")]
    pub operator_url: String,
    pub fulfillment: String,
    pub distributor: String,
    #[serde(rename = "encryptedKey")]
    pub encrypted_key: EncryptedKey,
    pub model: String,
    pub permissions: (),
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "licenseToken")]
pub struct EncryptedKey {
    #[serde(rename = "@keyInfo")]
    pub key_info: String,
    #[serde(rename = "$text")]
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "notify")]
pub struct FulfillmentNotify {
    #[serde(rename = "@critical")]
    pub critical: Option<String>,
    #[serde(rename = "notifyURL")]
    pub notify_url: String,
}

pub struct FulfillmentData {
    pub user: String,
    pub device: String,
    pub software_version: String,
    pub client_os: String,
    pub client_locale: String,
    pub client_version: String,
    pub device_type: String,
    pub fingerprint: String,
    pub fulfillment_token: String,
}

pub async fn fulfill<H: HttpClient>(
    http_client: &H,
    signer: &Signer,
    operator_url: &str,
    data: FulfillmentData,
) -> anyhow::Result<Envelope> {
    let mut raw_req = Fulfill {
        adept_xmlns: ADEPT_XMLNS,
        user: data.user.clone(),
        device: data.device.clone(),
        device_type: data.device_type.clone(),
        target_device: FulfillmentTargetDevice {
            software_version: data.software_version,
            client_os: data.client_os,
            client_locale: data.client_locale,
            client_version: data.client_version,
            device_type: data.device_type,
            fingerprint: data.fingerprint,
            activation_token: FulfillmentActivationToken {
                user: data.user,
                device: data.device,
            },
        },
        fulfillment_token_placeholder: (),
        signature: None,
    };

    let serialized_raw =
        substitute_fulfillment_token(&serialize_xml(&raw_req)?, &data.fulfillment_token);

    let signature = compute_signature_raw(signer, &serialized_raw)?;
    raw_req.set_signature(signature);

    let serialized =
        substitute_fulfillment_token(&serialize_xml(&raw_req)?, &data.fulfillment_token);

    let response = parse_response(
        http_client
            .request(make_post_serialized(operator_url, "/Fulfill", &serialized)?)
            .await?,
    )?;

    Ok(response)
}

fn substitute_fulfillment_token(s: &str, token: &str) -> String {
    const PLACEHOLDER_ELEMENT: &str = "<fulfillment_token_placeholder/>";
    s.replacen(PLACEHOLDER_ELEMENT, token, 1)
}
