mod acsm;
mod activation;
mod fulfillment;
mod request;
mod response;
mod signature;
mod types;
mod xml;

pub mod http_client;

const USERAGENT: &str = "book2png";
const CONTENT_TYPE: &str = "application/vnd.adobe.adept+xml";
const ADEPT_XMLNS: &str = "http://ns.adobe.com/adept";

pub use acsm::Acsm;
pub use activation::*;
pub use fulfillment::*;
pub use http_client::HttpClient;
pub use types::*;
