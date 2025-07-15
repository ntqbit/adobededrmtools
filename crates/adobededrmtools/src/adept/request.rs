use anyhow::Context;
use serde::Serialize;

use super::{
    CONTENT_TYPE, USERAGENT,
    http_client::{HttpContent, HttpMethod, HttpRequest},
    xml::serialize_xml,
};

fn make_url(base: &str, path: &str) -> String {
    format!("{}{}", base, path)
}

pub fn make_get(base: &str, path: &str) -> HttpRequest {
    HttpRequest {
        method: HttpMethod::Get,
        url: make_url(base, path),
        useragent: USERAGENT.to_string(),
        content: None,
    }
}

pub fn make_post<T: Serialize>(base: &str, path: &str, content: &T) -> anyhow::Result<HttpRequest> {
    let content = serialize_xml(content).context("could not serialize request")?;
    make_post_serialized(base, path, &content)
}

pub fn make_post_serialized(base: &str, path: &str, content: &str) -> anyhow::Result<HttpRequest> {
    log::debug!("serialized: {}", content);

    let req = HttpRequest {
        method: HttpMethod::Post,
        url: make_url(base, path),
        useragent: USERAGENT.to_string(),
        content: Some(HttpContent {
            content_type: CONTENT_TYPE.to_string(),
            content: String::into_bytes(content.to_string()),
        }),
    };

    Ok(req)
}
