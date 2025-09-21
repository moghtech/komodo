use std::str::FromStr;

use anyhow::Context;
use komodo_client::entities::{
  log_recording::{LogRecording, LogRecordingStatus, RecordedLog},
  komodo_timestamp,
  ResourceTargetVariant,
  stack::Stack,
  deployment::Deployment,
  server::Server,
};
use periphery_client::api::{
  compose::GetComposeLog,
  container::GetContainerLog,
};
use database::mungos::mongodb::bson::{doc, oid::ObjectId};
use derive_variants::ExtractVariant;
use tokio::time::{sleep, Duration};
use futures::StreamExt;

use crate::{
  helpers::periphery_client,
  resource,
  state::db_client,
};

/// Manages active log recording sessions
pub struct LogRecorderInstance;

impl LogRecorderInstance {
  /// Spawn a new log recording task
  pub async fn spawn(recording: LogRecording) {
    tokio::spawn(async move {
      if let Err(e) = Self::record_logs(recording).await {
        error!("Log recording error: {e:#}");
      }
    });
  }

  /// Main recording loop for a single recording
  async fn record_logs(recording: LogRecording) -> anyhow::Result<()> {
    let mut last_seen_lines = std::collections::HashSet::new();

    loop {
      // Check if recording should stop
      let current = db_client()
        .log_recordings
        .find_one(doc! { "_id": ObjectId::from_str(&recording.id)? })
        .await?;

      let should_stop = match &current {
        None => true, // Recording was deleted
        Some(rec) => {
          rec.status != LogRecordingStatus::Recording
            || rec
              .expire_ts
              .map(|exp| komodo_timestamp() > exp)
              .unwrap_or(false)
        }
      };

      if should_stop {
        // Update status if expired
        if let Some(rec) = current {
          if rec.status == LogRecordingStatus::Recording {
            db_client()
              .log_recordings
              .update_one(
                doc! { "_id": ObjectId::from_str(&recording.id)? },
                doc! { "$set": { "status": LogRecordingStatus::Expired.to_string() } },
              )
              .await?;
          }
        }
        break;
      }

      // Fetch new logs based on resource type
      let logs = Self::fetch_logs(&recording).await?;

      // Parse logs into stdout and stderr
      let (stdout, stderr) = Self::parse_logs(&logs);

      // Create a single log entry with all current logs
      if !stdout.is_empty() || !stderr.is_empty() {
        // Create hash of current content to detect changes
        let content_hash = format!("{}{}", stdout, stderr);

        // Only store if content has changed
        if !last_seen_lines.contains(&content_hash) {
          last_seen_lines.insert(content_hash);

          let log = RecordedLog {
            id: ObjectId::new().to_string(),
            recording_id: recording.id.clone(),
            ts: komodo_timestamp(),
            stdout,
            stderr,
            service: None,
          };

          db_client()
            .recorded_logs
            .insert_one(&log)
            .await?;
        }
      }

      // Sleep for 5 seconds before next poll
      sleep(Duration::from_secs(5)).await;
    }

    Ok(())
  }

  /// Fetch logs based on resource type
  async fn fetch_logs(recording: &LogRecording) -> anyhow::Result<String> {
    let (variant, id) = recording.target.extract_variant_id();

    match variant {
      ResourceTargetVariant::Stack => {
        let stack = resource::get::<Stack>(id).await?;
        let server = resource::get::<Server>(&stack.config.server_id).await?;
        let client = periphery_client(&server)?;

        let res = client
          .request(GetComposeLog {
            project: stack.project_name(false),
            services: recording.services.clone(),
            tail: 500,
            timestamps: false,
          })
          .await
          .context("Failed to get stack logs")?;

        Ok(format!("{}\n{}", res.stdout, res.stderr))
      }
      ResourceTargetVariant::Deployment => {
        let deployment = resource::get::<Deployment>(id).await?;
        let container_name = format!("{}-{}", deployment.name, deployment.id);
        let server = resource::get::<Server>(&deployment.config.server_id).await?;
        let client = periphery_client(&server)?;

        let res = client
          .request(GetContainerLog {
            name: container_name,
            tail: 500,
            timestamps: false,
          })
          .await
          .context("Failed to get deployment logs")?;

        Ok(format!("{}\n{}", res.stdout, res.stderr))
      }
      ResourceTargetVariant::Server => {
        // For containers on a server
        let server = resource::get::<Server>(id).await?;
        let client = periphery_client(&server)?;

        // Get container name from recording services (first one)
        let container = recording.services.first()
          .context("No container specified for server log recording")?;

        let res = client
          .request(GetContainerLog {
            name: container.clone(),
            tail: 500,
            timestamps: false,
          })
          .await
          .context("Failed to get container logs")?;

        Ok(format!("{}\n{}", res.stdout, res.stderr))
      }
      _ => anyhow::bail!("Log recording not supported for resource type: {}", variant),
    }
  }

  /// Parse log output into stdout and stderr
  fn parse_logs(logs: &str) -> (String, String) {
    // Split logs by newline separator (docker compose logs uses this pattern)
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    for line in logs.lines() {
      // Skip empty lines
      if line.trim().is_empty() {
        continue;
      }

      // Docker logs typically prefix stderr with ERROR, WARN, or have special markers
      // For now, we'll put everything in stdout unless it's clearly an error
      // In production, would parse docker log format more accurately
      if line.contains("ERROR") || line.contains("FATAL") || line.contains("PANIC") {
        stderr_lines.push(line);
      } else {
        stdout_lines.push(line);
      }
    }

    (stdout_lines.join("\n"), stderr_lines.join("\n"))
  }
}

/// Initialize log recorder on startup - check for active recordings
pub async fn init_log_recorder() -> anyhow::Result<()> {
  let active_recordings = db_client()
    .log_recordings
    .find(doc! { "status": LogRecordingStatus::Recording.to_string() })
    .await;

  if let Ok(mut cursor) = active_recordings {
    let mut recordings = Vec::new();
    while let Some(recording) = cursor.next().await {
      if let Ok(recording) = recording {
        recordings.push(recording);
      }
    }

    for recording in recordings {
      // Check if expired
      if let Some(expire_ts) = recording.expire_ts {
        if komodo_timestamp() > expire_ts {
          // Mark as expired
          let _ = db_client()
            .log_recordings
            .update_one(
              doc! { "_id": ObjectId::from_str(&recording.id).ok() },
              doc! { "$set": { "status": LogRecordingStatus::Expired.to_string() } },
            )
            .await;
          continue;
        }
      }

      // Restart recording
      LogRecorderInstance::spawn(recording).await;
    }
  }

  Ok(())
}