use super::{
    GetTaskRequest, GetTaskResponse, KeySigner, LoginMessage, LoginRequest, ProverType,
    SubmitProofRequest, api::Api,
};
use crate::{config::CoordinatorConfig, prover::ProverProviderType, utils::VERSION};
use tokio::sync::Mutex;

pub struct CoordinatorClient {
    pub prover_name: String,
    pub prover_provider_type: ProverProviderType,
    pub key_signer: KeySigner,
    prover_types: Vec<ProverType>,
    vks: Vec<String>,
    api: Api,
    suppress_empty_task_errors: bool,
    token: Mutex<String>,
}

impl CoordinatorClient {
    pub fn new(
        cfg: CoordinatorConfig,
        prover_types: Vec<ProverType>,
        suppress_empty_task_errors: bool,
        vks: Vec<String>,
        prover_name: String,
        prover_provider_type: ProverProviderType,
        key_signer: KeySigner,
    ) -> eyre::Result<Self> {
        let api = Api::new(cfg)?;
        let client = Self {
            prover_types,
            vks,
            prover_name,
            prover_provider_type,
            key_signer,
            api,
            suppress_empty_task_errors,
            token: Default::default(),
        };
        Ok(client)
    }

    pub async fn get_task(
        &self,
        req: &GetTaskRequest<'_>,
    ) -> eyre::Result<Option<GetTaskResponse>> {
        let response = {
            let token = self.token.lock().await;
            self.api.get_task(req, &token).await?
        };

        let response = if response.is_jwt_token_expired() {
            let token = self.refresh_token().await?;
            self.api.get_task(req, &token).await?
        } else {
            response
        };

        if response.is_empty_task_error() && self.suppress_empty_task_errors {
            return Ok(None);
        }
        Ok(Some(response.into_result()?))
    }

    pub async fn submit_proof(&self, req: &SubmitProofRequest<'_>) -> eyre::Result<()> {
        let response = {
            let token = self.token.lock().await;
            self.api.submit_proof(req, &token).await?
        };

        if response.is_jwt_token_expired() {
            let token = self.refresh_token().await?;
            self.api.submit_proof(req, &token).await?.into_result()?;
        } else {
            response.into_result()?;
        }
        Ok(())
    }

    /// Refresh the jwt token for authentication.
    pub async fn refresh_token(&self) -> eyre::Result<String> {
        let token = {
            // login and get new token
            let challenge = self.api.challenge().await?;

            let login_message = LoginMessage {
                challenge: &challenge.token,
                prover_version: VERSION,
                prover_name: &self.prover_name,
                prover_provider_type: self.prover_provider_type,
                prover_types: &self.prover_types,
                vks: &self.vks,
            };
            let buffer = alloy_rlp::encode(&login_message);
            let signature = self
                .key_signer
                .sign_buffer(&buffer)
                .map_err(|e| eyre::eyre!("Failed to sign the login message: {e}"))?;

            let response = self
                .api
                .login(
                    &LoginRequest {
                        message: login_message,
                        public_key: &self.key_signer.get_public_key(),
                        signature: &signature,
                    },
                    &challenge.token,
                )
                .await?;

            response.token
        };

        let mut guard = self.token.lock().await; // ignore poisoned lock
        *guard = token.clone();

        Ok(token)
    }
}
