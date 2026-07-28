# Branch Buddy 🌳🤝

**Branch Buddy** is a lightweight Git & Jujutsu (`jj`) companion CLI that brings persistent parent/base branch metadata to your local version control workflows, while making branch/bookmark creation from human-readable titles completely effortless.

Git does not naturally record explicit parent/base branch metadata—it only tracks a commit graph. While Jujutsu natively tracks commit parentage in a DAG, managing named branch/bookmark lineage metadata across Git and Jujutsu can still be cumbersome. Branch Buddy bridges this gap by persistently storing `branch.<name>.base` records in `.git/config` at the moment of branch/bookmark creation, giving you uniform lineage tracking, ancestry tree visualization, and focused commit logs across both Git and Jujutsu environments.

---

## 🌟 Key Features

- 🧠 **Persistent Base Metadata**: Never forget where a branch originated. Branch Buddy explicitly tracks parent branch relationships in `.git/config`.
- 🦎 **Dual Engine Support (Git + Jujutsu `jj`)**: Seamlessly auto-detects Jujutsu (`.jj`) or Git environments. In Jujutsu mode, it creates changes, attaches bookmarks, formats Change IDs (`[change: <id>]`), and dynamically resolves `trunk()`.
- 🗣️ **Human-Friendly Naming**: Just type `"Fix user signup flow"` and Branch Buddy generates clean, unique branch slugs (`fix-user-signup-flow` or `feature/AUTH-101-fix-user-signup-flow`).
- 🌳 **Ancestry Trees (`branch-buddy tree`)**: Visualize stacked branches and bookmarks with clear parent-child lineages, feature icons (🌿), and dynamic trunk indicators (🪵).
- 🪵 **Focused Stack Logs (`branch-buddy log`)**: View noise-free commit history between your branch and its base (`base..@`), or pass `--stack` to view changes across all parent branches up to `trunk()`.
- 🔮 **Legacy Guessing (`branch-buddy guess-base`)**: Automatically estimate bases for legacy branches created before installing Branch Buddy.
- 🩺 **Health Checks (`branch-buddy doctor`)**: Diagnose and heal broken or orphaned base-branch chains when parent branches are deleted.
- 📊 **Branch Health Report (`branch-buddy status`)**: See at a glance whether your branch is ahead/behind its base, how stale it is, and how many files have changed.
- ⚙️ **Repository Configuration (`.branchbuddy.toml`)**: Check in project-level defaults for naming patterns, default base branches, and tree legend preferences.
- ⌨️ **Shell Completions**: Generate tab-completion scripts for bash, zsh, fish, PowerShell, and Elvish.

---

## 📦 Installation

### Default Installation (via Cargo)

Install the latest release directly from GitHub:

```bash
cargo install --git https://github.com/cold-logic/branch-buddy
```

### Local Installation

Clone and install locally:

```bash
cargo install --path .
```

---

## 🚀 Quick Start (Git & Jujutsu Aliases)

Branch Buddy feels like a native extension of Git and Jujutsu.

### Git Aliases (`~/.gitconfig`)

```ini
[alias]
  # Create a branch from human-readable title (base = current branch)
  bb = "!branch-buddy new"

  # Conventional feature branch off main/development
  cobb = "!f() { branch-buddy new \"$1\" --type feature; }; f"

  # Base operations & ancestry tree
  base-branch = "!branch-buddy get-base"
  set-base = "!branch-buddy set-base"
  tree = "!branch-buddy tree"
  bblog = "!branch-buddy log"
```

---

## 🛠️ Usage

### 1. Creating Branches & Bookmarks (`new`)

Create branches from human-readable titles. Branch Buddy automatically slugifies the title, ensures uniqueness, and saves the base metadata.

```bash
$ branch-buddy new "Improve search UX" --type feature --ticket TKT-123
✨ Created branch: feature/TKT-123-improve-search-ux
🌱 Base: main
```

**Jujutsu (`jj`) Repositories:**
In a `jj` repo, `branch-buddy new` creates a new change (`jj new <base> -m "<title>"`), creates a matching `jj bookmark`, and sets base metadata automatically.

**Options:**
- `--base <branch>`: Explicit base branch (defaults to current branch/bookmark).
- `--type <type>`: Prefix (e.g. `feature`, `bugfix`).
- `--ticket <id>`: Issue tracker ID (e.g. `AUTH-101`).
- `--dry-run`: View slug without creating the branch/bookmark.
- `--no-checkout`: Create branch without switching your working copy.

---

### 2. Ancestry Tree Visualization (`tree`)

Inspect stacked branches and their parent lineage:

```bash
$ branch-buddy tree
🌳 Branch Ancestry (child → parent base):
🌿 bugfix/AUTH-103-token-refresh-fix [change: oypsursxmvlr]
└── 🌿 feature/AUTH-102-oauth2-google-support [change: tsylmxxwtpyq]
    └── 🌿 feature/AUTH-101-feature-auth-api [change: nvqxrpwvlywn]
        └── 🪵 main [change: tpvkxmlqzusl]

Legend: 🌿 Branch/Feature | 🪵 Trunk/Main Target
```

*Pass `--no-legend` to suppress the legend at the bottom.*

---

### 3. Focused Stack Commits Log (`log`)

View commits on your current feature branch relative to its base:

```bash
$ branch-buddy log
🪵 Log for feature/TKT-123-improve-search-ux (base: development):
5ca2075 [ztnsmvzo] feat(search): optimize query performance (TKT-123) (Dev User)
```

**Options:**
- `--stack` / `--all-ancestors`: Show commits across all stacked parent branches up to `trunk()`.
- `--stat`: Include diff file statistics for each commit.
- `-n <limit>` / `--limit <limit>`: Limit the number of displayed commits.

---

### 4. Branch Health Report (`status`)

Check whether your current branch is ahead, behind, or stale relative to its recorded base:

```bash
$ branch-buddy status
🌿 Branch:  feature/TKT-123-improve-search-ux
🌱 Base:    development (last synced ~3 days ago)
📦 Commits: 3 ahead of base

⚠  Base has moved: development is 2 commits ahead of your branch point.
   Run: git rebase development

Files changed vs base: 12 files, +340 −87 lines
```

Pass a branch name to check a specific branch, or `--json` for structured output:

```bash
$ branch-buddy status feature/TKT-123-improve-search-ux --json
{"branch": "feature/TKT-123-improve-search-ux", "base": "development", "ahead": 3, ...}
```

If no base is recorded, Branch Buddy prompts you to run `branch-buddy set-base <base>`.

---

### 5. Managing Base Branch Metadata

Check current base:
```bash
$ branch-buddy get-base
development
```

Override or update base branch metadata:
```bash
$ branch-buddy set-base development
🔗 Set base of feature/TKT-123-improve-search-ux to development
```

Guess base for legacy branches:
```bash
$ branch-buddy guess-base --write
🔮 Guessed base: development
💾 Saved base development for branch old-feature
```

---

### 6. Health Checks & Healing (`doctor`)

Check repository for broken or deleted parent branches:

```bash
$ branch-buddy doctor
⚠️ Branch 'feature/b' points to missing base 'feature/a'

Found 1 broken link(s). Run `branch-buddy doctor --fix` to auto-heal them.
```

Run `branch-buddy doctor --fix` to automatically heal broken parent links using merge-base and ancestry distance heuristics.

---

### 7. Repository Configuration (`.branchbuddy.toml`)

Create a project-level configuration file with `branch-buddy init`:

```bash
$ branch-buddy init
✨ Created configuration file at .branchbuddy.toml
```

Use `--global` to scaffold `~/.config/branchbuddy/config.toml` instead, and `--force` to overwrite an existing file. Add `--interactive` to answer a short wizard.

Example `.branchbuddy.toml`:

```toml
[naming]
pattern = "{type}/{ticket}-{slug}"
max_length = 63
prefix_separator = "/"
ticket_separator = "-"

[defaults]
base = "main"

[tree]
no_legend = false
```

Configuration resolves in this cascade: CLI flags > `--config <path>` > `.branchbuddy.toml` > `~/.config/branchbuddy/config.toml` > built-in defaults.

---

### 8. Shell Completions

Generate completion scripts for your shell from the built-in hidden subcommand:

```bash
branch-buddy completions bash > /path/to/completions/branch-buddy.bash
branch-buddy completions zsh > /path/to/completions/_branch-buddy
branch-buddy completions fish > /path/to/completions/branch-buddy.fish
```

Or use the just recipes to generate all five shells at once:

```bash
just completions
```

Zsh users can install locally with:

```bash
just install-completions
```

---

## 📄 License

MIT

