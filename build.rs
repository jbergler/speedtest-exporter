fn normalize_version(version: &str) -> String {
    let trimmed = version.trim();
    trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed)
        .to_string()
}

fn main() {
    println!("cargo::rerun-if-env-changed=APP_VERSION");

    let version = std::env::var("APP_VERSION")
        .ok()
        .map(|value| normalize_version(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string())
        });

    println!("cargo::rustc-env=APP_BUILD_VERSION={version}");
}