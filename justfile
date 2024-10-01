test-comrade-core:
 RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo test --manifest-path=crates/core/Cargo.toml

# Uses wasm feature for wasm32-unknown-unknown target, but not for wasm32-wasi target
test-core-wasm32-build:
  RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo build --target wasm32-unknown-unknown --manifest-path=crates/core/Cargo.toml --features wasm
  # wasi 
  RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo build --target wasm32-wasi --manifest-path=crates/core/Cargo.toml

# Generate the web wasm bindings for ./crates/multiwallet-bindings using wasm-pack 
generate-multiwallet-bindings:
  wasm-pack build ./crates/multiwallet-bindings --target web 

# Generate the web wasm bindings for ./crates/bindings using wasm-pack 
generate-bindings:
  wasm-pack build ./crates/bindings --target web

# Runs the Svelte Demo
run-demo: generate-multiwallet-bindings generate-bindings
  cd demo && npm run dev -- --open

# Build for production 
build: generate-multiwallet-bindings generate-bindings
  cd demo && npm run build
