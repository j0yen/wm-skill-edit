//! AC1 (MUST): `--skill autobuilder --anchor <KNOWN-LINE> --after <NEW>` inserts
//! `<NEW>` once; a second identical invocation is a no-op (exit 0, file unchanged).
//!
//! Read-only: edit-agent must not modify this file.

use std::process::Command;

fn binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.ends_with("deps") { p.pop(); }
    p.join("wm-skill-edit")
}

fn run(args: &[&str], env_home: &str) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .env("HOME", env_home)
        .output()
        .expect("failed to spawn wm-skill-edit")
}

#[test]
fn acceptance_ac1_insert_idempotent() {
    // Set up a fake skill directory under a temp HOME.
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();
    let skill_dir = tmp.path().join(".claude").join("skills").join("autobuilder");
    std::fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");
    std::fs::write(&skill_md, "## Stage 3\nstep 10: do something\n## Stage 4\n").unwrap();

    // First invocation: should insert.
    let out1 = run(
        &["--skill", "autobuilder", "--anchor", "step 10: do something", "--after", "step 11: ac-judge"],
        home,
    );
    assert!(
        out1.status.success(),
        "first invocation failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out1.stdout),
        String::from_utf8_lossy(&out1.stderr),
    );

    let content_after_first = std::fs::read_to_string(&skill_md).unwrap();
    assert!(
        content_after_first.contains("step 11: ac-judge"),
        "text not inserted: {content_after_first}"
    );
    // Count occurrences.
    assert_eq!(
        content_after_first.matches("step 11: ac-judge").count(),
        1,
        "expected exactly one insertion"
    );

    // Second invocation: must be a no-op (exit 0, file unchanged).
    let content_before_second = std::fs::read_to_string(&skill_md).unwrap();
    let out2 = run(
        &["--skill", "autobuilder", "--anchor", "step 10: do something", "--after", "step 11: ac-judge"],
        home,
    );
    assert!(
        out2.status.success(),
        "second (idempotent) invocation failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr),
    );
    let content_after_second = std::fs::read_to_string(&skill_md).unwrap();
    assert_eq!(
        content_before_second, content_after_second,
        "second invocation mutated the file — not idempotent"
    );
    assert_eq!(
        content_after_second.matches("step 11: ac-judge").count(),
        1,
        "second invocation added a duplicate"
    );
}
