// sidekwest bot
// sidekwest schedule <FILE>

use std::io::{stdin, Read};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use fernet::Fernet;
use regex::Regex;

use self::schedule::Webhook;
use self::secrecy::{decrypt, encrypt};

mod bot;
mod schedule;
mod secrecy;

#[derive(Debug, Parser, Clone)]
struct CLI {
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
    let caps = re.captures(s).ok_or_else(|| anyhow!("Did not match webhook URL"))?;
    let id = caps.name("id").ok_or_else(|| anyhow!("Could not find webhook id"))?.as_str();
    let token = caps.name("token").ok_or_else(|| anyhow!("Could not find webhook token"))?.as_str();
    let hook = Webhook ::new(id.parse()?, encrypt(token)?);
    Ok(hook)
}

fn main() -> Result<()> {
    load_dotenv();
    match CLI::parse().command {
        SubCommand::Bot => bot::bot_main(),
        SubCommand::Schedule { file } => schedule::run_schedule_update(file)?,
        SubCommand::Encrypt => {
            let str = encrypt(&read_stdin()?)?;
            print!("{str}");
        },
        SubCommand::Decrypt => {
            let str = decrypt(&read_stdin()?)?;
            print!("{str}");
        },
        SubCommand::TestFernet => {
            let str = read_stdin()?;
            let actual = encrypt(&str).and_then(|s| decrypt(&s))?;
            assert_eq!(str, actual);
            println!("OK");
        },
        SubCommand::EncryptWebhook => {
            let hook = url_to_webhook(&read_stdin()?)?;
            println!("{}", serde_json::to_string_pretty(&hook)?);
        },
        SubCommand::NewKey => {
            print!("{}", Fernet::generate_key());
        }
    };
    Ok(())
}
