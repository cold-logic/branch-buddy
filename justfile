set shell := ["bash", "-uc"]

# Show available recipes
default:
    @just --list

# Build the project (debug)
build:
    cargo build

# Build the project for release
release:
    cargo build --release

# Run tests
test:
    cargo test

# Format the code (requires rustfmt)
fmt:
    cargo fmt

# Run clippy for linting
lint:
    cargo clippy -- -D warnings

# Install branch-buddy locally using cargo
install:
    cargo install --path .

# Run branch-buddy (pass arguments after --, e.g. just run -- new "hello")
run +ARGS="":
    cargo run -- {{ARGS}}

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
    @echo "     fpath=(~/.zfunc \$fpath)"
    @echo "     autoload -Uz compinit && compinit"
