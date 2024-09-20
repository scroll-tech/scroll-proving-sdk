use ethers_core::{
    k256::{ecdsa::SigningKey, PublicKey, SecretKey},
    utils::hex,
};
use rand;
use rand::RngCore;
use std::fs::File;
use std::io::{Read, Write};

const DEFAULT_KEY_SIZE: usize = 32usize;

fn read_key_from_disk(key_path: &str) -> anyhow::Result<Vec<u8>> {
    let mut file = File::open(key_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(hex::decode(content)?)
}

fn gen_key_save_to_disk(key_path: &str) -> anyhow::Result<Vec<u8>> {
    // Generate a random private key.
    let mut secret = vec![0u8; DEFAULT_KEY_SIZE];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(secret.as_mut_slice());

    let content = hex::encode(secret.clone());
    let mut file = File::create(key_path)?;
    file.write_all(content.as_bytes())?;

    Ok(secret)
}

#[derive(Clone)]
pub struct KeySigner {
    public_key: PublicKey,
    signing_key: SigningKey,
}

impl KeySigner {
    pub fn new(key_path: &str) -> anyhow::Result<Self> {
        let secret = match read_key_from_disk(key_path) {
            Ok(secret) => secret,
            Err(_) => gen_key_save_to_disk(key_path)?,
        };

        let secret_key = SecretKey::from_bytes(secret.as_slice().into())?;
        let signing_key = SigningKey::from(secret_key.clone());
        Ok(Self {
            public_key: secret_key.public_key(),
            signing_key,
        })
    }

    pub fn sign(&self, data: &[u8]) -> anyhow::Result<String> {
        todo!()
    }
}
