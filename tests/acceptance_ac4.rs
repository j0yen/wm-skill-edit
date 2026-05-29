//! AC4 (MUST): `--anchor '<absent>'` exits 2 with 'anchor-not-found'; no write occurs.
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
fn acceptance_ac4_anchor_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();
    let skill_dir = tmp.path().join(".claude").join("skills").join("dream");
    std::fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");
    let original = "## Section\nsome content\n## End\n";
    std::fs::write(&skill_md, original).unwrap();

    let out = Command::new(binary())
        .args(["--skill", "dream", "--anchor", "THIS_LINE_DOES_NOT_EXIST", "--after", "new line"])
        .env("HOME", home)
        .output()
        .expect("failed to spawn");

    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 for absent anchor, got {:?}",
        out.status.code()
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("anchor-not-found"),
        "expected 'anchor-not-found' in stderr, got: {stderr}"
    );
    let content = std::fs::read_to_string(&skill_md).unwrap();
    assert_eq!(content, original, "file was modified despite anchor-not-found guard");
}
