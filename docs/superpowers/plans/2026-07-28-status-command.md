# `branch-buddy status` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `branch-buddy status` — a branch health report showing base, ahead/behind counts, merge-base age, diff stat, and a stale/fresh indicator with rebase suggestion.

**Architecture:** Two tasks — (1) CLI definition + pure helper functions with unit tests, (2) `handle_status` implementation for both Git and Jujutsu modes, wired into `main()`.

**Tech Stack:** Rust (Edition 2024), `clap`, `colored`, subprocess `git`/`jj`.

---

### Task 1: CLI Definition & Helper Functions with Unit Tests

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add `Status` subcommand to `Commands` enum**

```rust
    /// Show branch health report (base, ahead/behind, staleness, diff stat)
    Status {
        /// Target branch (defaults to current branch)
        branch: Option<String>,

        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
```

- [ ] **Step 2: Implement `human_age(secs: u64) -> String` helper**

```rust
fn human_age(secs: u64) -> String {
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("~{} minutes ago", secs / 60)
    } else if secs < 86400 {
        format!("~{} hours ago", secs / 3600)
    } else {
        format!("~{} days ago", secs / 86400)
    }
}
```

- [ ] **Step 3: Implement `parse_diff_shortstat(output: &str) -> (usize, usize, usize)` helper**

Parses `git diff --shortstat` output like `" 12 files changed, 340 insertions(+), 87 deletions(-)"` into `(files, insertions, deletions)`.

```rust
fn parse_diff_shortstat(output: &str) -> (usize, usize, usize) {
    let files = output
        .split_whitespace()
        .zip(output.split_whitespace().skip(1))
        .find(|(_, b)| b.starts_with("file"))
        .and_then(|(a, _)| a.parse().ok())
        .unwrap_or(0);
    let insertions = output
        .split_whitespace()
        .zip(output.split_whitespace().skip(1))
        .find(|(_, b)| b.starts_with("insertion"))
        .and_then(|(a, _)| a.parse().ok())
        .unwrap_or(0);
    let deletions = output
        .split_whitespace()
        .zip(output.split_whitespace().skip(1))
        .find(|(_, b)| b.starts_with("deletion"))
        .and_then(|(a, _)| a.parse().ok())
        .unwrap_or(0);
    (files, insertions, deletions)
}
```

- [ ] **Step 4: Add unit tests**

```rust
#[test]
fn test_human_age() {
    assert_eq!(human_age(30), "just now");
    assert_eq!(human_age(90), "~1 minutes ago");
    assert_eq!(human_age(7200), "~2 hours ago");
    assert_eq!(human_age(259200), "~3 days ago");
}

#[test]
fn test_parse_diff_shortstat() {
    let s = " 12 files changed, 340 insertions(+), 87 deletions(-)";
    assert_eq!(parse_diff_shortstat(s), (12, 340, 87));

    // Only insertions (no deletions)
    let s2 = " 3 files changed, 10 insertions(+)";
    assert_eq!(parse_diff_shortstat(s2), (3, 10, 0));

    // Empty (no changes)
    assert_eq!(parse_diff_shortstat(""), (0, 0, 0));
}
```

- [ ] **Step 5: Run `just lint && just test`**

Expected: PASS (all tests green, 0 clippy warnings).

---

### Task 2: Implement `handle_status` and Wire into `main()`

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement `handle_status` for Git mode**

```rust
fn handle_status(branch: Option<&str>, json: bool) -> Result<()> {
    let mode = VcsMode::detect();

    match mode {
        VcsMode::Git => {
            let current = match branch {
                Some(b) => b.to_string(),
                None => current_branch()?,
            };

            let base = git_config_get(&format!("branch.{}.base", current))?;

            // Ahead / behind
            let ahead: usize = Command::new("git")
                .args(["rev-list", "--count", &format!("{}..HEAD", base)])
                .output()?
                .stdout
                .split(|&b| b == b'\n')
                .next()
                .and_then(|s| std::str::from_utf8(s).ok())
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            let behind: usize = Command::new("git")
                .args(["rev-list", "--count", &format!("HEAD..{}", base)])
                .output()?
                .stdout
                .split(|&b| b == b'\n')
                .next()
                .and_then(|s| std::str::from_utf8(s).ok())
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            // Merge-base age
            let merge_base = String::from_utf8(
                Command::new("git")
                    .args(["merge-base", "HEAD", &base])
                    .output()?
                    .stdout,
            )?;
            let merge_base = merge_base.trim();

            let mb_epoch: u64 = String::from_utf8(
                Command::new("git")
                    .args(["log", "-1", "--format=%ct", merge_base])
                    .output()?
                    .stdout,
            )?
            .trim()
            .parse()
            .unwrap_or(0);

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let age_secs = now.saturating_sub(mb_epoch);

            // Diff stat
            let diff_out = String::from_utf8(
                Command::new("git")
                    .args(["diff", "--shortstat", &format!("{}..HEAD", base)])
                    .output()?
                    .stdout,
            )?;
            let (files, ins, dels) = parse_diff_shortstat(diff_out.trim());

            let stale = behind > 0;

            if json {
                println!(
                    r#"{{"branch":"{}","base":"{}","ahead":{},"behind":{},"stale":{},"last_synced_secs_ago":{},"diff_stat":{{"files":{},"insertions":{},"deletions":{}}}}}"#,
                    current, base, ahead, behind, stale, age_secs, files, ins, dels
                );
            } else {
                println!("🌿 Branch:  {}", current.green().bold());
                println!(
                    "🌱 Base:    {} (last synced {})",
                    base.blue(),
                    human_age(age_secs)
                );
                println!("📦 Commits: {} ahead of base", ahead);
                println!();
                if stale {
                    println!(
                        "{}  Base has moved: {} is {} commits ahead of your branch point.",
                        "⚠".yellow(),
                        base.blue(),
                        behind
                    );
                    println!("   Run: {}", format!("git rebase {}", base).cyan());
                } else {
                    println!("{}  Base is current. No rebase needed.", "✓".green());
                }
                println!();
                println!(
                    "Files changed vs base: {} files, +{} −{} lines",
                    files, ins, dels
                );
            }

            Ok(())
        }
        VcsMode::Jj => {
            // Jujutsu mode: similar but uses jj commands
            let current = match branch {
                Some(b) => b.to_string(),
                None => {
                    let out = Command::new("jj")
                        .args(["log", "-r", "@", "-T", "local_bookmarks", "--no-graph"])
                        .output()?
                        .stdout;
                    String::from_utf8(out)?.trim().to_string()
                }
            };

            let base = git_config_get(&format!("branch.{}.base", current))?;

            let ahead: usize = String::from_utf8(
                Command::new("jj")
                    .args(["log", "-r", &format!("{}..@ ~ {}", base, base), "--no-graph", "-T", "\"x\n\""])
                    .output()?
                    .stdout,
            )?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();

            let behind: usize = String::from_utf8(
                Command::new("jj")
                    .args(["log", "-r", &format!("@..{} ~ @", base), "--no-graph", "-T", "\"x\n\""])
                    .output()?
                    .stdout,
            )?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();

            let stale = behind > 0;

            // Use git merge-base for age (jj repos have git backend)
            let merge_base = String::from_utf8(
                Command::new("git")
                    .args(["merge-base", "HEAD", &base])
                    .output()
                    .unwrap_or_default()
                    .stdout,
            )
            .unwrap_or_default();
            let merge_base = merge_base.trim();

            let mb_epoch: u64 = String::from_utf8(
                Command::new("git")
                    .args(["log", "-1", "--format=%ct", merge_base])
                    .output()
                    .unwrap_or_default()
                    .stdout,
            )
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let age_secs = now.saturating_sub(mb_epoch);

            // Diff stat via git diff
            let diff_out = String::from_utf8(
                Command::new("git")
                    .args(["diff", "--shortstat", &format!("{}..HEAD", base)])
                    .output()
                    .unwrap_or_default()
                    .stdout,
            )
            .unwrap_or_default();
            let (files, ins, dels) = parse_diff_shortstat(diff_out.trim());

            if json {
                println!(
                    r#"{{"branch":"{}","base":"{}","ahead":{},"behind":{},"stale":{},"last_synced_secs_ago":{},"diff_stat":{{"files":{},"insertions":{},"deletions":{}}}}}"#,
                    current, base, ahead, behind, stale, age_secs, files, ins, dels
                );
            } else {
                println!("🌿 Branch:  {}", current.green().bold());
                println!(
                    "🌱 Base:    {} (last synced {})",
                    base.blue(),
                    human_age(age_secs)
                );
                println!("📦 Commits: {} ahead of base", ahead);
                println!();
                if stale {
                    println!(
                        "{}  Base has moved: {} is {} commits ahead of your branch point.",
                        "⚠".yellow(),
                        base.blue(),
                        behind
                    );
                    println!("   Run: {}", format!("jj rebase -d {}", base).cyan());
                } else {
                    println!("{}  Base is current. No rebase needed.", "✓".green());
                }
                println!();
                println!(
                    "Files changed vs base: {} files, +{} −{} lines",
                    files, ins, dels
                );
            }

            Ok(())
        }
    }
}
```

- [ ] **Step 2: Wire `Commands::Status` in `main()` match block**

```rust
        Commands::Status { branch, json } => {
            handle_status(branch.as_deref(), *json)?;
        }
```

- [ ] **Step 3: Run `just lint && just test && just install`**

Expected: PASS.

- [ ] **Step 4: Smoke-test manually**

Run `branch-buddy status` in the test-bed repo and verify human-readable output.  
Run `branch-buddy status --json` and verify JSON output is valid.

- [ ] **Step 5: Commit & Push to `main` via `jj`**

```bash
jj describe -m "feat: implement branch-buddy status command"
jj bookmark set main -r @
jj git push -b main
jj new
```
