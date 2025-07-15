use super::{HttpClient, adept, make_expiration, random_nonce};
use adobededrmtools_crypto::{Signer, b64, rand_bytes};
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub software_version: String,
    pub client_os: String,
    pub client_locale: String,
    pub client_version: String,
    pub device_type: String,
    pub fingerprint: String,
}

impl DeviceInfo {
    pub fn generate() -> Self {
        Self {
            software_version: "10.0.4".to_string(),
            client_os: "Linux".to_string(),
            client_locale: "C".to_string(),
            client_version: "Desktop".to_string(),
            device_type: "standalone".to_string(),
            fingerprint: b64(&rand_bytes::<20>()),
        }
    }
}

#[derive(Debug)]
pub struct ActivatedDevice {
    pub device: String,
}

pub async fn activate_device<H: HttpClient>(
    http_client: &H,
    signer: &Signer,
    activation_url: &str,
    user: &str,
    device_info: &DeviceInfo,
) -> anyhow::Result<ActivatedDevice> {
    // Activate account
    let activation_token = adept::activate(
        http_client,
        signer,
        &activation_url,
        adept::ActivateData {
            software_version: device_info.software_version.clone(),
            client_os: device_info.client_os.clone(),
            client_locale: device_info.client_locale.clone(),
            client_version: device_info.client_version.clone(),
            device_type: device_info.device_type.clone(),
            fingerprint: device_info.fingerprint.clone(),
            nonce: random_nonce(),
            expiration: make_expiration(),
            user: user.to_string(),
        },
    )
    .await
    .context("activate failed")?;

    log::debug!("activation token: {:?}", activation_token);
    Ok(ActivatedDevice {
        device: activation_token.device,
    })
}
