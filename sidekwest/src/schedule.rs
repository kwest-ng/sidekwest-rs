use std::cell::Cell;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use color_eyre::eyre::{bail, eyre};
use color_eyre::Result;
use regex::Regex;
use serde::Deserialize;

use crate::discord::api::{
    delete_messages, get_channel_messages, get_client, post_webhook, upsert_webhook,
};
use crate::discord::data::{Snowflake, Webhook};

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
    ping_role: Snowflake,
    ping_message: String,
    schedule_header: Option<String>,
    schedule_footer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PostLocation {
    guild_id: Snowflake,
    channel_id: Snowflake,
    message_id: Option<Snowflake>,
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

impl Schedule {
    async fn publish_schedule(&self) -> Result<()> {
        let client = get_client()?;
        let mut mesgs: Vec<_> =
            get_channel_messages(self.post_location.channel_id, &client).await?;
        mesgs.retain(|id| Some(id) != self.post_location.message_id.as_ref());
        eprintln!("Found {} messages to delete", mesgs.len());
        delete_messages(self.post_location.channel_id, mesgs, &client).await?;
        upsert_webhook(
            &client,
            &self.post_location.webhook,
            &self.post_location.message_id,
            &self.events_to_message()?,
        )
        .await?;
        let ping_msg = format!(
            "{}\n\n<@&{}>",
            self.config.ping_message, self.config.ping_role
        );
        post_webhook(&client, &self.post_location.webhook, &ping_msg).await?;
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
        let today = Local::now().date_naive();
        Self::get_next_day_from(needle, today)
    }

    fn get_next_day_from(needle: &str, start: NaiveDate) -> Result<NaiveDate> {
        let mut candidate = start;
        if needle.len() > 3 {
            bail!("day of week too long: {needle}")
        };
        for _ in 0..10 {
            let weekday = candidate.weekday().to_string().to_ascii_lowercase();
            if weekday.starts_with(&needle.to_ascii_lowercase()) {
                return Ok(candidate);
            }
            candidate = candidate.succ_opt().unwrap();
        }
        bail!("Failed to match day of week")
    }

    fn parse_time_of_day(tod: &str) -> Result<NaiveTime> {
        let re = Regex::new(TIME_PATTERN)?;
        let caps = re
            .captures(tod)
            .ok_or_else(|| eyre!("Failed to match time to regexp"))?;
        let time_value = caps
            .name("time_value")
            .ok_or_else(|| eyre!("Failed to capture time_value"))?
            .as_str();
        let ampm = caps
            .name("ampm")
            .ok_or_else(|| eyre!("Failed to capture ampm"))?
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
        if ampm.contains(['P', 'p']) {
            hours += 12
        }
        let time = NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0)
            .expect("naivetime creation failed");
        Ok(time)
    }
}
