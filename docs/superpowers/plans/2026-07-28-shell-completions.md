# Shell Completions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add shell completion support to `branch-buddy` via `clap_complete`: a hidden `completions <shell>` subcommand for runtime generation, plus `just completions` and `just install-completions` recipes for build-time file generation.

**Architecture:** Single-task implementation — add `clap_complete` dependency, add `Completions` subcommand variant, wire handler, update `justfile`.

**Tech Stack:** Rust (Edition 2024), `clap`, `clap_complete`.

---

### Task 1: Implement Shell Completions

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Modify: `justfile`

- [ ] **Step 1: Add `clap_complete` to `Cargo.toml`**

```toml
[dependencies]
# ... existing deps ...
clap_complete = "4"
```

- [ ] **Step 2: Add `use` import in `src/main.rs`**

At the top of `src/main.rs`, add:
```rust
use clap::CommandFactory;
use clap_complete::{Shell, generate};
```

- [ ] **Step 3: Add `Completions` hidden subcommand to `Commands` enum**

```rust
    /// Generate shell completion scripts
    #[command(hide = true)]
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
```

- [ ] **Step 4: Wire `Commands::Completions` in `main()` match block**

```rust
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "branch-buddy", &mut std::io::stdout());
        }
```

- [ ] **Step 5: Add `completions` and `install-completions` recipes to `justfile`**

```makefile
# Generate shell completion scripts into completions/
completions: install
    mkdir -p completions
    branch-buddy completions bash > completions/branch-buddy.bash
    branch-buddy completions zsh > completions/_branch-buddy
    branch-buddy completions fish > completions/branch-buddy.fish
    branch-buddy completions elvish > completions/branch-buddy.elv
    branch-buddy completions powershell > completions/branch-buddy.ps1
    @echo "✨ Completion scripts written to completions/"

# Install zsh completions for current user
install-completions: completions
    mkdir -p ~/.zfunc
    cp completions/_branch-buddy ~/.zfunc/_branch-buddy
    @echo "✨ Zsh completions installed to ~/.zfunc/_branch-buddy"
    @echo "   Add to ~/.zshrc if not already present:"
    @echo "     fpath=(~/.zfunc \$$fpath)"
    @echo "     autoload -Uz compinit && compinit"
```

- [ ] **Step 6: Run `just lint && just test`**

Expected: PASS (0 warnings, all tests green).

- [ ] **Step 7: Run `just completions` to verify generation**

Expected: 5 files created in `completions/` directory, each non-empty.

- [ ] **Step 8: Commit & Push to `main` via `jj`**

```bash
jj describe -m "feat: add shell completions via clap_complete"
jj bookmark set main -r @
jj git push -b main
jj new
```
