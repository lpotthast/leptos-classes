# Run `cargo install just`. Then run `just` to list available recipes.

default:
  just --list

# Perform the one-time local setup required for this crate.
once:
  just enable-wasm
  just install-tools

# Install the tools this crate depends on for local development.
install-tools:
  cargo install just
  cargo install cargo-leptos
  cargo install wasm-bindgen-cli

# Enable the WASM target required by `cargo leptos` and wasm tests.
enable-wasm:
  rustup target add wasm32-unknown-unknown

# Format the crate.
fmt:
  cargo fmt --all

# Run clippy for the crate test surface.
clippy:
  cargo clippy --tests -- -D warnings

# Type-check the crate.
check:
  cargo check

# Run unit tests, integration tests, and doctests for the native target.
test:
  cargo test

# Run only the crate unit tests and doctests.
test-lib:
  cargo test --lib

# Run the wasm-target test command expected to work from this crate directory.
test-wasm:
  cargo test --target wasm32-unknown-unknown

# Run the Chrome-based browser integration test headlessly.
browser-test:
  cargo test --test browser_test -- --nocapture

# Run the Chrome-based browser integration test with a visible browser.
browser-test-visible:
  BROWSER_TEST_VISIBLE=1 cargo test --test browser_test -- --nocapture

# Serve the crate-local test app for manual inspection.
serve-test-app:
  cd ./testing/test-app && cargo leptos serve

# Clean build artifacts for this crate and its local test app.
clean:
  cargo clean
  cargo clean --manifest-path ./testing/test-app/Cargo.toml

# Run the most important verification commands for this crate.
verify:
  cargo test --lib
  cargo test --target wasm32-unknown-unknown
  cargo test --test browser_test -- --nocapture
