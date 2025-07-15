use adobededrmtools_crypto::Signer;
use serde::{Deserialize, Serialize};

use super::ADEPT_XMLNS;
use super::http_client::HttpClient;
use super::request::{make_get, make_post};
use super::response::parse_response;
use super::signature::{compute_signature, impl_set_signature};

pub const DEFAULT_ACTIVATION_URL: &str = "https://adeactivate.adobe.com/adept";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "activationServiceInfo")]
pub struct ActivationServiceInfo {
    #[serde(rename = "authURL")]
    pub auth_url: String,
    #[serde(rename = "userInfoURL")]
    pub user_info_url: String,
    pub certificate: String,
}

pub async fn get_activation_service_info<H: HttpClient>(
    http_client: &H,
    activation_url: &str,
) -> anyhow::Result<ActivationServiceInfo> {
    let response = parse_response(
        http_client
            .request(make_get(activation_url, "/ActivationServiceInfo"))
            .await?,
    )?;

    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "authenticationServiceInfo")]
pub struct AuthenticationServiceInfo {
    #[serde(rename = "authURL")]
    pub auth_url: String,
    pub certificate: String,
    #[serde(rename = "signInMethods")]
    pub sign_in_methods: SignInMethods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInMethods {
    #[allow(dead_code)]
    #[serde(rename = "signInMethod")]
    pub sign_in_methods: Vec<SignInMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "signInMethod")]
pub struct SignInMethod {
    #[serde(rename = "@method")]
    pub method: String,
    #[serde(rename = "@type")]
    pub method_type: String,
    #[serde(rename = "$text")]
    pub name: String,
}

pub async fn get_authentication_service_info<H: HttpClient>(
    http_client: &H,
    authentication_url: &str,
) -> anyhow::Result<AuthenticationServiceInfo> {
    let response = parse_response(
        http_client
            .request(make_get(authentication_url, "/AuthenticationServiceInfo"))
            .await?,
    )?;

    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "adept:signIn")]
pub struct AdeptSignIn<'a> {
    #[serde(rename = "@xmlns:adept")]
    pub adept_xmlns: &'a str,
    #[serde(rename = "@method")]
    pub method: &'a str,

    #[serde(rename = "adept:signInData")]
    pub sign_in_data: String,
    #[serde(rename = "adept:publicAuthKey")]
    pub public_auth_key: String,
    #[serde(rename = "adept:encryptedPrivateAuthKey")]
    pub encrypted_private_auth_key: String,
    #[serde(rename = "adept:publicLicenseKey")]
    pub public_license_key: String,
    #[serde(rename = "adept:encryptedPrivateLicenseKey")]
    pub encrypted_private_license_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInData {
    #[serde(rename = "adept:signInData")]
    pub sign_in_data: String,
    #[serde(rename = "adept:publicAuthKey")]
    pub public_auth_key: String,
    #[serde(rename = "adept:encryptedPrivateAuthKey")]
    pub encrypted_private_auth_key: String,
    #[serde(rename = "adept:publicLicenseKey")]
    pub public_license_key: String,
    #[serde(rename = "adept:encryptedPrivateLicenseKey")]
    pub encrypted_private_license_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "credentials")]
pub struct Credentials {
    #[serde(rename = "user")]
    pub user: String,
    #[serde(rename = "pkcs12")]
    pub pkcs12: String,
    #[serde(rename = "encryptedPrivateLicenseKey")]
    pub encrypted_private_license_key: String,
    #[serde(rename = "licenseCertificate")]
    pub license_certificate: String,
}

pub async fn sign_in<H: HttpClient>(
    http_client: &H,
    authentication_url: &str,
    method: &str,
    data: SignInData,
) -> anyhow::Result<Credentials> {
    let req = AdeptSignIn {
        adept_xmlns: ADEPT_XMLNS,
        method,

        sign_in_data: data.sign_in_data,
        public_auth_key: data.public_auth_key,
        encrypted_private_auth_key: data.encrypted_private_auth_key,
        public_license_key: data.public_license_key,
        encrypted_private_license_key: data.encrypted_private_license_key,
    };

    let response = parse_response(
        http_client
            .request(make_post(authentication_url, "/SignInDirect", &req)?)
            .await?,
    )?;

    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "adept:activate")]
pub struct Activate {
    #[serde(rename = "@xmlns:adept")]
    pub adept_xmlns: &'static str,
    #[serde(rename = "@requestType")]
    pub request_type: String,
    #[serde(rename = "adept:fingerprint")]
    pub fingerprint: String,
    #[serde(rename = "adept:deviceType")]
    pub device_type: String,
    #[serde(rename = "adept:clientOS")]
    pub client_os: String,
    #[serde(rename = "adept:clientLocale")]
    pub client_locale: String,
    #[serde(rename = "adept:clientVersion")]
    pub client_version: String,
    #[serde(rename = "adept:targetDevice")]
    pub target_device: TargetDevice,
    #[serde(rename = "adept:nonce")]
    pub nonce: String,
    #[serde(rename = "adept:expiration")]
    pub expiration: String,
    #[serde(rename = "adept:user")]
    pub user: String,
    #[serde(rename = "adept:signature")]
    pub signature: Option<String>,
}

impl_set_signature!(Activate, signature);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDevice {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "activationToken")]
pub struct ActivationToken {
    pub device: String,
    pub fingerprint: String,
    #[serde(rename = "deviceType")]
    pub device_type: String,
    #[serde(rename = "activationURL")]
    pub activation_url: String,
    pub user: String,
    pub signature: String,
}

pub struct ActivateData {
    pub software_version: String,
    pub client_os: String,
    pub client_locale: String,
    pub client_version: String,
    pub device_type: String,
    pub fingerprint: String,
    pub nonce: String,
    pub expiration: String,
    pub user: String,
}

pub async fn activate<H: HttpClient>(
    http_client: &H,
    signer: &Signer,
    activation_url: &str,
    data: ActivateData,
) -> anyhow::Result<ActivationToken> {
    let req = compute_signature(
        signer,
        Activate {
            adept_xmlns: ADEPT_XMLNS,
            request_type: "initial".to_string(),
            fingerprint: data.fingerprint.clone(),
            device_type: data.device_type.clone(),
            client_os: data.client_os.clone(),
            client_locale: data.client_locale.clone(),
            client_version: data.client_version.clone(),
            target_device: TargetDevice {
                software_version: data.software_version,
                client_os: data.client_os,
                client_locale: data.client_locale,
                client_version: data.client_version,
                device_type: data.device_type,
                fingerprint: data.fingerprint,
            },
            nonce: data.nonce,
            expiration: data.expiration,
            user: data.user,
            signature: None,
        },
    )?;

    let response = parse_response(
        http_client
            .request(make_post(activation_url, "/Activate", &req)?)
            .await?,
    )?;

    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "adept:licenseServiceRequest")]
pub struct LicenseServiceRequest {
    #[serde(rename = "@xmlns:adept")]
    pub adept_xmlns: &'static str,
    #[serde(rename = "@identity")]
    pub identity: String,
    #[serde(rename = "adept:operatorURL")]
    pub operator_url: String,
    #[serde(rename = "adept:nonce")]
    pub nonce: String,
    #[serde(rename = "adept:expiration")]
    pub expiration: String,
    #[serde(rename = "adept:user")]
    pub user: String,
    #[serde(rename = "adept:signature")]
    pub signature: Option<String>,
}

impl_set_signature!(LicenseServiceRequest, signature);

pub struct InitLicenseService {
    pub operator_url: String,
    pub nonce: String,
    pub expiration: String,
    pub user: String,
}

pub async fn init_license_service<H: HttpClient>(
    http_client: &H,
    signer: &Signer,
    activation_url: &str,
    data: InitLicenseService,
) -> anyhow::Result<()> {
    let req = compute_signature(
        signer,
        LicenseServiceRequest {
            adept_xmlns: ADEPT_XMLNS,
            identity: "user".to_string(),
            operator_url: data.operator_url,
            nonce: data.nonce,
            expiration: data.expiration,
            user: data.user,
            signature: None,
        },
    )?;

    let response = parse_response(
        http_client
            .request(make_post(activation_url, "/InitLicenseService", &req)?)
            .await?,
    )?;

    Ok(response)
}
