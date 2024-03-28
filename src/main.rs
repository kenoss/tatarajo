use anyhow::Result;
use sabiniwm::Sabiniwm;

fn main() -> Result<()> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    Sabiniwm::start()?;

    Ok(())
}
