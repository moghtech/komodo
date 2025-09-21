use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::find_one_by_id,
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use komodo_client::{
  api::read::{GetLogRecording, GetRecordedLogs, ListLogRecordings, SearchRecordedLogs},
  entities::{
    log_recording::{LogRecording, LogRecordingListItem, LogRecordingStatus},
    permission::PermissionLevel,
    update::Log,
    SearchCombinator,
  },
};
use resolver_api::Resolve;

use crate::{
  helpers::query::get_user,
  permission::get_check_permissions,
  state::db_client,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetLogRecording {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<LogRecording> {
    let recording = find_one_by_id(&db_client().log_recordings, &self.id)
      .await
      .context("failed to find log recording")?
      .context("log recording not found")?;

    // Check permissions for the target resource based on type
    let (variant, id) = recording.target.extract_variant_id();
    match variant {
      komodo_client::entities::ResourceTargetVariant::Server => {
        get_check_permissions::<komodo_client::entities::server::Server>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Stack => {
        get_check_permissions::<komodo_client::entities::stack::Stack>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Deployment => {
        get_check_permissions::<komodo_client::entities::deployment::Deployment>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      _ => return Err(anyhow!("Invalid target type for log recording").into()),
    }

    Ok(recording)
  }
}

impl Resolve<ReadArgs> for ListLogRecordings {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<LogRecordingListItem>> {
    let mut filter = doc! {};

    if let Some(target) = &self.target {
      filter.insert("target.type", target.extract_variant_id().0.to_string());
      filter.insert("target.id", target.extract_variant_id().1.clone());
    }

    if self.active_only {
      filter.insert("status", LogRecordingStatus::Recording.to_string());
    }

    let recordings = find_collect(
      &db_client().log_recordings,
      filter,
      None,
    )
    .await
    .context("failed to collect log recordings")?;

    let mut list_items = Vec::new();

    for recording in recordings {
      // Check permissions for each recording's target
      let (variant, id) = recording.target.extract_variant_id();
      let has_permission = match variant {
        komodo_client::entities::ResourceTargetVariant::Server => {
          get_check_permissions::<komodo_client::entities::server::Server>(
            &id, user, PermissionLevel::Read.into()
          ).await.is_ok()
        }
        komodo_client::entities::ResourceTargetVariant::Stack => {
          get_check_permissions::<komodo_client::entities::stack::Stack>(
            &id, user, PermissionLevel::Read.into()
          ).await.is_ok()
        }
        komodo_client::entities::ResourceTargetVariant::Deployment => {
          get_check_permissions::<komodo_client::entities::deployment::Deployment>(
            &id, user, PermissionLevel::Read.into()
          ).await.is_ok()
        }
        _ => false,
      };
      if !has_permission {
        continue;
      }

      let username = get_user(&recording.user_id)
        .await
        .map(|u| u.username)
        .unwrap_or_else(|_| recording.user_id.clone());

      list_items.push(LogRecordingListItem {
        id: recording.id,
        target: recording.target,
        name: recording.name,
        user_id: recording.user_id,
        username,
        start_ts: recording.start_ts,
        expire_ts: recording.expire_ts,
        status: recording.status,
        services: recording.services,
      });
    }

    Ok(list_items)
  }
}

impl Resolve<ReadArgs> for GetRecordedLogs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Log> {
    // First check the recording exists and user has permission
    let recording = find_one_by_id(&db_client().log_recordings, &self.recording_id)
      .await
      .context("failed to find log recording")?
      .context("log recording not found")?;

    let (variant, id) = recording.target.extract_variant_id();
    match variant {
      komodo_client::entities::ResourceTargetVariant::Server => {
        get_check_permissions::<komodo_client::entities::server::Server>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Stack => {
        get_check_permissions::<komodo_client::entities::stack::Stack>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Deployment => {
        get_check_permissions::<komodo_client::entities::deployment::Deployment>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      _ => return Err(anyhow!("Invalid target type for log recording").into()),
    }

    // Fetch recorded logs
    let filter = doc! { "recording_id": &self.recording_id };
    let options = FindOptions::builder()
      .sort(doc! { "ts": -1 })
      .limit(self.tail as i64)
      .build();

    let logs = find_collect(
      &db_client().recorded_logs,
      filter,
      Some(options),
    )
    .await
    .context("failed to collect recorded logs")?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    // Reverse to get chronological order
    let mut logs = logs;
    logs.reverse();

    for log in logs {
      if self.timestamps {
        let ts = chrono::DateTime::from_timestamp_millis(log.ts)
          .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
          .unwrap_or_default();

        if !log.stdout.is_empty() {
          stdout.push_str(&format!("{} {}\n", ts, log.stdout));
        }
        if !log.stderr.is_empty() {
          stderr.push_str(&format!("{} {}\n", ts, log.stderr));
        }
      } else {
        if !log.stdout.is_empty() {
          stdout.push_str(&log.stdout);
          if !log.stdout.ends_with('\n') {
            stdout.push('\n');
          }
        }
        if !log.stderr.is_empty() {
          stderr.push_str(&log.stderr);
          if !log.stderr.ends_with('\n') {
            stderr.push('\n');
          }
        }
      }
    }

    Ok(Log {
      stdout,
      stderr,
      ..Default::default()
    })
  }
}

impl Resolve<ReadArgs> for SearchRecordedLogs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Log> {
    // First check the recording exists and user has permission
    let recording = find_one_by_id(&db_client().log_recordings, &self.recording_id)
      .await
      .context("failed to find log recording")?
      .context("log recording not found")?;

    let (variant, id) = recording.target.extract_variant_id();
    match variant {
      komodo_client::entities::ResourceTargetVariant::Server => {
        get_check_permissions::<komodo_client::entities::server::Server>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Stack => {
        get_check_permissions::<komodo_client::entities::stack::Stack>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Deployment => {
        get_check_permissions::<komodo_client::entities::deployment::Deployment>(
          &id, user, PermissionLevel::Read.into()
        ).await?;
      }
      _ => return Err(anyhow!("Invalid target type for log recording").into()),
    }

    // Fetch all recorded logs for the recording
    let filter = doc! { "recording_id": &self.recording_id };
    let options = FindOptions::builder().sort(doc! { "ts": 1 }).build();

    let logs = find_collect(
      &db_client().recorded_logs,
      filter,
      Some(options),
    )
    .await
    .context("failed to collect recorded logs")?;

    let mut stdout = String::new();

    for log in logs {
      let combined = format!("{}\n{}", log.stdout, log.stderr);

      // Apply search filter
      let matches = if self.combinator == SearchCombinator::And {
        self.terms.iter().all(|term| combined.contains(term))
      } else {
        self.terms.iter().any(|term| combined.contains(term))
      };

      let should_include = if self.invert { !matches } else { matches };

      if should_include {
        if self.timestamps {
          let ts = chrono::DateTime::from_timestamp_millis(log.ts)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
            .unwrap_or_default();
          stdout.push_str(&format!("{} {}", ts, combined));
        } else {
          stdout.push_str(&combined);
        }
      }
    }

    Ok(Log {
      stdout,
      stderr: String::new(),
      ..Default::default()
    })
  }
}