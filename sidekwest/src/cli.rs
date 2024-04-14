use std::io::{stdin, Read as _};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;

use crate::database::{db_connect, test_database, DEFAULT_DB};
use crate::secrecy::{decrypt, encrypt, generate_key, url_to_webhook};
use crate::{bot, schedule};

#[derive(Debug, Parser, Clone)]
struct CLI {
    #[arg(short = 'd', long)]
    /// The database URL to operate against
    database_url: Option<String>,

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
    /// Test the database and schema
    DbTest,
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

pub async fn run() -> Result<()> {
    let opts: CLI = CLI::parse();
    let db = db_connect(opts.database_url.as_deref().unwrap_or(DEFAULT_DB)).await?;
    match opts.command {
        Command::Bot => bot::bot_main(db).await,
        Command::Schedule { file } => schedule::run_schedule_update(file).await?,
        Command::Encrypt => {
            let str = encrypt(&read_stdin()?)?;
            print!("{str}");
        }
        Command::Decrypt => {
            let str = decrypt(&read_stdin()?)?;
            print!("{str}");
        }
        Command::TestFernet => {
            let str = read_stdin()?;
            let actual = encrypt(&str).and_then(|s| decrypt(&s))?;
            assert_eq!(str, actual);
            println!("OK");
        }
        Command::EncryptWebhook => {
            let hook = url_to_webhook(&read_stdin()?)?;
            println!("{}", serde_json::to_string_pretty(&hook)?);
        }
        Command::NewKey => {
            print!("{}", generate_key());
        }
        Command::DbTest => {
            test_database().await?;
        }
    };
    Ok(())
}
