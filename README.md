# Branch Buddy 🌳🤝

**Branch Buddy** is a lightweight Git companion CLI that brings first-class parent/base branch metadata to your local Git workflows, while also making branch creation from human-readable titles completely effortless.

Git does not naturally remember which branch another branch was created from—it only tracks commit history. Branch Buddy fixes this by persistently storing a `branch.<name>.base` record in your local `.git/config` at the moment of branch creation.

## Features

- 🧠 **Persistent Base Metadata**: Never forget where a branch originated. Branch Buddy explicitly tracks parent branch relationships natively in `.git/config`.
- 🗣️ **Human-Friendly Naming**: Just type `"Fix user signup flow"` and let Branch Buddy generate the safe slug `fix-user-signup-flow`.
- 🎨 **Beautiful & Interactive UX**: Enjoy colorful outputs, fun emojis, and interactive fuzzy-select dropdowns when things get ambiguous.
- 🌲 **Ancestry Trees**: Visualize your stacked branches instantly with `branch-buddy tree`.
- 🔮 **Legacy Guessing**: Use built-in heuristics (`merge-base`) to automatically retroactively guess bases for branches created before you installed Branch Buddy.
- 🪝 **Hook Friendly**: Designed to fit seamlessly into Git `post-checkout` hooks for completely transparent operation.

## Installation

### Default Installation

You can install the latest version of Branch Buddy directly from GitHub using Cargo:

```bash
cargo install --git https://github.com/cold-logic/branch-buddy
```

### Local Development

If you want to install, build, or compile your own variant, you can clone the repository and install it locally:

```bash
cargo install --path .
```

## Quick Start (Git Aliases)

Branch Buddy is designed to feel like a native extension of Git. To get the best experience, add these aliases to your global `~/.gitconfig`:

```ini
[alias]
  # Create a branch from human-readable title (base = current branch)
  bb = "!branch-buddy new"

  # Conventional feature branch off main
  cobb = "!f() { branch-buddy new \"$1\" --base main --type feature; }; f"

  # Base operations
  base-branch = "!branch-buddy get-base"
  set-base = "!branch-buddy set-base"
  tree = "!branch-buddy tree"
```

## Usage

### Creating Branches

Use the `new` command to create branches from human-readable sentences. It will automatically slugify the title, make sure it is unique, check out the branch, and set the base branch.

```bash
$ branch-buddy new "Improve search UX" --type feature --ticket TKT-123
✨ Created branch: feature/TKT-123-improve-search-ux
🌱 Base: main
```

**Options:**
- `--base <branch>`: The base branch (defaults to the branch you are currently on).
- `--type <type>`: Add a prefix (e.g., `feature`, `bugfix`).
- `--ticket <id>`: Add an issue tracker ID.
- `--dry-run`: See the slugified name without actually creating the branch.
- `--no-checkout`: Create the branch but stay on your current branch.

### Detached HEAD & Jujutsu Support

If you run `branch-buddy new` while in a **detached HEAD** state (e.g., when checking out a specific commit or when using tools like [Jujutsu `jj`](https://github.com/martinvonz/jj)), Branch Buddy won't just fail! 

Instead, it dynamically queries all your local branches, ranks them by distance to your current commit, and opens a sleek interactive fuzzy-select menu (with the closest match pre-selected) so you can effortlessly select the correct base.

```bash
$ branch-buddy new 'my-cool-idea'
⚠️ Currently in detached HEAD. Resolving base branch...
? Select base branch for metadata (closest match pre-selected) ›
❯ main
  feature/shopping-cart
  develop
```

### Viewing the Branch Hierarchy Tree

Working with stacked branches? View the lineage of your current branch back to `main`:

```bash
$ branch-buddy tree
feature/TKT-123-improve-search-ux
└── dev
    └── main
```

### Getting and Setting Base Branches

Check the base of the current branch:
```bash
$ branch-buddy get-base
main
```

Explicitly override or set a base branch metadata:
```bash
$ branch-buddy set-base dev
```

### Guessing Legacy Branches

If you have older branches from before you started using Branch Buddy, you can use the `guess-base` command. It uses Git's commit history and `merge-base` distance to find the closest match among candidates:

```bash
$ branch-buddy guess-base --candidates main,master,develop --write
🔮 Guessed base: main
💾 Saved base main for branch old-feature
```

### Health Checks & Healing (Doctor)

If you manually delete an ancestor branch, the "chain" of base branches breaks. You can triage and fix orphaned branches using the `doctor` (or `fsck`) command:

```bash
$ branch-buddy doctor
⚠️  Branch 'feature/b' points to missing base 'feature/a'

Found 1 broken link(s). Run `branch-buddy doctor --fix` to auto-heal them.
```

When you pass the `--fix` flag, Branch Buddy will automatically re-run the `merge-base` heuristics to map orphaned branches to their closest living ancestors!

## Git Hook Integration

You can automate tracking by adding a `.git/hooks/post-checkout` file. This ensures that even if you create a branch using a standard `git checkout -b` or via your IDE, Branch Buddy will attempt to annotate it the first time you switch to it.

You can install this hook automatically by running:

```bash
$ branch-buddy install
```

If you *also* want the hook to perform an automatic health check (warning you if the current branch's base has been deleted), you can opt into that feature using:

```bash
$ branch-buddy doctor --install-hook
```

This will safely install the following `post-checkout` hook:

```bash
#!/bin/bash
# post-checkout

# Flag 1 means a branch checkout (not a file checkout)
if [ "$3" != "1" ]; then exit 0; fi

# Get current branch
curr=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
if [ "$curr" == "HEAD" ]; then exit 0; fi

# If the current branch has a base, but that base branch was deleted, warn the user
base=$(branch-buddy get-base "$curr" 2>/dev/null)
if [ -n "$base" ] && ! git show-ref --verify --quiet "refs/heads/$base"; then
    echo "⚠️  Base branch '$base' is missing! Run 'branch-buddy doctor --fix' to heal it."
fi

# If base is already set, do nothing
if branch-buddy has-base "$curr" >/dev/null 2>&1; then
    exit 0
fi

# Try to determine the previous branch name
prev_branch=$(git rev-parse --abbrev-ref @{-1} 2>/dev/null)

if [ -n "$prev_branch" ] && [ "$prev_branch" != "HEAD" ]; then
    branch-buddy set-base "$prev_branch" "$curr" >/dev/null 2>&1
fi
```

## License
MIT
