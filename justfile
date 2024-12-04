test-comrade-core:
 RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo test --manifest-path=crates/core/Cargo.toml

# Uses wasm feature for wasm32-unknown-unknown target, but not for wasm32-wasi target
test-core-wasm32-build:
  RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo build --target wasm32-unknown-unknown --manifest-path=crates/core/Cargo.toml --features wasm
  # wasi 
  RUST_LOG=trace RUSTFLAGS="--allow dead_code" cargo build --target wasm32-wasip1 --manifest-path=crates/core/Cargo.toml

test: test-comrade-core test-core-wasm32-build

# Generate the web wasm bindings for ./crates/multiwallet-bindings using wasm-pack 
generate-multiwallet-bindings:
  wasm-pack build ./crates/multiwallet-bindings --target web 

# Generate the web wasm bindings for ./crates/core-bindings using wasm-pack 
generate-bindings:
  wasm-pack build ./crates/core-bindings --target web

# for each dir in crates which has a `wit` directory in it, AND has src/bindings.rs, build it
build-wits:
 for dir in crates/*; do \
    if ([ -d $dir/wit ] && [ -f $dir/src/bindings.rs ]); then \
     cargo component build --manifest-path=$dir/Cargo.toml; \
     cargo component build --manifest-path=$dir/Cargo.toml --release; \
   fi \
 done

# Runs the Svelte Demo
run-demo: generate-multiwallet-bindings generate-bindings build-wits
  cd demo && npm run dev -- --open

# Build for production 
build: generate-multiwallet-bindings generate-bindings build-wits
  cd demo && npm run build
