// sidekwest bot
// sidekwest schedule <FILE>

use std::io::{stdin, Read};
use std::path::PathBuf;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use clap::{Parser, Subcommand};
use fernet::Fernet;
use regex::Regex;

use self::database::{db_connect, test_database};
use self::schedule::Webhook;
use self::secrecy::{decrypt, encrypt};

mod bot;
mod database;
mod schedule;
mod secrecy;

const DEFAULT_DB: &str = "sqlite://testing.sqlite?mode=rwc";

#[derive(Debug, Parser, Clone)]
struct CLI {
    #[arg(short = 'd', long)]
    /// The database URL to operate against
    database_url: Option<String>,

    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Debug, Subcommand, Clone)]
enum SubCommand {
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

fn load_dotenv() {
    let path = shellexpand::tilde("~/.config/sidekwest/dotenv");
    dotenv::from_path(path.as_ref()).expect("failed to load dotenv file");
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

const URL_PATTERN: &str = r"/webhooks/(?<id>.*?)/(?<token>.*?)/?$";

fn url_to_webhook(s: &str) -> Result<Webhook> {
    let re = Regex::new(URL_PATTERN)?;
    let caps = re
        .captures(s)
        .ok_or_else(|| eyre!("Did not match webhook URL"))?;
    let id = caps
        .name("id")
        .ok_or_else(|| eyre!("Could not find webhook id"))?
        .as_str();
    let token = caps
        .name("token")
        .ok_or_else(|| eyre!("Could not find webhook token"))?
        .as_str();
    let hook = Webhook::new(id.parse()?, encrypt(token)?);
    Ok(hook)
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    let opts: CLI = CLI::parse();
    let _db = db_connect(opts.database_url.as_deref().unwrap_or(DEFAULT_DB)).await?;
    match opts.command {
        SubCommand::Bot => bot::bot_main().await,
        SubCommand::Schedule { file } => {

                schedule::run_schedule_update(file).await?
        },
        SubCommand::Encrypt => {
            let str = encrypt(&read_stdin()?)?;
            print!("{str}");
        }
        SubCommand::Decrypt => {
            let str = decrypt(&read_stdin()?)?;
            print!("{str}");
        }
        SubCommand::TestFernet => {
            let str = read_stdin()?;
            let actual = encrypt(&str).and_then(|s| decrypt(&s))?;
            assert_eq!(str, actual);
            println!("OK");
        }
        SubCommand::EncryptWebhook => {
            let hook = url_to_webhook(&read_stdin()?)?;
            println!("{}", serde_json::to_string_pretty(&hook)?);
        }
        SubCommand::NewKey => {
            print!("{}", Fernet::generate_key());
        }
        SubCommand::DbTest => {
            test_database().await?;
        }
    };
    Ok(())
}
