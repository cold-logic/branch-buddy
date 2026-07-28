# Design Document: `branch-buddy status`

**Date**: 2026-07-28  
**Status**: Approved  
**Target Version**: `0.8.0`  

---

## 1. Overview

`branch-buddy status` gives a branch health report for the current (or specified) branch — showing its recorded base, how many commits it's ahead/behind, how long ago it last diverged from the base, and a diff stat summary.

When the base has moved ahead, it surfaces a rebase suggestion. Once `branch-buddy sync` lands, the suggestion upgrades from `git rebase <base>` to `branch-buddy sync`.

---

## 2. CLI Interface

```
branch-buddy status [branch]

Arguments:
    branch    Target branch (defaults to current branch)

Options:
    --json    Emit structured JSON instead of human-readable output
```

---

## 3. Output Format

### 3.1 Human-Readable — Stale Branch

```
🌿 Branch:  feat/AUTH-101-login-flow
🌱 Base:    main (last synced ~3 days ago)
📦 Commits: 3 ahead of base

⚠  Base has moved: main is 2 commits ahead of your branch point.
   Run: git rebase main

Files changed vs base: 12 files, +340 −87 lines
```

### 3.2 Human-Readable — Fresh Branch

```
🌿 Branch:  feat/AUTH-101-login-flow
🌱 Base:    main (last synced ~2 hours ago)
📦 Commits: 3 ahead of base

✓  Base is current. No rebase needed.

Files changed vs base: 12 files, +340 −87 lines
```

### 3.3 JSON (`--json`)

```json
{
  "branch": "feat/AUTH-101-login-flow",
  "base": "main",
  "ahead": 3,
  "behind": 2,
  "stale": true,
  "last_synced_secs_ago": 259200,
  "diff_stat": {
    "files": 12,
    "insertions": 340,
    "deletions": 87
  }
}
```

---

## 4. Data Sources

| Field | Git command | Jujutsu equivalent |
|---|---|---|
| Current branch | `git branch --show-current` | `jj log -r @ -T "local_bookmarks"` |
| Base | `git config branch.<name>.base` | same (stored in `.git/config`) |
| Ahead count | `git rev-list --count <base>..<HEAD>` | `jj log -r "<base>..@ ~ <base>" --no-graph \| wc -l` |
| Behind count | `git rev-list --count <HEAD>..<base>` | `jj log -r "@..<base> ~ @" --no-graph \| wc -l` |
| Last synced | `git log -1 --format=%ct $(git merge-base HEAD <base>)` → epoch → human age | `jj log -r "ancestors(@) & ancestors(<base>)" --limit 1 -T "committer.timestamp()"` |
| Diff stat | `git diff --shortstat <base>..HEAD` | `jj diff --stat -r "<base>..@"` |

---

## 5. Edge Cases

| Situation | Behaviour |
|---|---|
| No base recorded | `⚠ No base recorded. Run: branch-buddy set-base <base>` |
| On trunk branch | `🪵 You are on the trunk branch — no base applies` |
| Detached HEAD (Git) | Error: `Not on a branch (detached HEAD). Checkout a branch first.` |
| 0 commits ahead | Show `0 commits ahead of base` (not an error) |
| Base ref doesn't exist locally | Error: `Base branch '<name>' not found locally. Fetch first.` |

---

## 6. Rebase Suggestion

- **Current**: Show `Run: git rebase <base>` (Git) or `Run: jj rebase -d <base>` (Jujutsu)
- **After `branch-buddy sync` lands**: Upgrade to `Run: branch-buddy sync`

---

## 7. Testing Plan

1. Unit tests:
   - `parse_diff_shortstat` — parse `git diff --shortstat` output into `(files, insertions, deletions)`
   - `human_age` — convert elapsed seconds to human-readable string (`~3 days ago`, `~2 hours ago`, `just now`)
2. `just lint && just test`
