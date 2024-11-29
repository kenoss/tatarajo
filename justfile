check:
  cargo build && cargo clippy && cargo fmt -- --check

check-strict:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; just check

check-warn:
  export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; clear; cargo build --color always |& head -n 32

run *ARGS:
  cargo run {{ARGS}}

test *ARGS:
  cargo test {{ARGS}}

export TEMPLATE_SESSION := '''
[Desktop Entry]
Name=tatarajo
Comment=A tiling wayland compositor, influenced xmonad
Exec=EXEC
Type=Application
'''

export TEMPLATE_LAUNCH := '''
#!/usr/bin/env bash

RUST_LOG=info RUST_BACKTRACE=1 TATARAJO_XKB_CONFIG='{"layout": "custom", "repeat_delay": 200, "repeat_rate": 60}' BIN_PATH
'''

install-session-dev:
  cargo build --release
  mkdir -p target/session
  echo "$TEMPLATE_SESSION" | sed "s|EXEC|$(pwd)/target/session/launch|" > target/session/tatarajo.desktop
  echo "$TEMPLATE_LAUNCH" | sed "s|BIN_PATH|$(pwd)/target/release/tatarajo-pistachio|" > target/session/launch
  chmod +x $(pwd)/target/session/launch
  sudo install -m 644 target/session/tatarajo.desktop /usr/share/wayland-sessions/
