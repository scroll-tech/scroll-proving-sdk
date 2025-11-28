use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::FmtSpan;

pub static VERSION: &str = concat!(
    env!("GO_TAG", "semver from `./common/version.go` is required"),
    "-",
    env!("GIT_REV", "git rev of `scroll` is required"),
    "-",
    env!("ZK_VERSION", "`zkvm-prover` version and commit is required"),
);

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
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .try_init()
        .expect("Failed to initialize tracing subscriber");
}

pub fn format_cloud_prover_name(provider_name: String, index: usize) -> String {
    // note the name of cloud prover is in fact in the format of "cloud_prover_{provider-name}_index",
    format!("cloud_prover_{}_{}", provider_name, index)
}
