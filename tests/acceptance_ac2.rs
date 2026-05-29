//! AC2 (MUST): `--skill not-in-allowlist` exits 2 with stderr containing 'skill-not-allowed'.
//!
//! Read-only: edit-agent must not modify this file.

use std::process::Command;

fn binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.ends_with("deps") { p.pop(); }
    p.join("wm-skill-edit")
}

#[test]
fn acceptance_ac2_skill_not_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();

    let out = Command::new(binary())
        .args(["--skill", "definitely-not-allowed", "--anchor", "foo", "--after", "bar"])
        .env("HOME", home)
        .output()
        .expect("failed to spawn");

    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2, got {:?}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("skill-not-allowed"),
        "expected 'skill-not-allowed' in stderr, got: {stderr}"
    );
}
