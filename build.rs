use std::env;

const DEFAULT_COMMIT: &str = "unknown";
const DEFAULT_ZK_VERSION: &str = "000000-000000";
const DEFAULT_TAG: &str = "v0.0.0";

fn main() {
    println!(
        "cargo:rustc-env=GIT_REV={}",
        env::var("GIT_REV").unwrap_or_else(|_| DEFAULT_COMMIT.to_string()),
    );
    println!(
        "cargo:rustc-env=GO_TAG={}",
        env::var("GO_TAG").unwrap_or_else(|_| DEFAULT_TAG.to_string()),
    );
    println!(
        "cargo:rustc-env=ZK_VERSION={}",
        env::var("ZK_VERSION").unwrap_or_else(|_| DEFAULT_ZK_VERSION.to_string()),
    );
}
