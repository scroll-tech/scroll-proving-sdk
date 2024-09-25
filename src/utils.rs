use tracing_subscriber::filter::{EnvFilter, LevelFilter};

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
