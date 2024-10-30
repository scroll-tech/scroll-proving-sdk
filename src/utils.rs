use tracing_subscriber::filter::{EnvFilter, LevelFilter};

const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_version(circuit_version: &str) -> String {
    format!("sdk-{}-{}", SDK_VERSION, circuit_version)
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
