use adobededrmtools::{DownloadInfo, http_client};
use anyhow::Context;

pub struct ReqwestHttpClient;

impl http_client::HttpClient for ReqwestHttpClient {
    async fn request(
        &self,
        request: http_client::HttpRequest,
    ) -> anyhow::Result<http_client::HttpResponse> {
        let method = match request.method {
            http_client::HttpMethod::Get => reqwest::Method::GET,
            http_client::HttpMethod::Post => reqwest::Method::POST,
        };

        let mut request_builder = reqwest::Client::new()
            .request(method, request.url)
            .header("User-Agent", request.useragent);

        if let Some(content) = request.content {
            request_builder = request_builder
                .header("Content-Type", content.content_type)
                .body(content.content);
        }

        let response = request_builder
            .send()
            .await
            .context("reqwest request send failed")?;

        let content_type = response
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string())
            .unwrap_or_default();

        let response_code = response.status().as_u16();

        let body = response
            .bytes()
            .await
            .context("failed to read response body")?;

        Ok(http_client::HttpResponse {
            response_code,
            content_type,
            body: body.into(),
        })
    }
}

pub trait ResourceDownloader {
    fn download_resource(
        &self,
        download_info: &DownloadInfo,
    ) -> impl Future<Output = anyhow::Result<DownloadedResource>>;
}

pub struct DownloadedResource {
    pub data: Vec<u8>,
}

pub struct ReqwestResourceDownloader;

impl ResourceDownloader for ReqwestResourceDownloader {
    async fn download_resource(
        &self,
        download_info: &DownloadInfo,
    ) -> anyhow::Result<DownloadedResource> {
        match download_info {
            DownloadInfo::Simple(src) => {
                let res = reqwest::get(src)
                    .await
                    .context("reqwest get request failed")?;

                if res.status() != reqwest::StatusCode::OK {
                    return Err(anyhow::anyhow!(
                        "resource download server returned non-200 status code: {}",
                        res.status()
                    ));
                }

                let data = res.bytes().await?;

                Ok(DownloadedResource { data: data.into() })
            }
        }
    }
}
