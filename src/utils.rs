use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use std::cell::OnceCell;

static DEFAULT_COMMIT: &str = "unknown";
static mut VERSION: OnceCell<String> = OnceCell::new();

pub const TAG: &str = "v0.0.0";
pub const DEFAULT_ZK_VERSION: &str = "000000-000000";

fn init_version() -> String {
    let commit = option_env!("GIT_REV").unwrap_or(DEFAULT_COMMIT);
    let tag = option_env!("GO_TAG").unwrap_or(TAG);
    let zk_version = option_env!("ZK_VERSION").unwrap_or(DEFAULT_ZK_VERSION);
    format!("{tag}-{commit}-{zk_version}")
}

pub fn get_version() -> String {
    unsafe { VERSION.get_or_init(init_version).clone() }
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_ansi(false)
        .with_level(true)
        .with_target(true)
        .try_init()
        .expect("Failed to initialize tracing subscriber");
}

pub fn format_cloud_prover_name(provider_name: String, index: usize) -> String {
    // note the name of cloud prover is in fact in the format of "cloud_prover_{provider-name}_index",
    format!("cloud_prover_{}_{}", provider_name, index)
}
