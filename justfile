check:
  cargo build && cargo clippy && cargo fmt -- --check

check-strict:
  export RUSTFLAGS='-D warnings'; cargo build && cargo clippy && cargo fmt -- --check

run *ARGS:
  cargo run {{ARGS}}

test *ARGS:
  cargo test {{ARGS}}
