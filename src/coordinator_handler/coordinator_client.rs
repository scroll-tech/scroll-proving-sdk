use super::{
    api::Api, error::ErrorCode, GetTaskRequest, GetTaskResponseData, KeySigner, LoginMessage,
    LoginRequest, Response, SubmitProofRequest, SubmitProofResponseData,
};
use crate::{
    config::CoordinatorConfig,
    prover::{CircuitType, ProverProviderType},
    utils::get_version,
};
use tokio::sync::{Mutex, MutexGuard};

pub struct CoordinatorClient {
    circuit_types: Vec<CircuitType>,
    vks: Vec<String>,
    pub prover_name: String,
    pub prover_provider_type: ProverProviderType,
    pub key_signer: KeySigner,
    api: Api,
    token: Mutex<Option<String>>,
}

impl CoordinatorClient {
    pub fn new(
        cfg: CoordinatorConfig,
        circuit_types: Vec<CircuitType>,
        vks: Vec<String>,
        prover_name: String,
        prover_provider_type: ProverProviderType,
        key_signer: KeySigner,
    ) -> anyhow::Result<Self> {
        let api = Api::new(cfg)?;
        let client = Self {
            circuit_types,
            vks,
            prover_name,
            prover_provider_type,
            key_signer,
            api,
            token: Mutex::new(None),
        };
        Ok(client)
    }

    pub async fn get_task(
        &self,
        req: &GetTaskRequest,
    ) -> anyhow::Result<Response<GetTaskResponseData>> {
        let token = self.get_token(false).await?;
        let response = self.api.get_task(req, &token).await?;

        if response.errcode == ErrorCode::ErrJWTTokenExpired {
            let token = self.get_token(true).await?;
            self.api.get_task(req, &token).await
        } else {
            Ok(response)
        }
    }

    pub async fn submit_proof(
        &self,
        req: &SubmitProofRequest,
    ) -> anyhow::Result<Response<SubmitProofResponseData>> {
        let token = self.get_token(false).await?;
        let response = self.api.submit_proof(req, &token).await?;

        if response.errcode == ErrorCode::ErrJWTTokenExpired {
            let token = self.get_token(true).await?;
            self.api.submit_proof(req, &token).await
        } else {
            Ok(response)
        }
    }

    /// Retrieves a token for authentication, optionally forcing a re-login.
    ///
    /// This function attempts to get the stored token if `force_relogin` is set to `false`.
    ///
    /// If the token is expired, `force_relogin` is set to `true`, or a login was never performed
    /// before, it will authenticate and fetch a new token.
    pub async fn get_token(&self, force_relogin: bool) -> anyhow::Result<String> {
        let token_guard = self.token.lock().await;

        match *token_guard {
            Some(ref token) if !force_relogin => return Ok(token.to_string()),
            _ => (),
        }

        self.login(token_guard).await
    }

    async fn login<'t>(
        &self,
        mut token_guard: MutexGuard<'t, Option<String>>,
    ) -> anyhow::Result<String> {
        let challenge_response = self
            .api
            .challenge()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to request a challenge: {e}"))?;

        if challenge_response.errcode != ErrorCode::Success {
            anyhow::bail!(
                "Challenge request failed with {:?} {}",
                challenge_response.errcode,
                challenge_response.errmsg
            );
        }

        let login_response_data = challenge_response
            .data
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing challenge token"))?;

        let mut prover_types = vec![];
        if self.circuit_types.contains(&CircuitType::Bundle)
            || self.circuit_types.contains(&CircuitType::Batch)
        {
            prover_types.push(CircuitType::Batch)
        }
        if self.circuit_types.contains(&CircuitType::Chunk) {
            prover_types.push(CircuitType::Chunk)
        }

        let login_message = LoginMessage {
            challenge: login_response_data.token.clone(),
            prover_version: get_version().to_string(),
            prover_name: self.prover_name.clone(),
            prover_provider_type: Some(self.prover_provider_type),
            prover_types,
            vks: self.vks.clone(),
        };

        let buffer = rlp::encode(&login_message);
        let signature = self
            .key_signer
            .sign_buffer(&buffer)
            .map_err(|e| anyhow::anyhow!("Failed to sign the login message: {e}"))?;


        let hex_string: String = buffer.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();

        println!("login_message Hex string: {}", hex_string);
        println!("login_message signature: {}", signature);

        let login_request = LoginRequest {
            message: login_message,
            public_key: self.key_signer.get_public_key(),
            signature,
        };

        let login_response = self
            .api
            .login(&login_request, &login_response_data.token)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to login: {e}"))?;

        if login_response.errcode != ErrorCode::Success {
            anyhow::bail!(
                "Login request failed with {:?} {}",
                login_response.errcode,
                login_response.errmsg
            );
        }
        let token = login_response
            .data
            .map(|r| r.token)
            .ok_or_else(|| anyhow::anyhow!("Empty data in response, lack of login"))?;

        *token_guard = Some(token.clone());

        Ok(token)
    }
}
