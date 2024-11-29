# tatarajo

A tiling Wayland compositor for education

## Goal and non goal

Goal

- Provides an 1-day educational course of Wayland compositor with powerful [`smithay`](https://github.com/Smithay/smithay)
  crate.
- Provides a partially refactored and minimized version of [anvil](https://github.com/Smithay/smithay/tree/master/anvil), which
  is the shared infra of [sabiniwm](https://github.com/kenoss/sabiniwm) for who wants to start implementing yet another tiling
  Wayland compositor.
- Provides a content for [Rust.Tokyo 2024](https://rust.tokyo/).

Non goal

- Frequent update (The author is planning to update it per year.)
- Provides detailed knowledge of `smithay`.

## How to run and develop

See [justfile](./justfile). For example,

```shell
$ # build
$ cargo build
$ # run
$ cargo run
$ # check
$ just check-strict
$ # watch
$ cargo watch -c -s 'cargo build && just check-strict'
$ # install to /usr/share/wayland-sessions/
$ just install-session-dev
```

## udev

### Dependencies

See [.github/workflows/ci.yaml](.github/workflows/ci.yaml).

### Run

You can run it with udev backend in the following ways:

- From TTY (i.e., turning off display manager): Just `cargo run` works.
- From display manager: Use `just install-session-dev` and select `tatarajo`.

Note that you need to set an environment variable `TATARAJO_XKB_CONFIG`.

## Cource

You can implement the following topics:

- [x] Implement a "Tall" layout ([xmonad Tall](https://hackage.haskell.org/package/xmonad-0.18.0/docs/XMonad-Layout.html#t:Tall)).
- [x] Put margins for windows.
- [x] Decorate windows with borders.
- [ ] ...and features you want.

This repository provides an answer. You can see
[v0-begin](https://github.com/kenoss/tatarajo/tree/v0-begin)..[v0-end](https://github.com/kenoss/tatarajo/tree/v0-end).

雑メモ

- Tall と margin で diff が小さいのはちょっとズルです. 本当は `crate::model::grid_geometry` や `crate::view::window::Thickness`
  はこのへんのコミットに含めるべきです. (消すのが面倒だったのでそのままにしまった.)
- `smithay` の解説としてはもうちょっと `crate::view` 配下を削って `smithay` のものを直接触るべきです.
  が, 準備時間と発表時間と差分の見せ方の関係でこうなりました.
  (まぁ StackSet まわりは xmonad 由来だし, 下層との接続を見るのもそれはそれで面白いしええかとなった.
  どうせ本格的に自作すると色々見るわけだし.)

## Contributing

- If you are updating infra part, consider to contribute to [sabiniwm](https://github.com/kenoss/sabiniwm).
- If you are adding a topic for education, please discuss it in this repo.

Again, frequent update (of this repo) is not a goal.

## License

This repository is distributed under the terms of both the MIT license and the
Apache License (Version 2.0), with portions covered by various BSD-like
licenses.

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
