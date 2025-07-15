use super::{HttpClient, adept, services::AdobeAuthServiceInfo};
use crate::serializarion::serde_base64;
use adobededrmtools_crypto::{
    b64, decrypt_aes, encrypt_aes, encrypt_with_cert, make_keypair, parse_pkcs12, rand_bytes, unb64,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

struct EphemeralKey([u8; 16]);

impl EphemeralKey {
    pub fn from_bytes(v: [u8; 16]) -> Self {
        Self(v)
    }

    pub fn generate() -> Self {
        Self::from_bytes(rand_bytes())
    }

    pub fn raw(&self) -> &[u8; 16] {
        &self.0
    }
}

#[derive(Debug)]
struct SignInCredentials<'a> {
    username: &'a str,
    password: &'a str,
}

fn serialize_signin_credentials(key: &EphemeralKey, username: &str, password: &str) -> Vec<u8> {
    let mut data = Vec::new();

    data.extend_from_slice(&key.0);

    data.extend_from_slice(&[username.len() as u8]);
    data.extend_from_slice(username.as_bytes());

    data.extend_from_slice(&[password.len() as u8]);
    data.extend_from_slice(password.as_bytes());

    data
}

struct PrivateKeys {
    private_auth_key: Vec<u8>,
    private_license_key: Vec<u8>,
}

fn make_sign_in_data(
    key: &EphemeralKey,
    auth_certificate: &[u8],
    credentials: SignInCredentials,
) -> anyhow::Result<(adept::SignInData, PrivateKeys)> {
    let serialized_credentials =
        serialize_signin_credentials(&key, credentials.username, credentials.password);
    let encrypted_credentials = encrypt_with_cert(&auth_certificate, &serialized_credentials)
        .context("could not encrypt authentication credentials for auth certificate")?;
    let (public_auth_key, private_auth_key) = make_keypair();
    let encrypted_private_auth_key = encrypt_aes(key.raw(), &private_auth_key);
    let (public_license_key, private_license_key) = make_keypair();
    let encrypted_private_license_key = encrypt_aes(key.raw(), &private_license_key);

    let sign_in_data = adept::SignInData {
        sign_in_data: b64(&encrypted_credentials),
        public_auth_key: b64(&public_auth_key),
        encrypted_private_auth_key: b64(&encrypted_private_auth_key),
        public_license_key: b64(&public_license_key),
        encrypted_private_license_key: b64(&encrypted_private_license_key),
    };
    let private_keys = PrivateKeys {
        private_auth_key,
        private_license_key,
    };

    Ok((sign_in_data, private_keys))
}

fn is_sign_in_method_available(auth_service: &AdobeAuthServiceInfo, method: &str) -> bool {
    let available_methods = &auth_service.sign_in_methods;
    available_methods.iter().find(|&x| x == method).is_some()
}

const ANONYMOUS_CREDENTIALS: SignInCredentials<'static> = SignInCredentials {
    username: "anonymous",
    password: "",
};

#[derive(Debug, Clone, Copy)]
pub enum SignInMethod<'a> {
    Anonymous,
    _Dummy(&'a str),
}

fn sign_in_method_to_credentials(
    auth_service: &AdobeAuthServiceInfo,
    sign_in_method: SignInMethod<'_>,
) -> anyhow::Result<SignInCredentials<'static>> {
    let method = sign_in_method_to_method_name(sign_in_method);
    if !is_sign_in_method_available(auth_service, method) {
        return Err(anyhow::anyhow!(
            "sign in method is not available: {}",
            method
        ));
    }

    let credentials = match sign_in_method {
        SignInMethod::Anonymous => ANONYMOUS_CREDENTIALS,
        SignInMethod::_Dummy(_) => unreachable!(),
    };

    Ok(credentials)
}

fn sign_in_method_to_method_name(sign_in_method: SignInMethod<'_>) -> &'static str {
    match sign_in_method {
        SignInMethod::Anonymous => "anonymous",
        SignInMethod::_Dummy(_) => unreachable!(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub user: String,
    #[serde(with = "serde_base64")]
    pub private_auth_key: Vec<u8>,
    #[serde(with = "serde_base64")]
    pub user_certificate: Vec<u8>,
    #[serde(with = "serde_base64")]
    pub private_license_key: Vec<u8>,
    #[serde(with = "serde_base64")]
    pub license_certificate: Vec<u8>,
}

pub async fn sign_in<H: HttpClient>(
    http_client: &H,
    auth_service: &AdobeAuthServiceInfo,
    sign_in_method: SignInMethod<'_>,
) -> anyhow::Result<UserCredentials> {
    let ephemeral_key = EphemeralKey::generate();
    let sign_in_credentials = sign_in_method_to_credentials(auth_service, sign_in_method)
        .context("could not convert sign in method to credentials")?;
    let method = sign_in_method_to_method_name(sign_in_method);
    let (sign_in_data, private_keys) = make_sign_in_data(
        &ephemeral_key,
        &auth_service.auth_certificate,
        sign_in_credentials,
    )
    .context("failed to make sign in data")?;

    let credentials = adept::sign_in(http_client, &auth_service.auth_url, method, sign_in_data)
        .await
        .context("sign in failed")?;

    log::debug!("credentials: {:?}", credentials);

    let pkcs_password = b64(ephemeral_key.raw());
    let pkcs_der = unb64(&credentials.pkcs12)?;
    let parsed_pkcs = parse_pkcs12(&pkcs_der, &pkcs_password).context("could not parse pkcs12")?;

    let private_license_key = decrypt_aes(
        &ephemeral_key.0,
        &unb64(&credentials.encrypted_private_license_key)?,
    )
    .context("could not decrypt encrypted private license key")?;

    let license_certificate = unb64(&credentials.license_certificate)?;

    if private_license_key != private_keys.private_license_key {
        log::warn!(
            "The generated private license key and the private license key returned after sign in don't match, but they are expected to"
        );
    }
    if parsed_pkcs.pkey != private_keys.private_auth_key {
        log::warn!(
            "The generated private auth key and the user private key from pkcs12 returned after sign in don't match, but they are expected to"
        );
    }

    Ok(UserCredentials {
        user: credentials.user,
        private_auth_key: parsed_pkcs.pkey,
        user_certificate: parsed_pkcs.cert,
        private_license_key,
        license_certificate,
    })
}
