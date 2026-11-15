//! Registry constraints that Cargo does not enforce during a publish dry-run.

#[test]
fn keywords_fit_crates_io_limit() {
    let output = std::process::Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .output()
        .expect("cargo metadata should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let package = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == env!("CARGO_PKG_NAME"))
        .expect("package metadata should include this crate");
    let keywords = package["keywords"].as_array().unwrap();
    let count = keywords.len();
    assert!(
        count <= 5,
        "crates.io accepts at most 5 keywords per crate; found {count}: {keywords:?}"
    );
}
