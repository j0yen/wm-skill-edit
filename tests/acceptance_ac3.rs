//! AC3 (MUST): `--anchor '<ambiguous>'` that matches >1 line exits 2 with
//! 'anchor-not-unique'; no file write occurs.
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
fn acceptance_ac3_anchor_not_unique() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();
    let skill_dir = tmp.path().join(".claude").join("skills").join("build");
    std::fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");
    // Two identical lines — anchor is ambiguous.
    let original = "## Section\nduplicate anchor line\nduplicate anchor line\n## End\n";
    std::fs::write(&skill_md, original).unwrap();

    let out = Command::new(binary())
        .args(["--skill", "build", "--anchor", "duplicate anchor line", "--after", "new line"])
        .env("HOME", home)
        .output()
        .expect("failed to spawn");

    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 for ambiguous anchor, got {:?}",
        out.status.code()
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("anchor-not-unique"),
        "expected 'anchor-not-unique' in stderr, got: {stderr}"
    );
    // File must be unmodified.
    let content = std::fs::read_to_string(&skill_md).unwrap();
    assert_eq!(content, original, "file was modified despite anchor-not-unique guard");
}
