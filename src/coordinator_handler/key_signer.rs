use ethers_core::{
    k256::{
        ecdsa::{signature::hazmat::PrehashSigner, RecoveryId, Signature, SigningKey},
        elliptic_curve::{sec1::ToEncodedPoint, FieldBytes},
        PublicKey, Secp256k1, SecretKey,
    },
    types::Signature as EthSignature,
    types::{H256, U256},
    utils::{hex, hex::ToHex},
};
use rand;
use rand::RngCore;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tiny_keccak::{Hasher, Keccak};

const DEFAULT_KEY_SIZE: usize = 32usize;

fn read_key_from_disk(key_path: &PathBuf) -> anyhow::Result<Vec<u8>> {
    let mut file = File::open(key_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(hex::decode(content)?)
}

fn gen_key_save_to_disk(key_path: &PathBuf) -> anyhow::Result<Vec<u8>> {
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
    pub fn new(key_path: &PathBuf) -> anyhow::Result<Self> {
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

    pub fn new_from_secret_key(secret_key: &str) -> anyhow::Result<Self> {
        let secret = hex::decode(secret_key).unwrap();
        let secret_key = SecretKey::from_bytes(secret.as_slice().into())?;
        let signing_key = SigningKey::from(secret_key.clone());
        Ok(Self {
            public_key: secret_key.public_key(),
            signing_key,
        })
    }

    pub fn get_public_key(&self) -> String {
        let v: Vec<u8> = Vec::from(self.public_key.to_encoded_point(true).as_bytes());
        buffer_to_hex(&v, false)
    }

    /// Signs the provided hash.
    pub fn sign_hash(&self, hash: H256) -> anyhow::Result<EthSignature> {
        let signing_key = &self.signing_key as &dyn PrehashSigner<(Signature, RecoveryId)>;
        let (recoverable_sig, recovery_id) = signing_key.sign_prehash(hash.as_ref())?;

        let v = u8::from(recovery_id) as u64;

        let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
        let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
        let r = U256::from_big_endian(r_bytes.as_slice());
        let s = U256::from_big_endian(s_bytes.as_slice());

        Ok(EthSignature { r, s, v })
    }

    pub fn sign_buffer<T>(&self, buffer: &T) -> anyhow::Result<String>
    where
        T: AsRef<[u8]>,
    {
        let pre_hash = keccak256(buffer);

        let hash = H256::from(pre_hash);
        let sig = self.sign_hash(hash)?;

        Ok(buffer_to_hex(&sig.to_vec(), true))
    }
}

fn buffer_to_hex<T>(buffer: &T, has_prefix: bool) -> String
where
    T: AsRef<[u8]>,
{
    if has_prefix {
        format!("0x{}", buffer.encode_hex::<String>())
    } else {
        buffer.encode_hex::<String>()
    }
}

/// Compute the Keccak-256 hash of input bytes.
///
/// Note that strings are interpreted as UTF-8 bytes,
pub fn keccak256<T: AsRef<[u8]>>(bytes: T) -> [u8; 32] {
    let mut output = [0u8; 32];

    let mut hasher = Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);

    output
}
