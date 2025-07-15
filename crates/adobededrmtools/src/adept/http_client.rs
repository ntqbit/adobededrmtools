pub enum HttpMethod {
    Get,
    Post,
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub useragent: String,
    pub content: Option<HttpContent>,
}

pub struct HttpContent {
    pub content_type: String,
    pub content: Vec<u8>,
}

pub struct HttpResponse {
    pub response_code: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

pub trait HttpClient {
    fn request(&self, request: HttpRequest) -> impl Future<Output = anyhow::Result<HttpResponse>>;
}
