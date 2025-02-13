use anyhow::{anyhow, Context};
use git::GitRes;
use komodo_client::entities::{
  sync::{ResourceSync, SyncFileContents},
  to_komodo_name,
  toml::ResourcesToml,
  update::Log,
  CloneArgs,
};

use crate::{config::core_config, helpers::git_token};

use super::file::extend_resources;

pub struct RemoteResources {
  pub resources: anyhow::Result<ResourcesToml>,
  pub files: Vec<SyncFileContents>,
  pub file_errors: Vec<SyncFileContents>,
  pub logs: Vec<Log>,
  pub hash: Option<String>,
  pub message: Option<String>,
}

/// Use `match_tags` to filter resources by tag.
pub async fn get_remote_resources(
  sync: &ResourceSync,
) -> anyhow::Result<RemoteResources> {
  if sync.config.files_on_host {
    // =============
    // FILES ON HOST
    // =============
    let root_path = core_config()
      .sync_directory
      .join(to_komodo_name(&sync.name));
    let (mut logs, mut files, mut file_errors) =
      (Vec::new(), Vec::new(), Vec::new());
    let resources = super::file::read_resources(
      &root_path,
      &sync.config.resource_path,
      &sync.config.match_tags,
      &mut logs,
      &mut files,
      &mut file_errors,
    );
    return Ok(RemoteResources {
      resources,
      files,
      file_errors,
      logs,
      hash: None,
      message: None,
    });
  } else if sync.config.repo.is_empty() {
    // ==========
    // UI DEFINED
    // ==========
    let mut resources = ResourcesToml::default();
    let resources = if !sync.config.file_contents.is_empty() {
      super::deserialize_resources_toml(&sync.config.file_contents)
        .context("failed to parse resource file contents")
        .map(|more| {
          extend_resources(
            &mut resources,
            more,
            &sync.config.match_tags,
          );
          resources
        })
    } else {
      Ok(resources)
    };

    return Ok(RemoteResources {
      resources,
      files: vec![SyncFileContents {
        resource_path: String::new(),
        path: "database file".to_string(),
        contents: sync.config.file_contents.clone(),
      }],
      file_errors: vec![],
      logs: vec![Log::simple(
        "Read from database",
        "Resources added from database file".to_string(),
      )],
      hash: None,
      message: None,
    });
  }

  // ===============
  // REPO BASED SYNC
  // ===============

  if sync.config.repo.is_empty() {
    return Err(anyhow!("No sync files configured"));
  }

  let mut clone_args: CloneArgs = sync.into();

  let access_token = if let Some(account) = &clone_args.account {
    git_token(&clone_args.provider, account, |https| clone_args.https = https)
      .await
      .with_context(
        || format!("Failed to get git token in call to db. Stopping run. | {} | {account}", clone_args.provider),
      )?
  } else {
    None
  };

  let repo_path =
    clone_args.unique_path(&core_config().repo_directory)?;
  clone_args.destination = Some(repo_path.display().to_string());
  // Don't want to run these on core.
  clone_args.on_clone = None;
  clone_args.on_pull = None;

  let GitRes {
    mut logs,
    hash,
    message,
    ..
  } = git::pull_or_clone(
    clone_args,
    &core_config().repo_directory,
    access_token,
    &[],
    "",
    None,
    &[],
  )
  .await
  .with_context(|| {
    format!("Failed to update resource repo at {repo_path:?}")
  })?;

  let hash = hash.context("failed to get commit hash")?;
  let message =
    message.context("failed to get commit hash message")?;

  let (mut files, mut file_errors) = (Vec::new(), Vec::new());
  let resources = super::file::read_resources(
    &repo_path,
    &sync.config.resource_path,
    &sync.config.match_tags,
    &mut logs,
    &mut files,
    &mut file_errors,
  );

  Ok(RemoteResources {
    resources,
    files,
    file_errors,
    logs,
    hash: Some(hash),
    message: Some(message),
  })
}
