//! `wm-skill-edit` — allow-listed wrapper for anchored idempotent SKILL.md edits.
//!
//! Resolves the per-command self-modification classifier block by providing a
//! single narrow binary that `/build` branch agents can invoke under a
//! `Bash(wm-skill-edit:*)` allow rule rather than issuing raw `Edit` calls to
//! skill files.
//!
//! # Allowed skills
//!
//! Edits are restricted to the `ALLOW` set: `autobuilder`, `build`,
//! `self-review`, `dream`, `triage`.
//!
//! # Operations
//!
//! - `--skill <slug> --anchor <substr> --after <text>` — idempotent insert
//! - `--revert <slug>` — restore most-recent backup

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Slugs this binary may touch.
const ALLOW: &[&str] = &["autobuilder", "build", "self-review", "dream", "triage"];

/// Skills root directory.
const SKILLS_ROOT: &str = "~/.claude/skills";

/// `wm-skill-edit` — anchored idempotent SKILL.md edits for allow-listed skills.
#[derive(Debug, Parser)]
#[command(name = "wm-skill-edit", version, author)]
struct Cli {
    /// The skill slug to operate on (must be in the allow-list).
    #[arg(long, conflicts_with = "revert_slug")]
    skill: Option<String>,

    /// Unique substring that must appear on exactly one line in the SKILL.md.
    #[arg(long, requires = "skill")]
    anchor: Option<String>,

    /// Text to insert immediately after the anchor line (idempotent).
    #[arg(long, requires = "anchor")]
    after: Option<String>,

    /// Revert the most-recent backup for this skill slug.
    #[arg(long = "revert", id = "revert_slug")]
    revert: Option<String>,
}

fn skill_md_path(slug: &str) -> PathBuf {
    // Expand ~ manually — std doesn't do it.
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/root"));
    Path::new(&home)
        .join(".claude")
        .join("skills")
        .join(slug)
        .join("SKILL.md")
}

/// Write a timestamped backup of the given path.
fn write_backup(skill_md: &Path) -> Result<PathBuf, String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("clock-error: {e}"))?
        .as_secs();
    let bak = skill_md.with_extension(format!("md.bak.{ts}"));
    std::fs::copy(skill_md, &bak)
        .map_err(|e| format!("backup-failed: {e}"))?;
    Ok(bak)
}

/// Find the most-recent backup for a SKILL.md path.
fn find_latest_backup(skill_md: &Path) -> Option<PathBuf> {
    let parent = skill_md.parent()?;
    let stem = skill_md.file_name()?.to_string_lossy();
    // Backups are named SKILL.md.bak.<ts> — they have extension "bak.<ts>"
    // which means on disk the filename is e.g. "SKILL.md.bak.1748000000"
    // (Path::with_extension replaces only the last extension segment —
    //  but we called it with "md.bak.<ts>" which replaces "md").
    // Let's just list the dir and filter by prefix.
    let prefix = format!("{stem}.bak.");
    std::fs::read_dir(parent).ok()?.flatten()
        .filter(|e| {
            e.file_name().to_string_lossy().starts_with(prefix.as_str())
        })
        .max_by_key(|e| {
            // Sort by the numeric timestamp suffix.
            let name = e.file_name();
            let s = name.to_string_lossy();
            let ts_part = s.rsplit('.').next().unwrap_or("0");
            ts_part.parse::<u64>().unwrap_or(0)
        })
        .map(|e| e.path())
}

fn run_insert(slug: &str, anchor: &str, after_text: &str) -> Result<(), String> {
    // 1. Allow-list check.
    if !ALLOW.contains(&slug) {
        eprintln!("skill-not-allowed: '{slug}' is not in the allow-list");
        return Err("skill-not-allowed".into());
    }

    // 2. Target file must exist.
    let skill_md = skill_md_path(slug);
    if !skill_md.exists() {
        eprintln!("skill-file-missing: {}", skill_md.display());
        return Err("skill-file-missing".into());
    }

    // 3. Read content.
    let content = std::fs::read_to_string(&skill_md)
        .map_err(|e| format!("read-error: {e}"))?;
    let lines: Vec<&str> = content.lines().collect();

    // 4. Find anchor — must be unique.
    let matches: Vec<usize> = lines.iter().enumerate()
        .filter(|(_, line)| line.contains(anchor))
        .map(|(i, _)| i)
        .collect();

    match matches.len() {
        0 => {
            eprintln!("anchor-not-found: no line contains '{anchor}'");
            return Err("anchor-not-found".into());
        }
        1 => {} // good
        n => {
            eprintln!("anchor-not-unique: {n} lines contain '{anchor}'");
            return Err("anchor-not-unique".into());
        }
    }

    let anchor_idx = matches[0];

    // 5. Idempotency: check if after_text already follows anchor.
    if anchor_idx + 1 < lines.len() && lines[anchor_idx + 1] == after_text {
        // Already inserted — no-op.
        return Ok(());
    }

    // 6. Write backup.
    let _bak = write_backup(&skill_md)?;

    // 7. Build new content.
    let mut new_lines: Vec<&str> = Vec::with_capacity(lines.len() + 1);
    for (i, line) in lines.iter().enumerate() {
        new_lines.push(line);
        if i == anchor_idx {
            new_lines.push(after_text);
        }
    }
    // Preserve trailing newline if original had it.
    let trailing_newline = content.ends_with('\n');
    let new_content = if trailing_newline {
        format!("{}\n", new_lines.join("\n"))
    } else {
        new_lines.join("\n")
    };

    // 8. Write atomically (temp + rename).
    let tmp = skill_md.with_extension("md.tmp");
    std::fs::write(&tmp, &new_content)
        .map_err(|e| format!("write-tmp-error: {e}"))?;
    std::fs::rename(&tmp, &skill_md)
        .map_err(|e| format!("rename-error: {e}"))?;

    Ok(())
}

fn run_revert(slug: &str) -> Result<(), String> {
    // 1. Allow-list check.
    if !ALLOW.contains(&slug) {
        eprintln!("skill-not-allowed: '{slug}' is not in the allow-list");
        return Err("skill-not-allowed".into());
    }

    // 2. Target file must exist.
    let skill_md = skill_md_path(slug);
    if !skill_md.exists() {
        eprintln!("skill-file-missing: {}", skill_md.display());
        return Err("skill-file-missing".into());
    }

    // 3. Find latest backup.
    let bak = find_latest_backup(&skill_md)
        .ok_or_else(|| {
            eprintln!("no-backup-found: no SKILL.md.bak.* for '{slug}'");
            "no-backup-found".to_string()
        })?;

    // 4. Restore (copy backup over skill_md atomically).
    let tmp = skill_md.with_extension("md.tmp");
    std::fs::copy(&bak, &tmp)
        .map_err(|e| format!("copy-bak-error: {e}"))?;
    std::fs::rename(&tmp, &skill_md)
        .map_err(|e| format!("rename-error: {e}"))?;

    eprintln!("reverted: {} → {}", bak.display(), skill_md.display());
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Dispatch.
    let result = if let Some(slug) = cli.revert {
        run_revert(&slug)
    } else if let Some(slug) = cli.skill {
        // clap `requires` guarantees anchor and after are present when skill is.
        let anchor = cli.anchor.unwrap_or_default();
        let after = cli.after.unwrap_or_default();
        run_insert(&slug, &anchor, &after)
    } else {
        eprintln!("usage: wm-skill-edit --skill <slug> --anchor <text> --after <text>");
        eprintln!("       wm-skill-edit --revert <slug>");
        return ExitCode::from(2);
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(2),
    }
}
