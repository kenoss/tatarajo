#[macro_use]
extern crate maplit;

use anyhow::Result;
use big_s::S;
use sabiniwm::action::{self, Action, ActionFnI};
use sabiniwm::input::{KeySeqSerde, Keymap, ModMask};
use sabiniwm::view::predefined::LayoutMessageSelect;
use sabiniwm::view::stackset::WorkspaceTag;
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

        kbd("H-t") => action::ActionMoveFocus::Next.into_action(),
        kbd("H-h") => action::ActionMoveFocus::Prev.into_action(),
        kbd("H-T") => action::ActionWindowSwap::Next.into_action(),
        kbd("H-H") => action::ActionWindowSwap::Prev.into_action(),
        kbd("H-v") => action::ActionWorkspaceFocus::Next.into_action(),
        kbd("H-b") => action::ActionWorkspaceFocus::Prev.into_action(),

        kbd("H-k") => (action::ActionWindowKill {}).into_action(),
    });

    let workspace_tags = (0..=9).map(|i| WorkspaceTag(format!("{}", i))).collect();
    Sabiniwm::start(workspace_tags, keymap)?;

    Ok(())
}
