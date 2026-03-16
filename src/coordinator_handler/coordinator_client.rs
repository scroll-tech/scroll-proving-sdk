use super::{
    GetTaskRequest, GetTaskResponse, KeySigner, LoginMessage, LoginRequest, ProverType,
    SubmitProofRequest, api::Api,
};
use crate::{config::CoordinatorConfig, prover::ProverProviderType, utils::VERSION};
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::OnceCell;

pub struct CoordinatorClient {
    pub prover_name: String,
    pub prover_provider_type: ProverProviderType,
    pub key_signer: KeySigner,
    prover_types: Vec<ProverType>,
    vks: Vec<String>,
    api: Api,
    suppress_empty_task_errors: bool,
    token: OnceCell<Mutex<Arc<str>>>,
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
            token: OnceCell::new(),
        };
        Ok(client)
    }
    pub async fn token(&self) -> eyre::Result<Arc<str>> {
        let mutex = self.get_lazy_init_mutex().await?;
        let guard = mutex.lock().unwrap_or_else(|e| e.into_inner()); // ignore poisoned lock
        Ok(guard.clone())
    }

    pub async fn get_task(
        &self,
        req: &GetTaskRequest<'_>,
    ) -> eyre::Result<Option<GetTaskResponse>> {
        let response = self.api.get_task(req, &self.token().await?).await?;

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
        let response = self.api.submit_proof(req, &self.token().await?).await?;

        if response.is_jwt_token_expired() {
            let token = self.refresh_token().await?;
            self.api.submit_proof(req, &token).await?.into_result()?;
        } else {
            response.into_result()?;
        }
        Ok(())
    }

    /// Refresh the jwt token for authentication.
    pub async fn refresh_token(&self) -> eyre::Result<Arc<str>> {
        let mutex = self.get_lazy_init_mutex().await?;

        let token = refresh_token(
            &self.api,
            &self.prover_name,
            self.prover_provider_type,
            &self.prover_types,
            &self.vks,
            &self.key_signer,
        )
        .await?;
        let mut guard = mutex.lock().unwrap_or_else(|e| e.into_inner()); // ignore poisoned lock
        *guard = token.clone();

        Ok(token)
    }

    async fn get_lazy_init_mutex(&self) -> eyre::Result<&Mutex<Arc<str>>> {
        self.token
            .get_or_try_init::<eyre::Report, _, _>(|| async {
                let token = refresh_token(
                    &self.api,
                    &self.prover_name,
                    self.prover_provider_type,
                    &self.prover_types,
                    &self.vks,
                    &self.key_signer,
                )
                .await?;
                Ok(Mutex::new(token))
            })
            .await
    }
}

async fn refresh_token(
    api: &Api,
    prover_name: &str,
    prover_provider_type: ProverProviderType,
    prover_types: &[ProverType],
    vks: &[String],
    key_signer: &KeySigner,
) -> eyre::Result<Arc<str>> {
    // login and get new token
    let challenge = api.challenge().await?;

    let login_message = LoginMessage {
        challenge: Cow::Borrowed(&challenge.token),
        prover_version: VERSION.into(),
        prover_name: Cow::Borrowed(prover_name),
        prover_provider_type,
        prover_types: prover_types.into(),
        vks: vks.to_vec(),
    };
    let buffer = alloy_rlp::encode(&login_message);
    let signature = key_signer
        .sign_buffer(&buffer)
        .map_err(|e| eyre::eyre!("Failed to sign the login message: {e}"))?;

    let response = api
        .login(
            &LoginRequest {
                message: login_message,
                public_key: key_signer.get_public_key().into(),
                signature: signature.into(),
            },
            &challenge.token,
        )
        .await?;

    Ok(Arc::from(response.token))
}
