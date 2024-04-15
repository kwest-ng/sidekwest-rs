use std::collections::VecDeque;
use std::time::Duration;
use std::{env, thread};

use color_eyre::eyre::{eyre, Context as _};
use color_eyre::Result;
use regex::Regex;
use reqwest::header::{self, HeaderMap};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::secrecy::{self, encrypt};

use super::data::{Snowflake, Webhook};

const API_URL: &str = "https://discord.com/api/v9";

#[derive(Debug, Deserialize, Clone)]
struct MessageId {
    id: String,
}

#[derive(Debug, Deserialize, Copy, Clone)]
struct RateLimitResponse {
    retry_after: f32,
}

fn get_webhook_url(webhook: &Webhook) -> Result<String> {
    let wh_token = secrecy::decrypt(webhook.encrypted_token())?;
    Ok(format!("{API_URL}/webhooks/{}/{}", webhook.id(), wh_token))
}

pub async fn post_webhook(client: &Client, webhook: &Webhook, content: &str) -> Result<()> {
    let webhook_base_url = get_webhook_url(webhook)?;
    let body = json!({
        "content": content
    });
    client
        .post(&webhook_base_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn upsert_webhook(
    client: &Client,
    webhook: &Webhook,
    mesg_id: &Option<Snowflake>,
    content: &str,
) -> Result<()> {
    let webhook_base_url = get_webhook_url(webhook)?;

    let mut sticky_mesg = None;
    if let Some(id) = mesg_id {
        let url = format!("{webhook_base_url}/messages/{id}");
        if client.get(url).send().await?.status().is_success() {
            sticky_mesg = Some(id);
        } else {
            eprintln!("Message id was provided, but missing");
        };
    };
    let body = json!({
        "content": content
    });
    match sticky_mesg {
        Some(id) => {
            let url = format!("{webhook_base_url}/messages/{id}");
            client
                .patch(url)
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
        }
        None => {
            client
                .post(&webhook_base_url)
                .query(&[("wait", true)])
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
        }
    };
    Ok(())
}

pub fn get_client() -> Result<Client> {
    let token = env::var("DISCORD_TOKEN")?;

    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, format!("Bot {token}").try_into()?);
    let client = Client::builder()
        .default_headers(headers)
        .user_agent("Discord Bot (https://twitch.tv/kwest_ng, v0.1-alpha)")
        .build()?;
    Ok(client)
}

const URL_PATTERN: &str = r"/webhooks/(?<id>.*?)/(?<token>.*?)/?$";

pub fn url_to_webhook(s: &str) -> Result<Webhook> {
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

pub async fn delete_messages(
    channel_id: Snowflake,
    mesgs: Vec<Snowflake>,
    client: &Client,
) -> Result<()> {
    let mut queue = VecDeque::from_iter(mesgs);
    while let Some(mesg) = queue.pop_front() {
        let url = format!("{API_URL}/channels/{channel_id}/messages/{mesg}");
        let resp = client.delete(url).send().await?;

        if resp.status().is_success() {
            eprintln!("Deleted message id: {mesg}");
            continue;
        }

        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            eprintln!("Rate limited!");
            queue.push_front(mesg);
        }

        let delay = resp.json::<RateLimitResponse>().await?.retry_after + 3.0;
        eprintln!("Sleeping for {delay}s to reset rate limits");
        thread::sleep(Duration::from_secs_f32(delay));
    }
    Ok(())
}

pub async fn get_channel_messages(
    channel_id: Snowflake,
    client: &Client,
) -> Result<Vec<Snowflake>> {
    let url = format!("{API_URL}/channels/{channel_id}/messages");
    let resp = client
        .get(url)
        .query(&[("limit", 100)])
        .send()
        .await
        .context("Sending get")?;
    resp.error_for_status()
        .context("Checking get response")?
        .json::<Vec<MessageId>>()
        .await?
        .into_iter()
        .map(|m_id| m_id.id.parse::<Snowflake>().map_err(Into::into))
        .collect::<Result<Vec<_>>>()
}
