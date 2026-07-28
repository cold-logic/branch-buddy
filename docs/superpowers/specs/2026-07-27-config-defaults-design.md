# Design Document: Repository Config Defaults (`.branchbuddy.toml`)

**Date**: 2026-07-27  
**Status**: Approved  
**Target Version**: `0.4.0`  

---

## 1. Overview

Branch Buddy configuration allows repositories and developers to define project-wide naming patterns, maximum slug lengths, default base branches, and tree display options via a simple TOML configuration file.

Config resolution cascades in the following precedence order (highest priority first):
1. **Explicit CLI arguments/flags** (e.g. `--base`, `--type`, `--ticket`, `--no-legend`)
2. **Explicit config flag** (`--config <path>`)
3. **Repository config** (`.branchbuddy.toml` at repo root / working dir)
4. **Global user config** (`~/.config/branchbuddy/config.toml`)
5. **Built-in CLI defaults**

---

## 2. Configuration Schema (`.branchbuddy.toml`)

```toml
[naming]
# Supported template tags: {slug}, {type}, {ticket}
pattern = "{type}/{ticket}-{slug}"
max_length = 63
prefix_separator = "/"
ticket_separator = "-"

[defaults]
# Default fallback base branch when --base is omitted and not in a branch
base = "main"

[tree]
# Suppress the legend output at the bottom of `branch-buddy tree`
no_legend = false
```

---

## 3. Implementation Plan & Data Structures

Add `serde` and `toml` dependencies to `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

Config struct in `src/main.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub naming: Option<NamingConfig>,
    pub defaults: Option<DefaultsConfig>,
    pub tree: Option<TreeConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamingConfig {
    pub pattern: Option<String>,
    pub max_length: Option<usize>,
    pub prefix_separator: Option<String>,
    pub ticket_separator: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DefaultsConfig {
    pub base: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TreeConfig {
    pub no_legend: Option<bool>,
}
```

---

## 4. Testing & Validation Plan

1. **Unit Tests**:
   - Test TOML deserialization of valid `.branchbuddy.toml` samples.
   - Test cascade priority logic (Explicit > Repo > Global > Default).
2. **Clippy & Build**:
   - Run `just lint && just test && just install`.
