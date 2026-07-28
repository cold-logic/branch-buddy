# Design Document: `branch-buddy log` Subcommand

**Date**: 2026-07-27  
**Status**: Approved  
**Target Version**: `0.3.0`  

---

## 1. Overview

`branch-buddy log` provides a focused, noise-free commit history view for your current branch or specified feature stack.

Instead of showing the entire repository history or noisy merge commits, `branch-buddy log` isolates commits between the branch's declared base branch and `@` (`base..@`), highlighting Change IDs, commit hashes, ticket IDs, and commit messages.

---

## 2. Command Interface & Options

```rust
Log {
    /// Target branch (defaults to current branch/bookmark)
    branch: Option<String>,

    /// Include commits across all parent base branches up to trunk()
    #[arg(long, alias = "all-ancestors")]
    stack: bool,

    /// Show file diff statistics for each commit
    #[arg(long)]
    stat: bool,

    /// Limit the number of commits displayed
    #[arg(short = 'n', long)]
    limit: Option<usize>,
}
```

---

## 3. Dual Engine Behavior

### 3.1 Git Mode
- Default query: `git log <base>..@ --oneline` (or with `--stat` if `--stat` flag passed).
- If `--stack` is set, walks up `.git/config` parent base branches to the root base before calling `git log`.

### 3.2 Jujutsu (`jj`) Mode
- Default revset query: `jj log -r '<base>::@ & ~<base>' --no-graph`
- Formatted output displays short commit hash, `[change: <id>]`, and commit summary.
- If `--stack` is set, queries `jj log -r 'trunk()::@ & ~trunk()'` to show the full stack ancestry above `trunk()`.

---

## 4. Output Design

```text
🪵 Stack log for SF-10891-child-products-custom-exports (base: development)
├─ 5bff55b [ztnsmvzo] fix: include child products in custom exports (SF-10891) (2 hours ago)
└─ 2645479 [xuxlppvx] DO-244: Upgrade API-v2 local PostgreSQL stacks to 17 (#3161)
```

---

## 5. Testing & Validation Plan

1. **Unit Tests**:
   - Test log range parsing for Git mode (`base..branch`).
   - Test revset string generation for `jj` mode (`base::branch & ~base`).
2. **Clippy & Build**:
   - Run `just lint && just test`.
