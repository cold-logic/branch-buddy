# `branch-buddy init` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `branch-buddy init` subcommand to scaffold `.branchbuddy.toml` configuration files (local or `--global`) with optional `--interactive` wizard and `--force` overwrite protection.

**Architecture:** Add `Init` variant to `Commands` enum in `src/main.rs`. Implement `handle_init` function using `dialoguer::Input` / `dialoguer::Confirm` for interactive mode, writing formatted TOML files.

**Tech Stack:** Rust (Edition 2024), `clap`, `dialoguer`, `colored`, `dirs`.

---

### Task 1: CLI Definition & `handle_init` Implementation with Unit Tests

**Files:**
- Modify: `src/main.rs`
- Test: `src/main.rs` (unit tests)

- [ ] **Step 1: Add `Init` subcommand to `Commands` enum in `src/main.rs`**

```rust
    /// Scaffold a new .branchbuddy.toml configuration file
    Init {
        /// Create global configuration file at ~/.config/branchbuddy/config.toml
        #[arg(long)]
        global: bool,

        /// Overwrite existing configuration file if present
        #[arg(short = 'f', long)]
        force: bool,

        /// Run interactive wizard to prompt for configuration options
        #[arg(short = 'i', long)]
        interactive: bool,
    },
```

- [ ] **Step 2: Implement `handle_init` function in `src/main.rs`**

```rust
fn handle_init(global: bool, force: bool, interactive: bool) -> Result<()> {
    let target_path = if global {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let dir = home.join(".config").join("branchbuddy");
        std::fs::create_dir_all(&dir)?;
        dir.join("config.toml")
    } else {
        std::path::PathBuf::from(".branchbuddy.toml")
    };

    if target_path.exists() && !force {
        return Err(anyhow!(
            "Configuration file '{}' already exists. Use --force to overwrite.",
            target_path.display()
        ));
    }

    let content = if interactive {
        use dialoguer::Input;

        let pattern: String = Input::new()
            .with_prompt("Naming pattern ({slug}, {type}, {ticket})")
            .default("{type}/{ticket}-{slug}".into())
            .interact_text()?;

        let max_length: String = Input::new()
            .with_prompt("Maximum slug length")
            .default("63".into())
            .interact_text()?;

        let default_base: String = Input::new()
            .with_prompt("Default base branch")
            .default("main".into())
            .interact_text()?;

        format!(
            "# Branch Buddy Configuration\n\n[naming]\npattern = \"{}\"\nmax_length = {}\n\n[defaults]\nbase = \"{}\"\n\n[tree]\nno_legend = false\n",
            pattern, max_length, default_base
        )
    } else {
        r#"# Branch Buddy Configuration (.branchbuddy.toml)

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
"#.to_string()
    };

    std::fs::write(&target_path, content)?;
    println!(
        "✨ Created configuration file at {}",
        target_path.display().to_string().green().bold()
    );

    Ok(())
}
```

- [ ] **Step 3: Wire `Commands::Init` in `main()` match block**

```rust
        Commands::Init {
            global,
            force,
            interactive,
        } => {
            handle_init(*global, *force, *interactive)?;
        }
```

- [ ] **Step 4: Add unit tests for `Init` command logic**

Under `mod tests` in `src/main.rs`:
```rust
#[test]
fn test_init_cli_parsing() {
    let cli = Cli::try_parse_from(["branch-buddy", "init", "--global", "-f", "-i"]).unwrap();
    if let Commands::Init { global, force, interactive } = cli.command {
        assert!(global);
        assert!(force);
        assert!(interactive);
    } else {
        panic!("Expected Commands::Init");
    }
}
```

- [ ] **Step 5: Run `just lint && just test && just install`**

Run: `just lint && just test && just install`
Expected: PASS (0 warnings).

- [ ] **Step 6: Commit & Push to `main` via `jj`**

```bash
jj describe -m "feat: implement branch-buddy init subcommand"
jj bookmark set main -r @
jj git push -b main
```
