#![warn(rust_2018_idioms)]
// If no backend is enabled, a large portion of the codebase is unused.
// So silence this useless warning for the CI.
#![cfg_attr(
    not(any(feature = "winit", feature = "udev")),
    allow(dead_code, unused_imports)
)]

#[allow(unused_imports)]
#[macro_use]
extern crate tracing;

pub mod action;
pub mod backend;
pub mod cursor;
mod envvar;
mod external_trait_def;
pub mod focus;
pub mod input;
pub(crate) mod input_event;
pub mod input_handler;
pub(crate) mod model;
pub mod pointer;
pub mod render;
pub mod shell;
pub mod state;
pub mod state_delegate;
#[allow(unused)]
pub(crate) mod util;
pub mod view;

pub use state::{ClientState, SabiniwmState};
