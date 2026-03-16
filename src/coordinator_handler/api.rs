use super::{
    ChallengeResponse, GetTaskRequest, GetTaskResponse, LoginRequest, LoginResponse, Response,
    SubmitProofRequest,
};
use crate::config::CoordinatorConfig;
use core::time::Duration;
use eyre::Context;
use http::{Method, StatusCode};
use reqwest::{Url, header::CONTENT_TYPE};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize};
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
            base_url: Url::parse(cfg.base_url.trim_end_matches('/'))?,
            send_timeout: Duration::from_secs(cfg.connection_timeout_sec),
            client,
        })
    }

    pub async fn challenge(&self) -> eyre::Result<ChallengeResponse> {
        const PATH: &str = "/coordinator/v1/challenge";
        let response: Response<ChallengeResponse> =
            self.request(Method::GET, PATH, None::<&()>, None).await?;
        response.into_result().context("challenge request failed")
    }

    #[instrument(skip(self, req, token), level = Level::DEBUG)]
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

        debug!("sent request");
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

        debug!("received response");
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
        req: &GetTaskRequest<'_>,
        token: &str,
    ) -> eyre::Result<Response<GetTaskResponse>> {
        const PATH: &str = "/coordinator/v1/get_task";

        if self.send_timeout < Duration::from_secs(600) {
            tracing::warn!(
                "get_task API is time-consuming, timeout setting is too low ({}), set it to more than 600s",
                self.send_timeout.as_secs(),
            );
        }

        self.request(Method::POST, PATH, Some(req), Some(token))
            .await
    }

    pub async fn submit_proof(
        &self,
        req: &SubmitProofRequest<'_>,
        token: &str,
    ) -> eyre::Result<Response<()>> {
        const PATH: &str = "/coordinator/v1/submit_proof";
        self.request(Method::POST, PATH, Some(req), Some(token))
            .await
    }

    #[instrument(skip(self, body, token), level = Level::DEBUG)]
    async fn request<Req, T>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Req>,
        token: Option<&str>,
    ) -> eyre::Result<Response<T>>
    where
        Req: ?Sized + Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.base_url.join(path)?;

        let mut builder = self
            .client
            .request(method, url)
            .header(CONTENT_TYPE, "application/json")
            .timeout(self.send_timeout);

        if let Some(token) = token {
            trace!(token = %token);
            builder = builder.bearer_auth(token);
        }

        if let Some(body) = body {
            let request_body = serde_json::to_string(body)?;
            let size = request_body.len();
            trace!(request_body = %request_body, size = %size);
            builder = builder.body(request_body);
        }

        debug!("sending request");
        let response = builder.send().await?;

        if response.status() != StatusCode::OK {
            eyre::bail!("{path}, status not ok: {}", response.status())
        }

        let response = response.text().await?;
        debug!("received response");
        trace!(response_body = %response);
        let response = serde_json::from_str(&response)?;
        Ok(response)
    }
}
