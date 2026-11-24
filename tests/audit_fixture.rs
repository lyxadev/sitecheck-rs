use std::process::Command;

#[test]
fn audits_fixture_folder_offline() {
    let bin = env!("CARGO_BIN_EXE_sitecheck");
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/site");
    let out = Command::new(bin)
        .args(["audit", fixture, "--json"])
        .output()
        .expect("run sitecheck");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid json");
    let pages = json["pages"].as_array().expect("pages array");
    assert_eq!(pages.len(), 2);
    let score = json["score"].as_u64().expect("score");
    assert!(score < 100, "fixture has deliberate problems");
    let codes: Vec<&str> = pages
        .iter()
        .flat_map(|p| p["issues"].as_array().unwrap())
        .map(|i| i["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"img-alt-missing"));
    assert!(codes.contains(&"broken-link"));
}

#[test]
fn fail_under_exits_non_zero() {
    let bin = env!("CARGO_BIN_EXE_sitecheck");
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/site");
    let status = Command::new(bin)
        .args(["audit", fixture, "--fail-under", "101"])
        .status()
        .expect("run sitecheck");
    assert_eq!(status.code(), Some(1));
}
