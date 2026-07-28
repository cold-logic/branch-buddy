# `.branchbuddy.toml` Configuration Defaults Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement cascading `.branchbuddy.toml` configuration support (CLI flags > `--config <path>` > `.branchbuddy.toml` > `~/.config/branchbuddy/config.toml` > Built-in defaults) for project naming patterns, max slug length, default base branch, and tree display options.

**Architecture:** Add `serde` and `toml` to `Cargo.toml`. Add `Config` struct and `Config::load()` method in `src/main.rs`. Merge config preferences into `new_branch`, `print_tree`, and CLI parsing.

**Tech Stack:** Rust (Edition 2024), `serde`, `toml`, `clap`, `colored`.

---

### Task 1: Add `serde` & `toml` Dependencies and `Config` Data Structure with Unit Tests

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

- [ ] **Step 1: Add `serde` and `toml` to `Cargo.toml`**

Add dependencies:
```toml
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

- [ ] **Step 2: Define `Config` structs in `src/main.rs`**

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

- [ ] **Step 3: Write unit tests for partial TOML parsing and missing config fallback**

Under `mod tests`:
```rust
#[test]
fn test_config_partial_deserialization() {
    let toml_str = r#"
    [naming]
    max_length = 50

    [tree]
    no_legend = true
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.naming.as_ref().unwrap().max_length, Some(50));
    assert_eq!(config.naming.as_ref().unwrap().pattern, None);
    assert_eq!(config.tree.as_ref().unwrap().no_legend, Some(true));
    assert!(config.defaults.is_none());
}
```

- [ ] **Step 4: Run `just test` to verify unit test passes**

Run: `just test`
Expected: PASS

---

### Task 2: Implement Cascading Config Loading and Merge into Application Logic

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add `Config::load` and `Config::merge` helper methods in `src/main.rs`**

```rust
impl Config {
    pub fn load(explicit_path: Option<&std::path::Path>) -> Self {
        let mut config = Config::default();

        // 1. Global config (~/.config/branchbuddy/config.toml)
        if let Some(home) = dirs::home_dir() {
            let global_path = home.join(".config").join("branchbuddy").join("config.toml");
            if let Ok(content) = std::fs::read_to_string(global_path) {
                if let Ok(parsed) = toml::from_str::<Config>(&content) {
                    config.merge(parsed);
                }
            }
        }

        // 2. Repo root config (.branchbuddy.toml)
        let repo_path = std::path::Path::new(".branchbuddy.toml");
        if let Ok(content) = std::fs::read_to_string(repo_path) {
            if let Ok(parsed) = toml::from_str::<Config>(&content) {
                config.merge(parsed);
            }
        }

        // 3. Explicit config path from CLI
        if let Some(path) = explicit_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(parsed) = toml::from_str::<Config>(&content) {
                    config.merge(parsed);
                }
            }
        }

        config
    }

    pub fn merge(&mut self, other: Config) {
        if let Some(other_naming) = other.naming {
            let n = self.naming.get_or_insert_with(Default::default);
            if other_naming.pattern.is_some() { n.pattern = other_naming.pattern; }
            if other_naming.max_length.is_some() { n.max_length = other_naming.max_length; }
            if other_naming.prefix_separator.is_some() { n.prefix_separator = other_naming.prefix_separator; }
            if other_naming.ticket_separator.is_some() { n.ticket_separator = other_naming.ticket_separator; }
        }
        if let Some(other_defaults) = other.defaults {
            let d = self.defaults.get_or_insert_with(Default::default);
            if other_defaults.base.is_some() { d.base = other_defaults.base; }
        }
        if let Some(other_tree) = other.tree {
            let t = self.tree.get_or_insert_with(Default::default);
            if other_tree.no_legend.is_some() { t.no_legend = other_tree.no_legend; }
        }
    }
}
```

- [ ] **Step 2: Add global `--config` flag to `Cli` struct**

```rust
#[derive(Parser)]
#[command(name = "branch-buddy")]
struct Cli {
    /// Path to custom configuration file
    #[arg(global = true, long)]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}
```

- [ ] **Step 3: Integrate `Config` settings into `new_branch` and `print_tree`**

- Pass loaded config into `new_branch` for pattern formatting, max length, and default base.
- In `print_tree_with_mode`, fallback `no_legend` to `config.tree.and_then(|t| t.no_legend).unwrap_or(false)`.

- [ ] **Step 4: Run `just lint && just test && just install`**

Run: `just lint && just test && just install`
Expected: Clean pass (0 warnings).

- [ ] **Step 5: Commit & Push to `main` via `jj`**

```bash
jj describe -m "feat: implement .branchbuddy.toml repository configuration defaults"
jj bookmark set main -r @
jj git push -b main
```
