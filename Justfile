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
  cargo install cargo-audit
  cargo install cargo-deny
  cargo install cargo-semver-checks
  cargo install leptosfmt

# Enable the WASM target required by `cargo leptos` and the wasm compile check.
enable-wasm:
  rustup target add wasm32-unknown-unknown

# Format the crate.
fmt:
  cargo fmt --all

# Run format checks.
fmt-check:
  cargo fmt --all --check

# Format Leptos code.
leptosfmt:
  leptosfmt ./testing/test-app/*

# Run clippy.
clippy:
  cargo clippy --all-targets -- -D warnings

# Type-check the crate.
check:
  cargo check --all-targets

# Verify the library compiles for the wasm32-unknown-unknown target.
check-wasm:
  cargo check --target wasm32-unknown-unknown --locked

# Run unit tests, integration tests, and doctests for the native target.
test:
  cargo test

# Run only the crate unit tests.
test-lib:
  cargo test --lib

# Run only the crate doc tests.
test-doc:
  cargo test --doc

# Build documentation with rustdoc warnings denied (matches CI).
doc:
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked

# Serve the crate-local test app for manual inspection.
serve-test-app:
  cd ./testing/test-app && cargo leptos serve

# Run the Chrome-based browser integration test headlessly.
browser-test:
  cargo test --test browser_test -- --nocapture

# Run the Chrome-based browser integration test with a visible browser.
browser-test-visible:
  BROWSER_TEST_VISIBLE=1 cargo test --test browser_test -- --nocapture

# Start the test app and pause before running browser assertions.
browser-test-pause:
  BROWSER_TEST_VISIBLE=1 BROWSER_TEST_PAUSE=1 cargo test --test browser_test -- --nocapture

# Scan Cargo.lock against the RustSec advisory database.
audit:
  cargo audit

# Run cargo-deny's supply-chain checks (advisories, bans, sources).
deny:
  cargo deny check

# Detect breaking public-API changes vs. the latest published release.
semver-check:
  cargo semver-checks

# Clean build artifacts for this crate and its local test app.
clean:
  cargo clean
  cargo clean --manifest-path ./testing/test-app/Cargo.toml

# Run the most important verification commands for this crate.
verify:
  just fmt-check
  just check
  just check-wasm
  just clippy
  just test-lib
  just test-doc
  just browser-test
  just doc

# Verify the Leptos nightly integration on native and WASM targets.
verify-nightly:
  cargo +nightly check --all-targets --features nightly --locked
  cargo +nightly check --target wasm32-unknown-unknown --features nightly --locked
  cargo +nightly test --lib --features nightly --locked
  cargo +nightly test --doc --features nightly --locked
