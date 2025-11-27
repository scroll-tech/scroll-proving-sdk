use super::{
    ChallengeResponseData, GetTaskRequest, GetTaskResponseData, LoginRequest, LoginResponseData,
    Response, SubmitProofRequest, SubmitProofResponseData,
};
use crate::config::CoordinatorConfig;
use core::time::Duration;
use reqwest::{header::CONTENT_TYPE, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Serialize;
use tracing::Level;

pub struct Api {
    pub base_url: Url,
    send_timeout: Duration,
    pub client: ClientWithMiddleware,
}

impl Api {
    pub fn new(cfg: CoordinatorConfig) -> eyre::Result<Self> {
        let retry_wait_duration = Duration::from_secs(cfg.retry_wait_time_sec);
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(retry_wait_duration / 2, retry_wait_duration)
            .build_with_max_retries(cfg.retry_count);

        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            base_url: Url::parse(&cfg.base_url)?,
            send_timeout: core::time::Duration::from_secs(cfg.connection_timeout_sec),
            client,
        })
    }

    fn build_url(&self, method: &str) -> eyre::Result<Url> {
        self.base_url.join(method).map_err(|e| eyre::eyre!(e))
    }

    #[instrument(target = "coordinator_client", skip(self, req, token), level = Level::DEBUG)]
    async fn post_with_token<Req, Resp>(
        &self,
        method: &str,
        req: &Req,
        token: &String,
    ) -> eyre::Result<Resp>
    where
        Req: ?Sized + Serialize,
        Resp: serde::de::DeserializeOwned,
    {
        let url = self.build_url(method)?;
        let request_body = serde_json::to_string(req)?;
        let size = request_body.len();

        info!("sent request");
        trace!(token = %token, request_body = %request_body, size = %size);
        let response = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .body(request_body)
            .timeout(self.send_timeout)
            .send()
            .await?;

        if response.status() != http::status::StatusCode::OK {
            eyre::bail!(
                "[coordinator client], {method}, status not ok: {}",
                response.status()
            )
        }

        let response_body = response.text().await?;

        info!("received response");
        trace!(response_body = %response_body);
        serde_json::from_str(&response_body).map_err(|e| eyre::eyre!(e))
    }

    pub async fn challenge(&self) -> eyre::Result<Response<ChallengeResponseData>> {
        let method = "/coordinator/v1/challenge";
        let url = self.build_url(method)?;

        let response = self
            .client
            .get(url)
            .header(CONTENT_TYPE, "application/json")
            .timeout(self.send_timeout)
            .send()
            .await?;

        let response_body = response.text().await?;

        serde_json::from_str(&response_body).map_err(|e| eyre::eyre!(e))
    }

    pub async fn login(
        &self,
        req: &LoginRequest,
        token: &String,
    ) -> eyre::Result<Response<LoginResponseData>> {
        let method = "/coordinator/v1/login";
        self.post_with_token(method, req, token).await
    }

    pub async fn get_task(
        &self,
        req: &GetTaskRequest,
        token: &String,
    ) -> eyre::Result<Response<GetTaskResponseData>> {
        let method = "/coordinator/v1/get_task";
        if self.send_timeout < Duration::from_secs(600) {
            tracing::warn!(
                "get_task API is time-consuming, timeout setting is too low ({}), set it to more than 600s",
                self.send_timeout.as_secs(),
            );
        }

        self.post_with_token(method, req, token).await
    }

    pub async fn submit_proof(
        &self,
        req: &SubmitProofRequest,
        token: &String,
    ) -> eyre::Result<Response<SubmitProofResponseData>> {
        let method = "/coordinator/v1/submit_proof";
        self.post_with_token(method, req, token).await
    }
}
