# Design Document: `branch-buddy init` Command

**Date**: 2026-07-27  
**Status**: Approved  
**Target Version**: `0.5.0`  

---

## 1. Overview

`branch-buddy init` scaffolds a clean, well-commented `.branchbuddy.toml` configuration file when one does not exist.

It supports:
- **Instant Template Mode** (default): Generates a standard annotated `.branchbuddy.toml` instantly.
- **Interactive Wizard Mode** (`--interactive`): Uses `dialoguer` prompts to walk the user through custom naming patterns, max lengths, default base branches, and tree display options.
- **Global Config Target** (`--global`): Generates `~/.config/branchbuddy/config.toml` instead of local repo `.branchbuddy.toml`.
- **Force Overwrite** (`--force`): Safely prompts before overwriting an existing config unless `--force` is specified.

---

## 2. CLI Interface

```rust
    /// Scaffold a new .branchbuddy.toml configuration file
    Init {
        /// Create global configuration file at ~/.config/branchbuddy/config.toml
        #[arg(global = true, long)]
        global: bool,

        /// Overwrite existing configuration file if present
        #[arg(short = 'f', long)]
        force: bool,

        /// Run interactive wizard to prompt for configuration options
        #[arg(short = 'i', long)]
        interactive: bool,
    },
```

---

## 3. Scaffolding Behavior

### 3.1 Target Path Resolution
- If `--global` is passed: `~/.config/branchbuddy/config.toml` (creates parent directories if missing).
- Otherwise: `.branchbuddy.toml` in the current working directory.

### 3.2 Overwrite Protection
If target file exists and `--force` is `false`:
- Output error: `⚠️  Configuration file '.branchbuddy.toml' already exists. Use --force to overwrite.`

### 3.3 Default Template Content
```toml
# Branch Buddy Configuration (.branchbuddy.toml)

[naming]
# Pattern template. Available tags: {slug}, {type}, {ticket}
pattern = "{type}/{ticket}-{slug}"
max_length = 63
prefix_separator = "/"
ticket_separator = "-"

[defaults]
# Default fallback base branch when --base is omitted
base = "main"

[tree]
# Hide legend at bottom of tree view
no_legend = false
```

---

## 4. Testing & Validation Plan

1. **Unit Tests**:
   - Test target path resolution (`global` vs `local`).
   - Test default template generation string validity.
2. **Clippy & Build**:
   - Run `just lint && just test && just install`.
