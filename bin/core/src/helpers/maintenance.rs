use anyhow::Context;
use chrono::{Datelike, Local};
use komodo_client::entities::{
  DayOfWeek,
  server::{MaintenanceScheduleType, MaintenanceWindow},
};

use crate::config::core_config;

/// Check if the current timestamp falls within this maintenance window
pub fn is_maintenance_window_active(
  window: &MaintenanceWindow,
  timestamp: i64,
) -> bool {
  if !window.enabled {
    return false;
  }

  let dt = chrono::DateTime::from_timestamp(timestamp / 1000, 0)
    .unwrap_or_else(|| chrono::Utc::now());

  let (local_time, local_weekday, local_date) =
    match (window.timezone.as_str(), core_config().timezone.as_str())
    {
      ("", "") => {
        let local_dt = dt.with_timezone(&Local);
        (local_dt.time(), local_dt.weekday(), local_dt.date_naive())
      }
      ("", timezone) | (timezone, _) => {
        let tz: chrono_tz::Tz = match timezone
          .parse()
          .context("Failed to parse timezone")
        {
          Ok(tz) => tz,
          Err(e) => {
            warn!(
              "Failed to parse maintenance window timezone: {e:#}"
            );
            return false;
          }
        };
        let local_dt = dt.with_timezone(&tz);
        (local_dt.time(), local_dt.weekday(), local_dt.date_naive())
      }
    };

  match &window.schedule_type {
    MaintenanceScheduleType::Daily => {
      is_time_in_window(window, local_time)
    }
    MaintenanceScheduleType::Weekly { day_of_week } => {
      convert_day_of_week(local_weekday) == *day_of_week
        && is_time_in_window(window, local_time)
    }
    MaintenanceScheduleType::OneTime { date } => {
      // Parse the date string and check if it matches current date
      if let Ok(maintenance_date) =
        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
      {
        local_date == maintenance_date
          && is_time_in_window(window, local_time)
      } else {
        false
      }
    }
  }
}

fn is_time_in_window(
  window: &MaintenanceWindow,
  current_time: chrono::NaiveTime,
) -> bool {
  let start_time = chrono::NaiveTime::from_hms_opt(
    window.hour as u32,
    window.minute as u32,
    0,
  )
  .unwrap_or(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());

  let end_time = start_time
    + chrono::Duration::minutes(window.duration_minutes as i64);

  // Handle case where maintenance window crosses midnight
  if end_time < start_time {
    current_time >= start_time || current_time <= end_time
  } else {
    current_time >= start_time && current_time <= end_time
  }
}

fn convert_day_of_week(value: chrono::Weekday) -> DayOfWeek {
  match value {
    chrono::Weekday::Mon => DayOfWeek::Monday,
    chrono::Weekday::Tue => DayOfWeek::Tuesday,
    chrono::Weekday::Wed => DayOfWeek::Wednesday,
    chrono::Weekday::Thu => DayOfWeek::Thursday,
    chrono::Weekday::Fri => DayOfWeek::Friday,
    chrono::Weekday::Sat => DayOfWeek::Saturday,
    chrono::Weekday::Sun => DayOfWeek::Sunday,
  }
}
