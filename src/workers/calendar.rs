// Enviame - Full-stack Priority Messenger with a Rust backend that respects priority settings and delivers messages.
// Copyright (C) 2025 Brian Chen (differental)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use chrono::{DateTime, NaiveTime, Utc};
use icalendar::{Calendar, CalendarComponent, Component, DatePerhapsTime, EventStatus};
use std::{env, time::Duration};
use tokio::time::interval;

use crate::constants::{CALENDAR_DATETIME_FORMAT, DEFAULT_TZ};
use crate::state::{AppState, CalendarCache};

const ZERO_TIME: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

fn process_datetime(dt: DatePerhapsTime) -> Option<DateTime<Utc>> {
    match dt {
        DatePerhapsTime::Date(date) => Some(
            date.and_time(ZERO_TIME)
                .and_local_timezone(*DEFAULT_TZ)
                .unwrap()
                .to_utc(),
        ),
        DatePerhapsTime::DateTime(dt) => dt.try_into_utc(),
    }
}

async fn get_busy_status(url: &str) -> anyhow::Result<(bool, DateTime<Utc>)> {
    let contents = reqwest::get(url).await?.text().await?;
    let calendar = contents
        .parse::<Calendar>()
        .map_err(|e| anyhow::anyhow!(e))?;

    let now = Utc::now();
    let tomorrow_now = Utc::now() + chrono::Duration::days(1);

    // Rule for events:
    // 1. Dates (All-day events) become 00.00 in user-specified timezone.
    //      Note that for all-day events, DTEND is (typically) the day
    //      after the actual end date, so this works as intended.
    // 2. Must have both a valid dt_start and a valid dt_end

    let mut blocking_datetimes = Vec::<(DateTime<Utc>, DateTime<Utc>)>::new();

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
                    if dt_end >= now && dt_start <= tomorrow_now {
                        // Handling finished events has no point
                        //    but it might be worth considering whether
                        //    including ALL future events will be more helpful
                        blocking_datetimes.push((dt_start, dt_end));
                    }
                }
            }
        }
    }

    let get_time = |offset_days: i64, hour: u32| {
        (now + chrono::Duration::days(offset_days))
            .date_naive()
            .and_hms_opt(hour, 0, 0)
            .unwrap()
            .and_local_timezone(*DEFAULT_TZ)
            .unwrap()
            .to_utc()
    };

    // Nighttime configuration - configured at 23.00-08.00 in user-configured timezone
    const START_HOUR: u32 = 23;
    const END_HOUR: u32 = 8;

    // Add two blocking periods: Yesterday 23.00 to today 08.00, and today 23.00 to tomorrow 08.00
    let yesterday_start = get_time(-1, START_HOUR);
    let today_end = get_time(0, END_HOUR);
    let today_start = get_time(0, START_HOUR);
    let tomorrow_end = get_time(1, END_HOUR);

    if today_end >= now {
        blocking_datetimes.push((yesterday_start, today_end));
    }
    blocking_datetimes.push((today_start, tomorrow_end));
    blocking_datetimes.sort();

    let mut is_busy = false;
    // if busy, return this: the latest (from now) where the calendar is free
    let mut last_dt_end = now;
    // if not busy, return this: the earliest where the calendar isn't free
    // This initilisation has no point since blocking_datetimes cannot be empty
    let mut first_dt_start = now;

    for (dt_start, dt_end) in blocking_datetimes {
        if dt_start > last_dt_end {
            first_dt_start = dt_start;
            break;
        }
        is_busy = true;
        last_dt_end = last_dt_end.max(dt_end);
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
                consecutive_fail_count = 0;

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
                eprintln!("Calendar worker failed to update busy status: {err:?}");
                if consecutive_fail_count >= MAX_CONSECUTIVE_FAILS {
                    return;
                }
            }
        }
    }
}
