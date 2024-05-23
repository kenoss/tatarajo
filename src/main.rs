use anyhow::Result;

static POSSIBLE_BACKENDS: &[&str] = &[
    #[cfg(feature = "winit")]
    "--winit : Run anvil as a X11 or Wayland client using winit.",
    #[cfg(feature = "udev")]
    "--tty-udev : Run anvil as a tty udev client (requires root if without logind).",
];

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

    let arg = ::std::env::args().nth(1);
    match arg.as_ref().map(|s| &s[..]) {
        #[cfg(feature = "winit")]
        Some("--winit") => {
            tracing::info!("Starting anvil with winit backend");
            sabiniwm::winit::run_winit();
        }
        #[cfg(feature = "udev")]
        Some("--tty-udev") => {
            tracing::info!("Starting anvil on a tty using udev");
            sabiniwm::udev::run_udev();
        }
        Some(other) => {
            tracing::error!("Unknown backend: {}", other);
        }
        None => {
            println!("USAGE: sabiniwm --<backend>");
            println!();
            println!("Possible backends are:");
            for b in POSSIBLE_BACKENDS {
                println!("\t{}", b);
            }
        }
    }

    Ok(())
}
