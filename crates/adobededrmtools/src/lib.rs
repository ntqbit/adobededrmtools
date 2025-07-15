mod adept;

mod activation;
mod auth;
pub mod dedrm;
mod facade;
mod fulfillment;
mod serializarion;
mod services;

pub use activation::DeviceInfo;
pub use adept::{Acsm, DEFAULT_ACTIVATION_URL, HttpClient, http_client};
pub use adobededrmtools_crypto::make_signer;
pub use auth::UserCredentials;
pub use facade::{
    AdobeAccount, AdobeMinServicesInfo, CreateAccountParams, create_adobe_account, fulfill_acsm,
};
pub use fulfillment::{DownloadInfo, Resource, ResourceEncryptedKey};

fn random_nonce() -> String {
    use adobededrmtools_crypto::{b64, rand_bytes};
    b64(&rand_bytes::<8>())
}

fn make_expiration() -> String {
    use std::time::{Duration, SystemTime};
    let expiration_instant = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 10))
        .expect("could not add duration to instant");

    let dt: chrono::DateTime<chrono::Utc> = expiration_instant.into();
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
