use std::time::SystemTime;

use color_eyre::eyre::Error;
use nutype::nutype;
use poise::serenity_prelude::{GetMessages, MessageId};
use poise::{serenity_prelude as serenity, Framework, FrameworkOptions, PrefixFrameworkOptions};


struct UserData {
    start_time: SystemTime,
}
type Context<'a> = poise::Context<'a, UserData, Error>;

#[nutype(derive(Debug, From, Into))]
struct ShouldDelete(bool);

async fn _purge(ctx: &Context<'_>, f: impl Fn(MessageId) -> ShouldDelete) -> Result<(), Error> {
    let chan = ctx.guild_channel().await.unwrap();
    let mut messages = chan.messages(ctx, GetMessages::new().limit(100)).await?;
    messages.retain(|msg| f(msg.id).into_inner());
    if messages.len() == 0 {
        return Ok(());
    }
    chan.delete_messages(ctx, messages).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
async fn audit(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    // role_ingest::download_roles(&ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, required_permissions = "ADMINISTRATOR")]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let timestamp = ctx
        .data()
        .start_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let msg = format!("Bot started <t:{timestamp}:R>");
    ctx.reply(msg).await?;
    Ok(())
}

pub async fn bot_main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing env var: DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework: Framework<UserData, Error> = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![uptime(), register()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("sk!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|_, _, _| {
            Box::pin(async move {
                println!("Starting");
                Ok(UserData {
                    start_time: SystemTime::now(),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to build client");
    client.start().await.expect("client failure");
}
