# AGENTS.md

## Project Overview

**Branch Buddy** is a lightweight Git companion CLI written in Rust. It provides:

1. **Persistent base-branch metadata** — stores `branch.<name>.base` in local `.git/config` so you always know where a branch originated.
2. **Human-friendly branch naming** — converts titles like `"Fix user signup flow"` into slugified branch names (`fix-user-signup-flow`).

Current version: `0.1.0` (early stage, single-file implementation).

## Tech Stack

- **Language**: Rust (Edition 2024)
- **Build system**: Cargo
- **Task runner**: Just (`justfile`)
- **CLI framework**: clap (derive macros)
- **Error handling**: anyhow
- **Git interaction**: subprocess calls to `git` (no libgit2)

## Project Structure

```
src/main.rs          # All application logic (~750 lines)
docs/branch-buddy-prd.md  # PRD / ADR with full requirements
justfile             # Build, test, lint, install recipes
Cargo.toml           # Package manifest and dependencies
README.md            # User-facing documentation
.tool-versions       # Tool version pins (just 1.50.0)
```

The codebase is a single `main.rs` file containing:
- CLI struct definitions (clap derive)
- Subcommand implementations (new, get-base, set-base, has-base, guess-base, tree, install-hooks, doctor)
- Helper functions for Git operations
- Unit tests at the bottom in `#[cfg(test)]`

## Commands

| Command | What it does |
|---------|-------------|
| `just build` | Debug build (`cargo build`) |
| `just release` | Release build (`cargo build --release`) |
| `just test` | Run tests (`cargo test`) |
| `just fmt` | Format code (`cargo fmt`) |
| `just lint` | Lint with clippy (`cargo clippy -- -D warnings`) |
| `just install` | Install locally (`cargo install --path .`) |
| `just run -- <args>` | Run with args (`cargo run -- <args>`) |

## Coding Conventions

- **Error handling**: Use `anyhow::Result<T>` with `.context()` for all fallible operations. Provide clear, user-facing error messages.
- **Output style**: Colorized terminal output via the `colored` crate. Use emojis for key actions (✨ created, 🌳 tree, 🔍 guessing, etc.).
- **Git operations**: Always shell out to `git` via `std::process::Command`. Parse stdout for results, check exit codes.
- **Validation**: Validate Git refs before writing metadata. Check for cycles in tree operations.
- **Platform handling**: Use `#[cfg(unix)]` for file permission operations (hook installation).
- **Interactive UX**: Use `dialoguer::FuzzySelect` when user input is ambiguous (e.g., detached HEAD).
- **Clippy**: All warnings are treated as errors (`-D warnings`). Fix all clippy suggestions.
- **Formatting**: Code must pass `cargo fmt` with default settings.

## Architecture Decisions

- **Single-file design**: All logic lives in `src/main.rs`. This is intentional for now given the project's scope.
- **Subprocess over libgit2**: Git interaction uses CLI subprocess calls for portability and simplicity.
- **Local-only metadata**: Base branch info is stored in local `.git/config` (not shared across clones).
- **Slugification**: Lowercase → replace non-alphanumeric with `-` → collapse dashes → trim → truncate to 63 chars. Falls back to `branch-<timestamp>` if empty.

## Testing

- Unit tests live in `src/main.rs` inside `#[cfg(test)] mod tests`.
- Current tests cover: slugification, tree building, cycle detection.
- Run with `just test` or `cargo test`.
- No integration test harness yet (would require temporary Git repos).

## Version Control

- The project uses **Jujutsu (jj)** alongside Git.
- `.gitignore` excludes `/target`.
- `Cargo.lock` is committed for reproducible builds.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `anyhow` | Ergonomic error handling |
| `clap` (derive) | CLI argument parsing |
| `colored` | Terminal color output |
| `dialoguer` (fuzzy-select) | Interactive prompts |
| `regex` | Slugification pattern matching |

## When Making Changes

1. Run `just fmt` before committing.
2. Run `just lint` — all clippy warnings must be resolved.
3. Run `just test` — all tests must pass.
4. If adding a new subcommand, follow the existing pattern: add a variant to `Commands` enum, handle it in the `match` block in `main()`, and add helper functions as needed.
5. Keep error messages user-friendly and contextual.
6. Refer to `docs/branch-buddy-prd.md` for the full requirements and design rationale.
