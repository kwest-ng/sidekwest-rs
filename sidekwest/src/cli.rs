use std::io::{stdin, Read as _};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;

use crate::secrecy;
use crate::{bot, discord, schedule};

#[derive(Debug, Parser, Clone)]
struct CliOpts {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Clone)]
enum Command {
    /// Run the discord bot
    Bot,
    /// Run the schedule update process
    Schedule { file: PathBuf },
    /// Encrypt a single string from stdin
    Encrypt,
    /// Decrypt a single string from stdin
    Decrypt,
    /// Test round-trip encryption with the current SECRET_KEY
    TestFernet,
    /// From a webhook URL, produce a JSON object for the ID and encrypted token
    EncryptWebhook,
    /// Create a new Fernet key
    NewKey,
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

pub async fn run() -> Result<()> {
    let opts: CliOpts = CliOpts::parse();
    match opts.command {
        Command::Bot => bot::bot_main().await,
        Command::Schedule { file } => schedule::run_schedule_update(file).await?,
        Command::Encrypt => {
            let str = secrecy::encrypt(&read_stdin()?)?;
            print!("{str}");
        }
        Command::Decrypt => {
            let str = secrecy::decrypt(&read_stdin()?)?;
            print!("{str}");
        }
        Command::TestFernet => {
            let str = read_stdin()?;
            let actual = secrecy::encrypt(&str).and_then(|s| secrecy::decrypt(&s))?;
            assert_eq!(str, actual);
            println!("OK");
        }
        Command::EncryptWebhook => {
            let hook = discord::api::url_to_webhook(&read_stdin()?)?;
            println!("{}", serde_json::to_string_pretty(&hook)?);
        }
        Command::NewKey => {
            print!("{}", secrecy::generate_key());
        }
    };
    Ok(())
}
