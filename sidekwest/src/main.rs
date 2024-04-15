use color_eyre::Result;
use tracing_subscriber::EnvFilter;

mod bot;
mod cli;
mod discord;
mod schedule;
mod secrecy;

fn load_dotenv() {
    let path = shellexpand::tilde("~/.config/sidekwest/dotenv");
    dotenv::from_path(path.as_ref()).expect("failed to load dotenv file");
}

fn setup() {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install().expect("Failed to install eyre reporter");

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,sqlx=warn")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .expect("Failed to set tracing subscriber");
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    setup();
    cli::run().await?;
    Ok(())
}
