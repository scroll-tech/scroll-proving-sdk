use super::{
    ChallengeResponseData, GetTaskRequest, GetTaskResponseData, LoginRequest, LoginResponseData,
    Response,
};
use reqwest::{header::CONTENT_TYPE, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Serialize;

pub struct Api {
    pub base_url: Url,
    pub client: ClientWithMiddleware,
}

impl Api {
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        let client = ClientBuilder::new(reqwest::Client::new())
            // .with(RetryTransientMiddleware::new_with_policy(retry_policy)) // TODO: retry policy
            .build();

        Ok(Self {
            base_url: Url::parse(base_url)?,
            client,
        })
    }

    fn build_url(&self, method: &str) -> anyhow::Result<Url> {
        self.base_url.join(method).map_err(|e| anyhow::anyhow!(e))
    }

    async fn post_with_token<Req, Resp>(
        &self,
        method: &str,
        req: &Req,
        token: &String,
    ) -> anyhow::Result<Resp>
    where
        Req: ?Sized + Serialize,
        Resp: serde::de::DeserializeOwned,
    {
        let url = self.build_url(method)?;
        let request_body = serde_json::to_string(req)?;

        log::info!("[coordinator client], {method}, request: {request_body}");
        let response = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .body(request_body)
            // .timeout(self.send_timeout) // TODO: send_timeout
            .send()
            .await?;

        if response.status() != http::status::StatusCode::OK {
            log::error!(
                "[coordinator client], {method}, status not ok: {}",
                response.status()
            );
            anyhow::bail!(
                "[coordinator client], {method}, status not ok: {}",
                response.status()
            )
        }

        let response_body = response.text().await?;

        log::info!("[coordinator client], {method}, response: {response_body}");
        serde_json::from_str(&response_body).map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn challenge(&self) -> anyhow::Result<Response<ChallengeResponseData>> {
        let method = "/coordinator/v1/challenge";
        let url = self.build_url(method)?;

        let response = self
            .client
            .get(url)
            .header(CONTENT_TYPE, "application/json")
            // .timeout(self.send_timeout) // TODO: send_timeout
            .send()
            .await?;

        let response_body = response.text().await?;

        serde_json::from_str(&response_body).map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn login(
        &self,
        req: &LoginRequest,
        token: &String,
    ) -> anyhow::Result<Response<LoginResponseData>> {
        let method = "/coordinator/v1/login";
        self.post_with_token(method, req, token).await
    }

    pub async fn get_task(
        &self,
        req: &GetTaskRequest,
        token: &String,
    ) -> anyhow::Result<Response<GetTaskResponseData>> {
        let method = "/coordinator/v1/get_task";
        self.post_with_token(method, req, token).await
    }
}
