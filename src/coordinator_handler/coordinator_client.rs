use super::{
    api::Api, error::ErrorCode, GetTaskRequest, GetTaskResponseData, KeySigner, LoginMessage,
    LoginRequest, Response, SubmitProofRequest, SubmitProofResponseData,
};
use crate::{config::CoordinatorConfig, prover::CircuitType};
use std::sync::{Mutex, MutexGuard};
use tokio::runtime::Runtime;

pub struct CoordinatorClient {
    circuit_type: CircuitType,
    pub prover_name: String,
    key_signer: KeySigner,
    api: Api,
    token: Mutex<Option<String>>,
    rt: Runtime,
}

impl CoordinatorClient {
    pub fn new(
        cfg: CoordinatorConfig,
        circuit_type: CircuitType,
        prover_name: String,
        key_signer: KeySigner,
    ) -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let api = Api::new(&cfg.base_url)?; // TODO: retry policy
        let client = Self {
            circuit_type,
            prover_name,
            key_signer,
            api,
            token: Mutex::new(None),
            rt,
        };
        Ok(client)
    }

    pub fn get_task(&self, req: &GetTaskRequest) -> anyhow::Result<Response<GetTaskResponseData>> {
        let token = self.get_token_sync(false)?;
        let response = self.get_task_sync(req, &token)?;

        if response.errcode == ErrorCode::ErrJWTTokenExpired {
            let token = self.get_token_sync(true)?;
            self.get_task_sync(req, &token)
        } else {
            Ok(response)
        }
    }

    pub fn submit_proof(
        &self,
        req: &SubmitProofRequest,
    ) -> anyhow::Result<Response<SubmitProofResponseData>> {
        let token = self.get_token_sync(false)?;
        let response = self.submit_proof_sync(req, &token)?;

        if response.errcode == ErrorCode::ErrJWTTokenExpired {
            let token = self.get_token_sync(true)?;
            self.submit_proof_sync(req, &token)
        } else {
            Ok(response)
        }
    }

    fn get_task_sync(
        &self,
        req: &GetTaskRequest,
        token: &String,
    ) -> anyhow::Result<Response<GetTaskResponseData>> {
        self.rt.block_on(self.api.get_task(req, token))
    }

    fn submit_proof_sync(
        &self,
        req: &SubmitProofRequest,
        token: &String,
    ) -> anyhow::Result<Response<SubmitProofResponseData>> {
        self.rt.block_on(self.api.submit_proof(req, token))
    }

    fn get_token_sync(&self, force_relogin: bool) -> anyhow::Result<String> {
        self.rt.block_on(self.get_token_async(force_relogin))
    }

    /// Retrieves a token for authentication, optionally forcing a re-login.
    ///
    /// This function attempts to get the stored token if `force_relogin` is set to `false`.
    ///
    /// If the token is expired, `force_relogin` is set to `true`, or a login was never performed
    /// before, it will authenticate and fetch a new token.
    async fn get_token_async(&self, force_relogin: bool) -> anyhow::Result<String> {
        let token_guard = self
            .token
            .lock()
            .expect("Mutex locking only occurs within `get_token` fn, so there can be no double `lock` for one thread");

        match token_guard.as_deref() {
            Some(token) if !force_relogin => return Ok(token.to_string()),
            _ => (),
        }

        self.login_async(token_guard).await
    }

    async fn login_async<'t>(
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

        let login_message = LoginMessage {
            challenge: login_response_data.token.clone(),
            prover_name: self.prover_name.clone(),
            prover_version: "crate::version::get_version()".to_string(),
            prover_types: vec![self.circuit_type],
            vks: vec!["self.vks.clone()".to_string()],
        };

        let buffer = rlp::encode(&login_message);
        let signature = self
            .key_signer
            .sign_buffer(&buffer)
            .map_err(|e| anyhow::anyhow!("Failed to sign the login message: {e}"))?;

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
