use anyhow::Context;
use serde::{Deserialize, de::DeserializeOwned};

use super::{AdeptError, http_client::HttpResponse, xml::deserialize_xml};

fn parse_response_inner(response: HttpResponse) -> anyhow::Result<String> {
    // All the successfull requests have status code 200.
    if response.response_code != 200 {
        return Err(anyhow::anyhow!(
            "unsuccessful http request: status={}",
            response.response_code
        ));
    }

    const EXPECTED_CONTENT_TYPE: &str = "application/vnd.adobe.adept+xml";

    if response.content_type != EXPECTED_CONTENT_TYPE {
        return Err(anyhow::anyhow!(
            "response content type mismatch. expected: {}, actual: {}",
            EXPECTED_CONTENT_TYPE,
            response.content_type
        ));
    }

    // Convert the body to string.
    let response = String::from_utf8(response.body)
        .ok()
        .context("response body is expected to be a valid UTF8 string, but it is not")?;

    Ok(response)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "error")]
pub struct AdeptErrorDto {
    #[serde(rename = "@data")]
    data: String,
}

fn parse_adept_error(dto: AdeptErrorDto) -> anyhow::Result<AdeptError> {
    // Example: E_ADEPT_MISSING_REQUEST_CONTENT_TYPE http://adeactivate.adobe.com/adept/SignInDirect

    let mut parts = dto.data.split(" ");

    let Some(name) = parts.next() else {
        // If no space, then the whole error is the name.
        return Ok(AdeptError {
            name: dto.data,
            args: Vec::new(),
        });
    };

    let args = parts.map(|x| x.to_string()).collect();

    Ok(AdeptError {
        name: name.to_string(),
        args,
    })
}

fn try_parse_as_error(response: &str) -> anyhow::Result<Option<AdeptError>> {
    let Ok(parse_error) = deserialize_xml(&response) else {
        return Ok(None);
    };

    let adept_error = parse_adept_error(parse_error).context("could not parse adept error")?;
    Ok(Some(adept_error))
}

pub fn parse_response<T: DeserializeOwned>(response: HttpResponse) -> anyhow::Result<T> {
    let response = parse_response_inner(response)?;
    log::debug!("response: {}", response);

    // First try parse it as error.
    // If we first parse it as T and T is () or any other type that does not contain any fields,
    // then parsing it will succeed even if the response was actually an error, and we'll get a false positive.
    if let Some(error) = try_parse_as_error(&response)? {
        return Err(error.into());
    }

    // If this fails as well, then the server returned neither an error, nor the expected response,
    // which may suggest that the API has changed
    deserialize_xml(&response).context("could not deserialize xml")
}
