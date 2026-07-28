# Branch Buddy 🌳🤝

**Branch Buddy** is a lightweight Git & Jujutsu (`jj`) companion CLI that brings persistent parent/base branch metadata to your local version control workflows, while making branch/bookmark creation from human-readable titles completely effortless.

Git and Jujutsu don't naturally record explicit parent/base branch relationships across stacked features. Branch Buddy fixes this by storing persistent `branch.<name>.base` records in local `.git/config` at the moment of branch/bookmark creation, giving you full lineage tracking, ancestry tree visualization, and focused commit logs across both Git and Jujutsu repositories.

---

## 🌟 Key Features

- 🧠 **Persistent Base Metadata**: Never forget where a branch originated. Branch Buddy explicitly tracks parent branch relationships in `.git/config`.
- 🦎 **Dual Engine Support (Git + Jujutsu `jj`)**: Seamlessly auto-detects Jujutsu (`.jj`) or Git environments. In Jujutsu mode, it creates changes, attaches bookmarks, formats Change IDs (`[change: <id>]`), and dynamically resolves `trunk()`.
- 🗣️ **Human-Friendly Naming**: Just type `"Fix user signup flow"` and Branch Buddy generates clean, unique branch slugs (`fix-user-signup-flow` or `feature/AUTH-101-fix-user-signup-flow`).
- 🌳 **Ancestry Trees (`branch-buddy tree`)**: Visualize stacked branches and bookmarks with clear parent-child lineages, feature icons (🌿), and dynamic trunk indicators (🪵).
- 🪵 **Focused Stack Logs (`branch-buddy log`)**: View noise-free commit history between your branch and its base (`base..@`), or pass `--stack` to view changes across all parent branches up to `trunk()`.
- 🔮 **Legacy Guessing (`branch-buddy guess-base`)**: Automatically estimate bases for legacy branches created before installing Branch Buddy.
- 🩺 **Health Checks (`branch-buddy doctor`)**: Diagnose and heal broken or orphaned base-branch chains when parent branches are deleted.

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

### 4. Managing Base Branch Metadata

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

### 5. Health Checks & Healing (`doctor`)

Check repository for broken or deleted parent branches:

```bash
$ branch-buddy doctor
⚠️ Branch 'feature/b' points to missing base 'feature/a'

Found 1 broken link(s). Run `branch-buddy doctor --fix` to auto-heal them.
```

Run `branch-buddy doctor --fix` to automatically heal broken parent links using merge-base and ancestry distance heuristics.

---

## 📄 License

MIT

