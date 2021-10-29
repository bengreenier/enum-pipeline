# You'll need just to get started: `cargo install just`
# just manual: https://github.com/casey/just/#readme

_default:
    @just --list


# Installs tools needed for other `just` recipes
install-tools:
    cargo install cargo-hack cargo-bump cargo-workspaces


# Checks (using `clippy`) all crates across all features
check:
    cargo hack --feature-powerset --exclude-no-default-features clippy --locked -- -D warnings

# Tests all crates across all features
test:
    cargo hack --feature-powerset --exclude-no-default-features test --locked

# Sets the version of the crate to `version`
set-version version:
    cargo bump 0.1.0


# Attempts to publish the crate using `cargo workspaces`
publish:
    cargo workspaces publish --from-git -y --token $CARGO_REGISTRY_TOKEN
