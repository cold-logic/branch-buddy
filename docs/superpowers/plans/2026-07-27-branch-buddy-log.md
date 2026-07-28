# `branch-buddy log` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the `branch-buddy log` subcommand to show a focused commit history between a branch and its base (or full stack via `--stack`), working across Git and Jujutsu (`jj`).

**Architecture:** Add a `Log` variant to `Commands` enum in `src/main.rs`. Implement `handle_log` helper that inspects `VcsMode` to execute Git or Jujutsu log queries with `--stat`, `--stack`, and `--limit` support.

**Tech Stack:** Rust (Edition 2024), `clap`, `std::process::Command`, `colored`.

---

### Task 1: CLI Definition & Unit Tests for `branch-buddy log` Argument Parsing

**Files:**
- Modify: `src/main.rs`
- Test: `src/main.rs` (unit tests)

- [ ] **Step 1: Write unit test for `Log` command argument parsing**

In `src/main.rs` under `mod tests`:
```rust
#[test]
fn test_log_revset_formatting() {
    let base = "development";
    let branch = "feature-x";
    let git_range = format!("{}..{}", base, branch);
    assert_eq!(git_range, "development..feature-x");

    let jj_revset = format!("({})..({}) & ~({})", base, branch, base);
    assert_eq!(jj_revset, "(development)..(feature-x) & ~(development)");
}
```

- [ ] **Step 2: Add `Log` subcommand to `Commands` enum**

In `src/main.rs`:
```rust
    /// Show focused commit log between branch and base (or stack)
    Log {
        /// Target branch (defaults to current branch)
        branch: Option<String>,

        /// Include commits across all parent base branches up to trunk()
        #[arg(long, alias = "all-ancestors")]
        stack: bool,

        /// Show file diff statistics for each commit
        #[arg(long)]
        stat: bool,

        /// Limit number of commits displayed
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },
```

- [ ] **Step 3: Run `just test` to verify unit test passes**

Run: `just test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
jj describe -m "feat: add Log subcommand CLI definitions and tests"
```

---

### Task 2: Implement `handle_log` for Git and Jujutsu Modes

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement `handle_log` function in `src/main.rs`**

```rust
fn handle_log(
    branch: Option<&str>,
    stack: bool,
    stat: bool,
    limit: Option<usize>,
) -> Result<()> {
    let mode = VcsMode::detect();
    let target = match branch {
        Some(b) => b.to_string(),
        None => match mode {
            VcsMode::Git => current_branch()?,
            VcsMode::Jj => get_current_jj_bookmark_or_at(),
        },
    };

    let base = get_base(Some(&target)).unwrap_or_else(|_| match mode {
        VcsMode::Git => "main".to_string(),
        VcsMode::Jj => "trunk()".to_string(),
    });

    println!(
        "{}",
        format!("🪵 Log for {} (base: {}):", target.green(), base.blue()).bold()
    );

    match mode {
        VcsMode::Git => {
            let range = if stack {
                format!("main..{}", target)
            } else {
                format!("{}..{}", base, target)
            };
            let mut args = vec!["log", &range, "--oneline", "--color=always"];
            let limit_str = limit.map(|n| n.to_string());
            if let Some(ref l) = limit_str {
                args.push("-n");
                args.push(l);
            }
            if stat {
                args.push("--stat");
            }
            let output = Command::new("git").args(args).output()?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        VcsMode::Jj => {
            let revset = if stack {
                format!("trunk()::({}) & ~trunk()", target)
            } else {
                format!("({})::({}) & ~({})", base, target, base)
            };
            let mut args = vec![
                "log",
                "-r",
                &revset,
                "--no-graph",
                "-T",
                r#"commit_id.short() ++ " [" ++ change_id.short() ++ "] " ++ description.first_line() ++ " (" ++ author.name() ++ ")\n""#,
            ];
            let limit_str = limit.map(|n| n.to_string());
            if let Some(ref l) = limit_str {
                args.push("-n");
                args.push(l);
            }
            if stat {
                args.push("--stat");
            }
            let output = Command::new("jj").args(args).output()?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Wire `Commands::Log` in `main()` match block**

```rust
        Commands::Log {
            branch,
            stack,
            stat,
            limit,
        } => {
            handle_log(branch.as_deref(), *stack, *stat, *limit)?;
        }
```

- [ ] **Step 3: Run `just lint && just test && just install`**

Run: `just lint && just test && just install`
Expected: Clean pass with 0 warnings.

- [ ] **Step 4: Commit & Push to `main` via `jj`**

```bash
jj describe -m "feat: implement branch-buddy log subcommand"
jj bookmark set main -r @
jj git push -b main
```
