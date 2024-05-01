#[macro_use]
extern crate maplit;

use anyhow::Result;
use big_s::S;
use sabiniwm::action::{Action, ActionFnI, ActionMoveFocus};
use sabiniwm::input::{KeySeqSerde, Keymap, ModMask};
use sabiniwm::view::predefined::LayoutMessageSelect;
use sabiniwm::Sabiniwm;

fn tracing_init() -> Result<()> {
    use time::macros::format_description;
    use time::UtcOffset;
    use tracing_subscriber::fmt::time::OffsetTime;
    use tracing_subscriber::EnvFilter;

    match std::env::var("RUST_LOG") {
        Err(std::env::VarError::NotPresent) => {}
        _ => {
            let offset = UtcOffset::current_local_offset().expect("should get local offset!");
            let timer = OffsetTime::new(
                offset,
                format_description!("[hour]:[minute]:[second].[subsecond digits:3]"),
            );
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .with_timer(timer)
                .with_line_number(true)
                .with_ansi(true)
                .init();
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_init()?;

    let keyseq_serde = KeySeqSerde::new(hashmap! {
        S("C") => ModMask::CONTROL,
        S("M") => ModMask::MOD1,
        // S("s") => ModMask::MOD4,
        // S("H") => ModMask::MOD5,
        // Hyper uses Mod5 in my environment. Use Mod4 for development with winit.
        S("H") => ModMask::MOD4,
    });
    let kbd = |s| keyseq_serde.kbd(s).unwrap();
    let keymap = Keymap::new(hashmap! {
        kbd("H-x H-t") => Action::spawn("alacritty"),
        kbd("H-space") => LayoutMessageSelect::Next.into(),
        kbd("H-t") => ActionMoveFocus::Next.into_action(),
        kbd("H-h") => ActionMoveFocus::Prev.into_action(),
    });

    Sabiniwm::start(keymap)?;

    Ok(())
}
