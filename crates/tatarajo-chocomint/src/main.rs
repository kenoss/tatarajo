// This is a QWERTY version of tatarajo-pistachio.
// This is loosely updated. Last update is 2024-11-29.

#[allow(unused_imports)]
#[macro_use]
extern crate maplit;

use big_s::S;
use itertools::Itertools;
use tatarajo::action::{self, Action, ActionFnI};
use tatarajo::input::{KeySeqSerde, Keymap, ModMask};
use tatarajo::view::stackset::WorkspaceTag;
use tatarajo::TatarajoState;

fn should_use_udev() -> bool {
    matches!(
        std::env::var("DISPLAY"),
        Err(std::env::VarError::NotPresent)
    ) && matches!(
        std::env::var("WAYLAND_DISPLAY"),
        Err(std::env::VarError::NotPresent)
    )
}

fn tracing_init() -> eyre::Result<()> {
    use time::macros::format_description;
    use time::UtcOffset;
    use tracing_subscriber::fmt::time::OffsetTime;
    use tracing_subscriber::EnvFilter;

    match std::env::var("RUST_LOG") {
        Err(std::env::VarError::NotPresent) => {}
        _ => {
            let offset = UtcOffset::current_local_offset().unwrap();
            let timer = OffsetTime::new(
                offset,
                format_description!("[hour]:[minute]:[second].[subsecond digits:3]"),
            );

            let fmt = tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .with_timer(timer)
                .with_line_number(true)
                .with_ansi(true);

            if should_use_udev() {
                let log_file =
                    std::io::LineWriter::new(std::fs::File::create("/tmp/tatarajo.log")?);

                fmt.with_writer(std::sync::Mutex::new(log_file)).init();
            } else {
                fmt.init();
            }
        }
    }

    Ok(())
}

fn main() -> eyre::Result<()> {
    tracing_init()?;
    color_eyre::install()?;

    let workspace_tags = (0..=9)
        .map(|i| WorkspaceTag(format!("{}", i)))
        .collect_vec();

    let meta_keys = if should_use_udev() {
        hashmap! {
            S("C") => ModMask::CONTROL,
            S("M") => ModMask::MOD1,
            S("s") => ModMask::MOD4,
            S("H") => ModMask::MOD5,
        }
    } else {
        hashmap! {
            S("C") => ModMask::CONTROL,
            S("M") => ModMask::MOD1,
            // Hyper uses Mod5 in my environment. Use Mod4 for development with winit.
            S("H") => ModMask::MOD4,
        }
    };
    let keyseq_serde = KeySeqSerde::new(meta_keys);
    let kbd = |s| keyseq_serde.kbd(s).unwrap();
    let mut keymap = hashmap! {
        kbd("H-b H-q") => action::ActionQuitTatarajo.into_action(),
        kbd("H-b H-2") => action::ActionChangeVt(2).into_action(),

        kbd("H-b H-t") => Action::spawn("alacritty"),
        kbd("H-b H-e") => Action::spawn("emacs"),
        kbd("H-b H-b") => Action::spawn("firefox"),

        kbd("H-h") => action::ActionWorkspaceFocusNonEmpty::Prev.into_action(),
        kbd("H-k") => action::ActionMoveFocus::Prev.into_action(),
        kbd("H-j") => action::ActionMoveFocus::Next.into_action(),
        kbd("H-l") => action::ActionWorkspaceFocusNonEmpty::Next.into_action(),
        kbd("H-H") => action::ActionWindowMoveToWorkspace::Prev.into_action(),
        kbd("H-K") => action::ActionWindowSwap::Prev.into_action(),
        kbd("H-J") => action::ActionWindowSwap::Next.into_action(),
        kbd("H-L") => action::ActionWindowMoveToWorkspace::Next.into_action(),
        kbd("H-s") => action::ActionWorkspaceFocusNonEmpty::Prev.into_action(),
        kbd("H-d") => action::ActionMoveFocus::Prev.into_action(),
        kbd("H-f") => action::ActionMoveFocus::Next.into_action(),
        kbd("H-g") => action::ActionWorkspaceFocusNonEmpty::Next.into_action(),
        kbd("H-S") => action::ActionWindowMoveToWorkspace::Prev.into_action(),
        kbd("H-D") => action::ActionWindowSwap::Prev.into_action(),
        kbd("H-F") => action::ActionWindowSwap::Next.into_action(),
        kbd("H-G") => action::ActionWindowMoveToWorkspace::Next.into_action(),

        kbd("H-greater") => action::ActionWorkspaceFocus::Next.into_action(),
        kbd("H-n") => action::ActionWorkspaceFocus::Prev.into_action(),

        kbd("H-b H-k") => (action::ActionWindowKill {}).into_action(),
    };
    keymap.extend(workspace_tags.iter().cloned().enumerate().map(|(i, tag)| {
        (
            // TODO: Fix lifetime issue and use `kbd`.
            keyseq_serde.kbd(&format!("H-{i}")).unwrap(),
            action::ActionWorkspaceFocus::WithTag(tag).into_action(),
        )
    }));
    const SHIFTED: &[char] = &[')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
    fn keysym_str(c: char) -> &'static str {
        match c {
            '!' => "exclam",
            '@' => "at",
            '#' => "numbersign",
            '$' => "dollar",
            '%' => "percent",
            '^' => "asciicircum",
            '&' => "ampersand",
            '*' => "asterisk",
            '(' => "parenleft",
            ')' => "parenright",
            _ => unreachable!(),
        }
    }
    keymap.extend(workspace_tags.iter().cloned().enumerate().map(|(i, tag)| {
        (
            // TODO: Fix lifetime issue and use `kbd`.
            keyseq_serde
                .kbd(&format!("H-{}", keysym_str(SHIFTED[i])))
                .unwrap(),
            action::ActionWithSavedFocus(
                action::ActionWindowMoveToWorkspace::WithTag(tag).into_action(),
            )
            .into_action(),
        )
    }));
    let keymap = Keymap::new(keymap);

    TatarajoState::run(workspace_tags, keymap)?;

    Ok(())
}
