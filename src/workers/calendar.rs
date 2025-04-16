use chrono::{DateTime, NaiveTime, TimeZone, Timelike, Utc};
use icalendar::{Calendar, CalendarComponent, Component, DatePerhapsTime, EventStatus};
use std::env;
use std::time::Duration;
use tokio::time::interval;

use crate::constants::CALENDAR_DATETIME_FORMAT;
use crate::state::{AppState, CalendarCache};

static ZERO_TIME: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

fn process_datetime(dt: DatePerhapsTime) -> Option<DateTime<Utc>> {
    match dt {
        DatePerhapsTime::Date(date) => Some(date.and_time(ZERO_TIME).and_utc()),
        DatePerhapsTime::DateTime(dt) => dt.try_into_utc(),
    }
}

async fn get_busy_status(url: &str) -> anyhow::Result<(bool, DateTime<Utc>)> {
    let contents = reqwest::get(url).await?.text().await?;
    let calendar = contents
        .parse::<Calendar>()
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut is_busy = false;
    // if busy, return this: the latest (from now) where the calendar is free
    let mut last_dt_end = chrono::Utc::now();
    // if not busy, return this: the earliest where the calendar isn't free
    let mut first_dt_start = Utc.with_ymd_and_hms(2099, 12, 31, 23, 59, 59).unwrap();

    // Rule for events:
    // 1. Dates become 00.00, hence all-day one-day event doesn't count but all-day multi-day events do count
    // 2. Must have both a valid dt_start and a valid dt_end
    // 3. Assume no overlapping events (if there's a 9-11 and a 10-12, it will report 11 not 12)
    for component in &calendar.components {
        if let CalendarComponent::Event(event) = component {
            if let Some(status) = event.get_status() {
                if status == EventStatus::Cancelled {
                    continue;
                }
            }

            if let (Some(dt_start), Some(dt_end)) = (event.get_start(), event.get_end()) {
                if let (Some(dt_start), Some(dt_end)) =
                    (process_datetime(dt_start), process_datetime(dt_end))
                {
                    if dt_start < chrono::Utc::now()
                        && dt_end > chrono::Utc::now()
                        && dt_end > last_dt_end
                    {
                        is_busy = true;
                        last_dt_end = dt_end;
                    } else if !is_busy && dt_start > chrono::Utc::now() && dt_start < first_dt_start
                    {
                        first_dt_start = dt_start;
                    }
                }
            }
        }
    }

    // Nighttime configuration - configured at 22.00-07.00 **UTC**
    const START_HOUR: u32 = 22;
    const END_HOUR: u32 = 7;

    let now = chrono::Utc::now();
    let hour = now.hour();
    if hour >= START_HOUR || hour < END_HOUR {
        is_busy = true;
        let next_morning = if hour < END_HOUR {
            now.date_naive()
                .and_hms_opt(END_HOUR, 0, 0)
                .unwrap()
                .and_utc()
        } else {
            (now + chrono::Duration::days(1))
                .date_naive()
                .and_hms_opt(END_HOUR, 0, 0)
                .unwrap()
                .and_utc()
        };
        if next_morning > last_dt_end {
            last_dt_end = next_morning;
        }
    }
    if !is_busy {
        let this_night = now
            .date_naive()
            .and_hms_opt(START_HOUR, 0, 0)
            .unwrap()
            .and_utc();
        let next_night_start = if now < this_night {
            this_night
        } else {
            (now + chrono::Duration::days(1))
                .date_naive()
                .and_hms_opt(START_HOUR, 0, 0)
                .unwrap()
                .and_utc()
        };
        if next_night_start < first_dt_start {
            first_dt_start = next_night_start;
        }
    }

    Ok((is_busy, if is_busy { last_dt_end } else { first_dt_start }))
}

pub async fn calendar_worker(state: AppState) {
    let calendar_url = match env::var("CALENDAR_URL") {
        Ok(url) => url,
        Err(_) => {
            return;
        }
    };

    const MAX_CONSECUTIVE_FAILS: u16 = 5;
    let mut consecutive_fail_count: u16 = 0;

    let mut interval = interval(Duration::from_secs(120));

    loop {
        interval.tick().await;

        match get_busy_status(&calendar_url).await {
            Ok(busy_status) => {
                let new_cache = CalendarCache {
                    is_busy: busy_status.0,
                    timestamp: busy_status.1.format(CALENDAR_DATETIME_FORMAT).to_string(),
                };

                {
                    let old_cache = state.status.read().await;
                    if *old_cache == new_cache {
                        continue;
                    }
                }
                {
                    let mut old_cache = state.status.write().await;
                    *old_cache = new_cache;
                }
            }
            Err(ref err) => {
                consecutive_fail_count += 1;
                eprintln!("Calendar worker failed to update busy status: {:?}", err);
                if consecutive_fail_count >= MAX_CONSECUTIVE_FAILS {
                    return;
                }
            }
        }
    }
}
