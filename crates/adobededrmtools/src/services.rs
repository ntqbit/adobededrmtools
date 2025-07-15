use super::{HttpClient, adept};
use adobededrmtools_crypto::unb64;
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdobeServicesInfo {
    pub activation_url: String,
    pub auth_service: AdobeAuthServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdobeAuthServiceInfo {
    pub auth_url: String,
    pub auth_certificate: Vec<u8>,
    pub sign_in_methods: Vec<String>,
}

pub async fn get_services_info<H: HttpClient>(
    http_client: &H,
    activation_url: &str,
) -> anyhow::Result<AdobeServicesInfo> {
    // Activation service info
    let asi = adept::get_activation_service_info(http_client, activation_url)
        .await
        .context("get_activation_service_info failed")?;

    log::debug!("activation service info: {:?}", asi);

    let auth_url = asi.auth_url;

    // Authentication service info
    let auth = adept::get_authentication_service_info(http_client, &auth_url)
        .await
        .context("get_authentication_service_info failed")?;

    log::debug!("auth: {:?}", auth);
    Ok(AdobeServicesInfo {
        activation_url: activation_url.to_string(),
        auth_service: AdobeAuthServiceInfo {
            auth_url,
            auth_certificate: unb64(&auth.certificate)?,
            sign_in_methods: auth
                .sign_in_methods
                .sign_in_methods
                .into_iter()
                .map(|v| v.method)
                .collect(),
        },
    })
}
