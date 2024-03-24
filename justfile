export RUSTFLAGS := '-D warnings'

check:
  cargo build && cargo clippy && cargo fmt -- --check

watch-check:
  cargo watch -c -s 'just check'

run *ARGS:
  cargo run {{ARGS}}
