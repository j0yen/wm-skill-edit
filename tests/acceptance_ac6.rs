//! AC6 (MUST): The SKILL.md allowlist contains exactly:
//! autobuilder, build, self-review, dream, triage.
//! Each allowed slug accepts an edit; a 6th slug fails with 'skill-not-allowed'.
//!
//! Read-only: edit-agent must not modify this file.

use std::process::Command;

fn binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.ends_with("deps") { p.pop(); }
    p.join("wm-skill-edit")
}

fn run(args: &[&str], home: &str) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .env("HOME", home)
        .output()
        .expect("failed to spawn wm-skill-edit")
}

const ALLOWED: &[&str] = &["autobuilder", "build", "self-review", "dream", "triage"];

#[test]
fn acceptance_ac6_allowlist_coverage() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();

    // Each allowed slug should succeed when skill file exists.
    for slug in ALLOWED {
        let skill_dir = tmp.path().join(".claude").join("skills").join(slug);
        std::fs::create_dir_all(&skill_dir).unwrap();
        let skill_md = skill_dir.join("SKILL.md");
        std::fs::write(&skill_md, format!("## {slug}\nunique anchor for {slug}\n")).unwrap();

        let out = run(
            &["--skill", slug, "--anchor", &format!("unique anchor for {slug}"), "--after", "new line"],
            home,
        );
        assert!(
            out.status.success(),
            "allowed slug '{slug}' was rejected:\nstderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // A 6th slug must be rejected.
    let out = run(
        &["--skill", "verify", "--anchor", "foo", "--after", "bar"],
        home,
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 for out-of-allowlist slug 'verify'"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("skill-not-allowed"),
        "expected 'skill-not-allowed' for 'verify', got: {stderr}"
    );
}
