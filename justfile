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
