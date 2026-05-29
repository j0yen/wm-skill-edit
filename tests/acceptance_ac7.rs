//! AC7 (SHOULD): Guard — target SKILL.md missing → exit 2 with 'skill-file-missing'.
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
fn acceptance_ac7_skill_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();
    // Do NOT create the skill directory — SKILL.md is absent.

    let out = Command::new(binary())
        .args(["--skill", "build", "--anchor", "some anchor", "--after", "some text"])
        .env("HOME", home)
        .output()
        .expect("failed to spawn");

    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 when SKILL.md missing, got {:?}",
        out.status.code()
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("skill-file-missing"),
        "expected 'skill-file-missing' in stderr, got: {stderr}"
    );
}
