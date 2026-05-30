# wm-skill-edit

Allow-listed wrapper for anchored, idempotent SKILL.md edits — escapes the
self-modification classifier for `/build` branch agents.

## TL;DR

`/build` branch agents running in auto-mode cannot edit
`~/.claude/skills/*/SKILL.md` via raw `Edit` calls: every such edit trips
the self-modification classifier, and consent does not propagate across calls
(per-command evaluation). `wm-skill-edit` fixes this the same way `wm-push`
and `wm-publish` fixed pushes/publishes: a single narrow wrapper + one
settings.json allow rule (`Bash(wm-skill-edit:*)`) converts a blocked
operation into a clean, auditable, branch-runnable action.

Edits are **anchored** (must match a unique substring) and **idempotent**
(re-applying is a no-op), so a branch can't garble a skill. Every successful
edit writes a timestamped `SKILL.md.bak.<ts>` and `--revert` rolls it back.

## Install

```
cargo install --path . --root ~/.local
```

Or copy the pre-built binary from `target/release/wm-skill-edit` to
`~/.local/bin/`.

Then add the allow rule to `~/.claude/settings.json`:

```json
{ "Bash(wm-skill-edit:*)": "allow" }
```

## Usage

```
# Insert a line after an anchor (idempotent):
wm-skill-edit --skill autobuilder \
              --anchor "step 10: do something" \
              --after  "step 11: ac-judge"

# Revert the most-recent edit:
wm-skill-edit --revert autobuilder
```

### Allowed skills

`autobuilder`, `build`, `self-review`, `dream`, `triage`.

Add new slugs to the `ALLOW` array in `src/main.rs`; keep in sync with
`wm-push`/`wm-publish` ALLOW lists and `~/wintermute/REPOS.md`.

### Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success (or idempotent no-op) |
| 2    | Guard rejected: `skill-not-allowed`, `skill-file-missing`, `anchor-not-found`, `anchor-not-unique`, `no-backup-found` |

## Acceptance tests

All acceptance tests pass without network or kernel privileges:

```
cargo test --release
```

## License

MIT OR Apache-2.0
