#[macro_use]
extern crate maplit;

use anyhow::Result;
use big_s::S;
use sabiniwm::action::Action;
use sabiniwm::input::{KeySeqSerde, Keymap, ModMask};
use sabiniwm::Sabiniwm;

fn main() -> Result<()> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let keyseq_serde = KeySeqSerde::new(hashmap! {
        S("C") => ModMask::CONTROL,
        S("M") => ModMask::MOD1,
        S("s") => ModMask::MOD4,
        S("H") => ModMask::MOD5,
    });
    let kbd = |s| keyseq_serde.kbd(s).unwrap();
    let keymap = Keymap::new(hashmap! {
        kbd("H-c H-c t") => Action::spawn("alacritty"),
    });

    Sabiniwm::start(keymap)?;

    Ok(())
}
