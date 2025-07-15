use crate::adept::ResourceItemInfo;

use super::{DeviceInfo, HttpClient, UserCredentials, adept, make_expiration, random_nonce};

use adobededrmtools_crypto::{Signer, b64, unb64};
use anyhow::Context;

pub async fn fulfillment_auth<H: HttpClient>(
    http_client: &H,
    acsm: &adept::Acsm,
    credentials: &UserCredentials,
    auth_certificate: &[u8],
) -> anyhow::Result<()> {
    adept::fulfillment_auth(
        http_client,
        acsm.operator_url(),
        adept::FulfillmentAuthData {
            user: credentials.user.clone(),
            certificate: b64(&credentials.user_certificate),
            license_certificate: b64(&credentials.license_certificate),
            authentication_certificate: b64(auth_certificate),
        },
    )
    .await
    .context("fulfillment auth failed")?;

    Ok(())
}

pub async fn init_license_service<H: HttpClient>(
    http_client: &H,
    signer: &Signer,
    activation_url: &str,
    user: &str,
    operator_url: &str,
) -> anyhow::Result<()> {
    adept::init_license_service(
        http_client,
        signer,
        activation_url,
        adept::InitLicenseService {
            operator_url: operator_url.to_string(),
            nonce: random_nonce(),
            expiration: make_expiration(),
            user: user.to_string(),
        },
    )
    .await
    .context("init_license_service failed")?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub resource: String,
    pub item_type: String,
    pub encrypted_key: ResourceEncryptedKey,
    pub download: DownloadInfo,
}

#[derive(Debug, Clone)]
pub struct ResourceEncryptedKey {
    pub encrypted_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum DownloadInfo {
    Simple(String),
}

#[derive(Debug, Clone)]
pub struct FulfillmentResult {
    pub resources: Vec<Resource>,
}

pub async fn fulfill<H: HttpClient>(
    http_client: &H,
    signer: &Signer,
    acsm: &adept::Acsm,
    credentials: &UserCredentials,
    device_info: &DeviceInfo,
    activated_device: &str,
) -> anyhow::Result<FulfillmentResult> {
    let envelope = adept::fulfill(
        http_client,
        signer,
        acsm.operator_url(),
        adept::FulfillmentData {
            user: credentials.user.clone(),
            device: activated_device.to_string(),
            software_version: device_info.software_version.clone(),
            client_os: device_info.client_os.clone(),
            client_locale: device_info.client_locale.clone(),
            client_version: device_info.client_version.clone(),
            device_type: device_info.device_type.clone(),
            fingerprint: device_info.fingerprint.clone(),
            fulfillment_token: acsm.fulfillment_token().to_string(),
        },
    )
    .await
    .context("fulfill request failed")?;

    log::debug!("envelope: {:?}", envelope);

    Ok(FulfillmentResult {
        resources: envelope
            .fulfillmen_result
            .resources
            .into_iter()
            .map(|resource| convert_resource(resource))
            .collect::<anyhow::Result<Vec<_>>>()?,
    })
}

fn convert_resource(item: ResourceItemInfo) -> anyhow::Result<Resource> {
    let download = match item.download_type.as_str() {
        "simple" => DownloadInfo::Simple(item.src),
        _ => {
            return Err(anyhow::anyhow!(
                "unsupported download type: {}",
                item.download_type
            ));
        }
    };

    let encrypted_key = ResourceEncryptedKey {
        encrypted_key: unb64(&item.license_token.encrypted_key.key)?,
    };

    Ok(Resource {
        resource: item.resource,
        item_type: item.license_token.resource_item_type,
        encrypted_key,
        download,
    })
}
