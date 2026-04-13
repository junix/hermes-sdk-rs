use crate::types::{CreateResponseRequest, DeleteResponse, HermesError, Response};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8642";

/// Async client for the Hermes Responses API.
#[derive(Debug, Clone)]
pub struct HermesClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl HermesClient {
    /// Create a new client.
    ///
    /// `api_key` is the `API_SERVER_KEY` configured on the gateway.
    /// `base_url` defaults to `http://127.0.0.1:8642` if empty.
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let base_url = if base_url.is_empty() {
            DEFAULT_BASE_URL.to_string()
        } else {
            base_url
        };
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key: api_key.into(),
        }
    }

    /// `POST /v1/responses` — create a new response.
    pub async fn create_response(
        &self,
        request: &CreateResponseRequest,
    ) -> Result<Response, HermesError> {
        let url = format!("{}/v1/responses", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(request)
            .send()
            .await?;

        self.handle_response(resp).await
    }

    /// `GET /v1/responses/{id}` — retrieve a stored response.
    pub async fn get_response(&self, id: &str) -> Result<Response, HermesError> {
        let url = format!("{}/v1/responses/{}", self.base_url, id);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        self.handle_response(resp).await
    }

    /// `DELETE /v1/responses/{id}` — delete a stored response.
    pub async fn delete_response(&self, id: &str) -> Result<DeleteResponse, HermesError> {
        let url = format!("{}/v1/responses/{}", self.base_url, id);
        let resp = self
            .http
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(HermesError::from_response(status, &body));
        }

        resp.json::<DeleteResponse>()
            .await
            .map_err(HermesError::from)
    }

    /// `GET /health` — check gateway health.
    pub async fn health(&self) -> Result<bool, HermesError> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(resp.status().is_success())
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<Response, HermesError> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(HermesError::from_response(status, &body));
        }

        resp.json::<Response>().await.map_err(HermesError::from)
    }
}
