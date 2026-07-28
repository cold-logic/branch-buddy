# Design Document: Shell Completions

**Date**: 2026-07-28  
**Status**: Approved  
**Target Version**: `0.7.0`  

---

## 1. Overview

Add shell completion support for `branch-buddy` via `clap_complete`:

- **Build-time**: A `build.rs` generates completion scripts into `completions/` at compile time. Installable via `just install`.
- **Runtime**: A hidden `completions <shell>` subcommand lets users generate completions on-demand (for users who install via `cargo install`).

Supported shells: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

---

## 2. Dependencies

Add to `Cargo.toml`:

```toml
[build-dependencies]
clap_complete = "4"

[dependencies]
# existing deps...
clap_complete = "4"
```

---

## 3. Build-time Generation (`build.rs`)

Create `build.rs` at repo root:

```rust
use clap::CommandFactory;
use clap_complete::{Shell, generate_to};
use std::io::Error;

#[path = "src/cli.rs"]
mod cli;

fn main() -> Result<(), Error> {
    let out_dir = std::path::PathBuf::from("completions");
    std::fs::create_dir_all(&out_dir)?;

    let mut cmd = cli::Cli::command();
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Elvish, Shell::PowerShell] {
        generate_to(shell, &mut cmd, "branch-buddy", &out_dir)?;
    }
    Ok(())
}
```

> **Note:** Because all CLI types currently live in `src/main.rs`, the build script will use `clap_complete`'s `generate_to` via a small extraction pattern — or we generate at runtime from main and write to `completions/` in a `just completions` task instead (simpler, avoids splitting the file).

**Practical approach**: Use a `just completions` recipe that calls `branch-buddy completions <shell>` for each shell and redirects to `completions/`. This avoids splitting `src/main.rs` while still producing installable files.

---

## 4. Runtime Subcommand

```rust
/// Generate shell completion scripts (hidden from help)
#[command(hide = true)]
Completions {
    /// Shell to generate completions for
    shell: clap_complete::Shell,
},
```

Handler:
```rust
Commands::Completions { shell } => {
    let mut cmd = Cli::command();
    generate(*shell, &mut cmd, "branch-buddy", &mut std::io::stdout());
}
```

Usage:
```bash
branch-buddy completions zsh > ~/.zfunc/_branch-buddy
branch-buddy completions bash > /etc/bash_completion.d/branch-buddy
branch-buddy completions fish > ~/.config/fish/completions/branch-buddy.fish
```

---

## 5. `justfile` Updates

```makefile
# Generate completion scripts into completions/
completions:
    mkdir -p completions
    branch-buddy completions bash > completions/branch-buddy.bash
    branch-buddy completions zsh > completions/_branch-buddy
    branch-buddy completions fish > completions/branch-buddy.fish
    branch-buddy completions elvish > completions/branch-buddy.elv
    branch-buddy completions powershell > completions/branch-buddy.ps1

# Install completions for current user (zsh example, extend as needed)
install-completions: completions
    mkdir -p ~/.zfunc
    cp completions/_branch-buddy ~/.zfunc/_branch-buddy
    @echo "Add 'fpath=(~/.zfunc \$fpath)' and 'autoload -Uz compinit && compinit' to ~/.zshrc"
```

---

## 6. Testing

- Run `just completions` and verify all 5 files are generated and non-empty.
- Smoke-test: source zsh completion and verify `branch-buddy <TAB>` lists subcommands.
- CI: `just completions` added to lint/test pipeline.

---

## 7. Self-Review

- No placeholders: all shell targets named, file paths specified.
- Consistent with single-file design: no splitting of `src/main.rs`.
- Scope: narrow — one new subcommand, one new dep, two new just recipes.
