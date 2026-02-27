use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use reqwest::{
    self, Client, Method, Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};

use {
    super::super::error::{RestApiError, Result},
    super::rate_limit::RateLimiter,
};

pub(crate) trait SignatureGenerator: Send + Sync {
    fn generate(
        &self,
        timestamp: DateTime<Utc>,
        method: &Method,
        url: &Url,
        body: Option<&String>,
    ) -> Result<String>;
}

pub(crate) trait RestPath: Clone {
    fn to_path_string(self) -> String;
}

struct LnmRestCredentials<S: SignatureGenerator> {
    key: String,
    passphrase: String,
    signature_generator: S,
}

impl<S: SignatureGenerator> LnmRestCredentials<S> {
    fn new(key: String, passphrase: String, signature_generator: S) -> Self {
        Self {
            key,
            passphrase,
            signature_generator,
        }
    }

    fn get_authentication_headers(
        &self,
        method: &Method,
        url: &Url,
        body: Option<&String>,
    ) -> Result<HeaderMap> {
        let timestamp = Utc::now();

        let signature = self
            .signature_generator
            .generate(timestamp, method, url, body)?;

        let timestamp_str = timestamp.timestamp_millis().to_string();

        let mut headers = HeaderMap::new();

        headers.insert(
            HeaderName::from_static("lnm-access-key"),
            HeaderValue::from_str(&self.key)?,
        );
        headers.insert(
            HeaderName::from_static("lnm-access-signature"),
            HeaderValue::from_str(&signature)?,
        );
        headers.insert(
            HeaderName::from_static("lnm-access-passphrase"),
            HeaderValue::from_str(&self.passphrase)?,
        );
        headers.insert(
            HeaderName::from_static("lnm-access-timestamp"),
            HeaderValue::from_str(&timestamp_str)?,
        );

        Ok(headers)
    }
}

pub(crate) struct LnmRestBase<S: SignatureGenerator> {
    domain: String,
    credentials: Option<LnmRestCredentials<S>>,
    client: Client,
    rate_limiter: Option<RateLimiter>,
}

impl<S: SignatureGenerator> LnmRestBase<S> {
    pub fn new(
        timeout: Duration,
        domain: String,
        rate_limiter: Option<RateLimiter>,
    ) -> Result<Arc<Self>> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(RestApiError::HttpClient)?;

        Ok(Arc::new(Self {
            domain,
            credentials: None,
            client,
            rate_limiter,
        }))
    }

    pub fn with_credentials(
        timeout: Duration,
        domain: String,
        key: String,
        passphrase: String,
        signature_generator: S,
        rate_limiter: Option<RateLimiter>,
    ) -> Result<Arc<Self>> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(RestApiError::HttpClient)?;

        let creds = LnmRestCredentials::new(key, passphrase, signature_generator);

        Ok(Arc::new(Self {
            domain,
            credentials: Some(creds),
            client,
            rate_limiter,
        }))
    }

    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    fn build_url(&self, path: impl RestPath) -> Result<Url> {
        let url_str = format!("https://{}{}", self.domain, path.to_path_string());

        Url::parse(&url_str).map_err(|e| RestApiError::UrlParse(e.to_string()))
    }

    async fn make_request<T>(
        &self,
        method: Method,
        url: Url,
        body: Option<String>,
        authenticated: bool,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if let Some(rl) = &self.rate_limiter {
            rl.acquire(authenticated).await;
        }

        let headers = if authenticated {
            let creds = self
                .credentials
                .as_ref()
                .ok_or(RestApiError::MissingRequestCredentials)?;

            creds.get_authentication_headers(&method, &url, body.as_ref())?
        } else {
            HeaderMap::new()
        };

        let response = self.send_request(method, url, body, headers).await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .map_err(RestApiError::ResponseDecoding)?;

            return Err(RestApiError::ErrorResponse { status, text });
        }

        let raw_response = response
            .text()
            .await
            .map_err(RestApiError::ResponseDecoding)?;

        let response_data = serde_json::from_str::<T>(&raw_response)
            .map_err(|e| RestApiError::ResponseJsonDeserializeFailed { raw_response, e })?;

        Ok(response_data)
    }

    async fn send_request(
        &self,
        method: Method,
        url: Url,
        body: Option<String>,
        mut headers: HeaderMap,
    ) -> Result<reqwest::Response> {
        let req = match method {
            Method::POST | Method::PUT => {
                if body.is_some() {
                    headers.insert(
                        HeaderName::from_static("content-type"),
                        HeaderValue::from_static("application/json"),
                    );
                }

                let mut req = self.client.request(method, url).headers(headers);
                if let Some(body) = body {
                    req = req.body(body);
                }
                req
            }
            Method::GET | Method::DELETE => self.client.request(method, url).headers(headers),
            m => return Err(RestApiError::UnsupportedMethod(m)),
        };

        req.send().await.map_err(RestApiError::SendFailed)
    }

    pub async fn make_request_with_body<T, B>(
        &self,
        method: Method,
        path: impl RestPath,
        body: B,
        authenticated: bool,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = self.build_url(path)?;
        let body =
            serde_json::to_string(&body).map_err(RestApiError::RequestJsonSerializeFailed)?;

        self.make_request(method, url, Some(body), authenticated)
            .await
    }

    pub async fn make_request_with_query_params<I, K, V, T>(
        &self,
        method: Method,
        path: impl RestPath,
        query_params: I,
        authenticated: bool,
    ) -> Result<T>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
        T: DeserializeOwned,
    {
        let mut url = self.build_url(path)?;
        url.query_pairs_mut().extend_pairs(query_params);

        self.make_request(method, url, None, authenticated).await
    }

    pub async fn make_request_without_params<T>(
        &self,
        method: Method,
        path: impl RestPath,
        authenticated: bool,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;

        self.make_request(method, url, None, authenticated).await
    }

    pub async fn make_get_request_plain_text(&self, path: impl RestPath) -> Result<String> {
        let url = self.build_url(path)?;

        if let Some(rl) = &self.rate_limiter {
            rl.acquire(false).await;
        }

        let response = self
            .send_request(Method::GET, url, None, HeaderMap::new())
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .map_err(RestApiError::ResponseDecoding)?;

            return Err(RestApiError::ErrorResponse { status, text });
        }

        let raw_response = response
            .text()
            .await
            .map_err(RestApiError::ResponseDecoding)?;

        Ok(raw_response)
    }
}
