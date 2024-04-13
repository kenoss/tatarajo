check:
  cargo build && cargo clippy && cargo fmt -- --check

check-strict:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; just check

run *ARGS:
  cargo run {{ARGS}}

test *ARGS:
  cargo test {{ARGS}}
