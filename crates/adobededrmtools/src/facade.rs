use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::UserCredentials;
use crate::serializarion::serde_base64;

use super::{
    Acsm, DEFAULT_ACTIVATION_URL, HttpClient,
    activation::{DeviceInfo, activate_device},
    auth::{SignInMethod, sign_in},
    fulfillment::{Resource, fulfill, fulfillment_auth, init_license_service},
    make_signer,
    services::get_services_info,
};

pub struct CreateAccountParams {
    pub activation_url: String,
    pub device_info: DeviceInfo,
}

impl Default for CreateAccountParams {
    fn default() -> Self {
        Self {
            activation_url: DEFAULT_ACTIVATION_URL.to_string(),
            device_info: DeviceInfo::generate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdobeMinServicesInfo {
    pub activation_url: String,
    pub auth_url: String,
    #[serde(with = "serde_base64")]
    pub auth_certificate: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdobeAccount {
    pub services: AdobeMinServicesInfo,
    pub user_credentials: UserCredentials,
    pub device_info: DeviceInfo,
    pub activated_device: String,
}

pub async fn create_adobe_account<H: HttpClient>(
    http_client: &H,
    params: CreateAccountParams,
) -> anyhow::Result<AdobeAccount> {
    let services = get_services_info(http_client, &params.activation_url)
        .await
        .context("get_services_info failed")?;

    let user_credentials = sign_in(http_client, &services.auth_service, SignInMethod::Anonymous)
        .await
        .context("sign_in failed")?;

    let signer = make_signer(&user_credentials.private_auth_key)?;
    let device_info = params.device_info;

    let activated_device = activate_device(
        http_client,
        &signer,
        &services.activation_url,
        &user_credentials.user,
        &device_info,
    )
    .await
    .context("activate_device failed")?;

    Ok(AdobeAccount {
        services: AdobeMinServicesInfo {
            activation_url: services.activation_url,
            auth_url: services.auth_service.auth_url,
            auth_certificate: services.auth_service.auth_certificate,
        },
        user_credentials,
        device_info,
        activated_device: activated_device.device,
    })
}

pub async fn fulfill_acsm<H: HttpClient>(
    http_client: &H,
    acsm: &Acsm,
    account: &AdobeAccount,
) -> anyhow::Result<Vec<Resource>> {
    fulfillment_auth(
        http_client,
        acsm,
        &account.user_credentials,
        &account.services.auth_certificate,
    )
    .await?;

    let signer = make_signer(&account.user_credentials.private_auth_key)?;

    init_license_service(
        http_client,
        &signer,
        &account.services.activation_url,
        &account.user_credentials.user,
        acsm.operator_url(),
    )
    .await?;

    let result = fulfill(
        http_client,
        &signer,
        &acsm,
        &account.user_credentials,
        &account.device_info,
        &account.activated_device,
    )
    .await
    .context("fulfill failed")?;

    Ok(result.resources)
}
