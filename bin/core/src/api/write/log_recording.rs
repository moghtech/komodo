use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::find_one_by_id,
  mongodb::{bson::{doc, oid::ObjectId}},
};
use std::str::FromStr;
use komodo_client::{
  api::write::{DeleteLogRecording, StartLogRecording, StopLogRecording},
  entities::{
    log_recording::{LogRecording, LogRecordingStatus},
    permission::PermissionLevel,
    komodo_timestamp, NoData,
  },
};
use resolver_api::Resolve;

use crate::{
  monitor::log_recorder::LogRecorderInstance,
  permission::get_check_permissions,
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for StartLogRecording {
  async fn resolve(
    self,
    WriteArgs { user, .. }: &WriteArgs,
  ) -> serror::Result<LogRecording> {
    // Check permission for the target resource based on type
    let (variant, id) = self.target.extract_variant_id();
    match variant {
      komodo_client::entities::ResourceTargetVariant::Server => {
        get_check_permissions::<komodo_client::entities::server::Server>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Stack => {
        get_check_permissions::<komodo_client::entities::stack::Stack>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Deployment => {
        get_check_permissions::<komodo_client::entities::deployment::Deployment>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      _ => return Err(anyhow!("Invalid target type for log recording").into()),
    }

    // Calculate expiry timestamp if duration specified
    let expire_ts = self.duration_days.map(|days| {
      komodo_timestamp() + (days * 24.0 * 60.0 * 60.0 * 1000.0) as i64
    });

    let recording = LogRecording {
      id: String::new(), // Will be set by insert_one
      target: self.target.clone(),
      name: self.name,
      user_id: user.id.clone(),
      start_ts: komodo_timestamp(),
      expire_ts,
      status: LogRecordingStatus::Recording,
      services: self.services,
    };

    // Insert into database
    let insert_result = db_client()
      .log_recordings
      .insert_one(&recording)
      .await
      .context("failed to insert log recording")?;

    let mut recording = recording;
    recording.id = insert_result.inserted_id.as_object_id().unwrap().to_hex();

    // Start the log recorder task for this recording
    LogRecorderInstance::spawn(recording.clone()).await;

    Ok(recording)
  }
}

impl Resolve<WriteArgs> for StopLogRecording {
  async fn resolve(
    self,
    WriteArgs { user, .. }: &WriteArgs,
  ) -> serror::Result<LogRecording> {
    let recording = find_one_by_id(&db_client().log_recordings, &self.id)
      .await
      .context("failed to find log recording")?
      .context("log recording not found")?;

    // Check permission for the target resource
    let (variant, id) = recording.target.extract_variant_id();
    match variant {
      komodo_client::entities::ResourceTargetVariant::Server => {
        get_check_permissions::<komodo_client::entities::server::Server>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Stack => {
        get_check_permissions::<komodo_client::entities::stack::Stack>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Deployment => {
        get_check_permissions::<komodo_client::entities::deployment::Deployment>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      _ => return Err(anyhow!("Invalid target type for log recording").into()),
    }

    // Update status to stopped
    let update = doc! { "$set": { "status": LogRecordingStatus::Stopped.to_string() } };
    db_client().log_recordings
      .update_one(
        doc! { "_id": ObjectId::from_str(&self.id)? },
        update
      )
      .await
      .context("failed to update log recording status")?;

    let mut updated = recording;
    updated.status = LogRecordingStatus::Stopped;

    Ok(updated)
  }
}

impl Resolve<WriteArgs> for DeleteLogRecording {
  async fn resolve(
    self,
    WriteArgs { user, .. }: &WriteArgs,
  ) -> serror::Result<NoData> {
    let recording = find_one_by_id(&db_client().log_recordings, &self.id)
      .await
      .context("failed to find log recording")?
      .context("log recording not found")?;

    // Check permission for the target resource
    let (variant, id) = recording.target.extract_variant_id();
    match variant {
      komodo_client::entities::ResourceTargetVariant::Server => {
        get_check_permissions::<komodo_client::entities::server::Server>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Stack => {
        get_check_permissions::<komodo_client::entities::stack::Stack>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      komodo_client::entities::ResourceTargetVariant::Deployment => {
        get_check_permissions::<komodo_client::entities::deployment::Deployment>(
          &id, user, PermissionLevel::Write.into()
        ).await?;
      }
      _ => return Err(anyhow!("Invalid target type for log recording").into()),
    }

    // Delete the recording
    db_client().log_recordings
      .delete_one(doc! { "_id": ObjectId::from_str(&self.id)? })
      .await
      .context("failed to delete log recording")?;

    // Delete associated logs
    db_client()
      .recorded_logs
      .delete_many(doc! { "recording_id": &self.id })
      .await
      .context("failed to delete recorded logs")?;

    Ok(NoData {})
  }
}