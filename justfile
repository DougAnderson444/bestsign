test-comrade-core:
 RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo test --manifest-path=crates/bestsign-core/Cargo.toml

# Uses wasm feature for wasm32-unknown-unknown target, but not for wasm32-wasi target
test-core-wasm32-build:
  RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo build --target wasm32-unknown-unknown --manifest-path=crates/core/Cargo.toml --features wasm
  # wasi 
  RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo build --target wasm32-wasi --manifest-path=crates/core/Cargo.toml
