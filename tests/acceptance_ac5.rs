//! AC5 (MUST): Every successful edit writes exactly one `SKILL.md.bak.<ts>`;
//! `--revert <skill>` restores the file byte-for-byte.
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

#[test]
fn acceptance_ac5_backup_and_revert() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_str().unwrap();
    let skill_dir = tmp.path().join(".claude").join("skills").join("triage");
    std::fs::create_dir_all(&skill_dir).unwrap();
    let skill_md = skill_dir.join("SKILL.md");
    let original = "## Triage\nstep 1: scan candidates\n## End\n";
    std::fs::write(&skill_md, original).unwrap();

    // Perform edit.
    let out = run(
        &["--skill", "triage", "--anchor", "step 1: scan candidates", "--after", "step 2: classify"],
        home,
    );
    assert!(out.status.success(), "edit failed: {}", String::from_utf8_lossy(&out.stderr));

    // Exactly one backup file created.
    let entries: Vec<_> = std::fs::read_dir(&skill_dir)
        .unwrap()
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy().into_owned();
            s.starts_with("SKILL.md.bak.")
        })
        .collect();
    assert_eq!(entries.len(), 1, "expected exactly 1 backup, found {}", entries.len());

    // Backup contains the original content.
    let bak_content = std::fs::read(&entries[0].path()).unwrap();
    assert_eq!(bak_content, original.as_bytes(), "backup content does not match original");

    // File is now modified.
    let modified = std::fs::read_to_string(&skill_md).unwrap();
    assert!(modified.contains("step 2: classify"), "edited content not present");

    // Revert.
    let out2 = run(&["--revert", "triage"], home);
    assert!(out2.status.success(), "revert failed: {}", String::from_utf8_lossy(&out2.stderr));

    // File is restored byte-for-byte.
    let restored = std::fs::read(&skill_md).unwrap();
    assert_eq!(
        restored,
        original.as_bytes(),
        "reverted content does not match original"
    );
}
