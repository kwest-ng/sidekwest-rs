use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, thread};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use regex::Regex;
use reqwest::header::{self, HeaderMap};
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::secrecy;

const TIME_PATTERN: &str = r"^\s*(?<time_value>[01]?\d(?::[0-5]\d)?)\s*(?<ampm>[AaPp][Mm]?)";

pub async fn run_schedule_update(file: PathBuf) -> Result<()> {
    let schedule: Schedule = serde_json::from_reader(BufReader::new(File::open(file)?))?;
    schedule.publish_schedule().await
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Schedule {
    config: ScheduleConfig,
    post_location: PostLocation,
    events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ScheduleConfig {
    timezone: String,
    ping_role: u64,
    ping_message: String,
    schedule_header: Option<String>,
    schedule_footer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PostLocation {
    guild_id: u64,
    channel_id: u64,
    message_id: Option<u64>,
    webhook: Webhook,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Event {
    title: String,
    day_of_week: String,
    time_of_day: String,
    #[serde(rename = "override")]
    override_mesg: Option<String>,
    skip: Option<bool>,
    _event_time: Cell<Option<DateTime<Local>>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Webhook {
    id: u64,
    encrypted_token: String,
}

impl Webhook {
    pub fn new(id: u64, encrypted_token: String) -> Self {
        Self {
            id,
            encrypted_token,
        }
    }
}

impl Schedule {
    async fn publish_schedule(&self) -> Result<()> {
        let token = env::var("DISCORD_TOKEN")?;

        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, format!("Bot {token}").try_into()?);
        let client = Client::builder()
            .default_headers(headers)
            .user_agent("Discord Bot (https://twitch.tv/kwest_ng, v0.1-alpha)")
            .build()?;

        let mesgs: Vec<u64> = get_channel_messages(self.post_location.channel_id, &client).await?
            .into_iter()
            .filter(|id| Some(*id) != self.post_location.message_id)
            .collect();
        eprintln!("Found {} messages to delete", mesgs.len());
        delete_messages(self.post_location.channel_id, mesgs, &client).await?;

        let wh_token = secrecy::decrypt(&self.post_location.webhook.encrypted_token)?;
        let webhook_base_url = format!(
            "{API_URL}/webhooks/{}/{}",
            self.post_location.webhook.id, wh_token
        );

        let mut sticky_mesg = None;
        if let Some(id) = self.post_location.message_id {
            let url = format!("{webhook_base_url}/messages/{id}");
            if client.get(url).send().await?.status().is_success() {
                sticky_mesg = Some(id);
            } else {
                eprintln!("Message id was provided, but missing");
            };
        };
        let body = json!({
            "content": self.events_to_message()?
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
        let ping_msg = format!(
            "{}\n\n<@&{}>",
            self.config.ping_message, self.config.ping_role
        );
        let body = json!({
            "content": ping_msg
        });
        client
            .post(&webhook_base_url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    fn events_to_message(&self) -> Result<String> {
        let mut buf = String::with_capacity(1000);
        if let Some(header) = &self.config.schedule_header {
            buf.write_str(header)?;
            buf.write_char('\n')?;
        };
        buf.write_str("Stream Schedule:\n")?;
        for event in self.events.iter() {
            if event.skip.unwrap_or(false) {
                continue;
            }
            buf.write_str(&format!("- {}\n", event.to_msg_line()))?;
        }
        buf.write_char('\n')?;
        if let Some(footer) = &self.config.schedule_footer {
            buf.write_str(footer)?;
            buf.write_char('\n')?;
        };
        buf.write_str(&format!("Updated at <t:{}:f>", Utc::now().timestamp()))?;
        Ok(buf)
    }
}

impl Event {
    fn to_msg_line(&self) -> String {
        if let Some(mesg) = &self.override_mesg {
            return mesg.clone();
        };
        let timestamp = self.event_time().timestamp();
        debug_assert!(timestamp > 0);
        let timestamp = timestamp as u64;
        format!("<t:{timestamp}:F> ⮕ {}", self.title)
    }

    fn event_time(&self) -> DateTime<Local> {
        let ev_time = self._event_time.get();
        if ev_time.is_none() {
            let dt = self._calc_datetime().unwrap();
            self._event_time.set(Some(dt));
        };
        self._event_time.get().unwrap()
    }

    fn _calc_datetime(&self) -> Result<DateTime<Local>> {
        let date = Self::get_next_day(&self.day_of_week)?;
        let time = Self::parse_time_of_day(&self.time_of_day)?;
        let datetime = NaiveDateTime::new(date, time)
            .and_local_timezone(Local)
            .unwrap();
        Ok(datetime)
    }

    fn get_next_day(needle: &str) -> Result<NaiveDate> {
        let mut today = Local::now().date_naive();
        if needle.len() > 3 {
            bail!("day of week too long: {needle}")
        };
        for _ in 0..10 {
            let weekday = today.weekday().to_string().to_ascii_lowercase();
            if weekday.starts_with(&needle.to_ascii_lowercase()) {
                return Ok(today);
            }
            today = today.succ_opt().unwrap();
        };
        bail!("Failed to match day of week")
    }

    fn parse_time_of_day(tod: &str) -> Result<NaiveTime> {
        let re = Regex::new(TIME_PATTERN)?;
        let caps = re
            .captures(tod)
            .ok_or_else(|| anyhow!("Failed to match time to regexp"))?;
        let time_value = caps
            .name("time_value")
            .ok_or_else(|| anyhow!("Failed to capture time_value"))?
            .as_str();
        let ampm = caps
            .name("ampm")
            .ok_or_else(|| anyhow!("Failed to capture ampm"))?
            .as_str();
        let (mut hours, minutes) = match time_value.split_once(':') {
            Some((hrs, mns)) => (
                hrs.parse::<u8>().expect("parse hours failed"),
                mns.parse::<u8>().expect("parse mins failed"),
            ),
            None => (
                time_value.parse::<u8>().expect("parse hours only failed"),
                0,
            ),
        };
        if ampm.contains(&['P', 'p']) {
            hours += 12
        }
        let time = NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0)
            .expect("naivetime creation failed");
        Ok(time)
    }
}

const API_URL: &str = "https://discord.com/api/v9";

#[derive(Debug, Deserialize, Clone)]
struct MessageId {
    id: String,
}

#[derive(Debug, Deserialize, Copy, Clone)]
struct RateLimitResponse {
    retry_after: f32,
}

async fn delete_messages(channel_id: u64, mesgs: Vec<u64>, client: &Client) -> Result<()> {
    let mut queue = VecDeque::from_iter(mesgs);
    loop {
        let mesg = match queue.pop_front() {
            Some(mesg) => mesg,
            None => break,
        };
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

async fn get_channel_messages(channel_id: u64, client: &Client) -> Result<Vec<u64>> {
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
        .map(|m_id| m_id.id.parse::<u64>().map_err(Into::into))
        .collect::<Result<Vec<_>>>()
}
