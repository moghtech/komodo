use anyhow::{Context, anyhow};
use komodo_client::entities::{
  RepoExecutionArgs, RepoExecutionResponse,
  repo::Repo,
  sync::{ResourceSync, SyncFileContents},
  to_path_compatible_name,
  toml::ResourcesToml,
  update::Log,
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
  repo: Option<&Repo>,
) -> anyhow::Result<RemoteResources> {
  if sync.config.files_on_host {
    get_files_on_host(sync).await
  } else if let Some(repo) = repo {
    get_repo(sync, repo.into()).await
  } else if !sync.config.repo.is_empty() {
    get_repo(sync, sync.into()).await
  } else {
    get_ui_defined(sync).await
  }
}

async fn get_files_on_host(
  sync: &ResourceSync,
) -> anyhow::Result<RemoteResources> {
  let root_path = core_config()
    .sync_directory
    .join(to_path_compatible_name(&sync.name));
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
  Ok(RemoteResources {
    resources,
    files,
    file_errors,
    logs,
    hash: None,
    message: None,
  })
}

async fn get_repo(
  sync: &ResourceSync,
  mut clone_args: RepoExecutionArgs,
) -> anyhow::Result<RemoteResources> {
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

  let (
    RepoExecutionResponse {
      mut logs,
      commit_hash,
      commit_message,
      ..
    },
    _,
  ) = git::pull_or_clone(
    clone_args,
    &core_config().repo_directory,
    access_token,
  )
  .await
  .with_context(|| {
    format!("Failed to update resource repo at {repo_path:?}")
  })?;

  // Ensure clone / pull successful,
  // propogate error log -> 'errored' and return.
  if let Some(failure) = logs.iter().find(|log| !log.success) {
    return Ok(RemoteResources {
      resources: Err(anyhow!("Repo clone / pull failed")),
      files: Vec::new(),
      file_errors: vec![SyncFileContents {
        resource_path: String::from("Repo error"),
        path: format!("Failed at: {}", failure.stage),
        contents: failure.combined(),
      }],
      logs,
      hash: None,
      message: None,
    });
  }

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
    hash: commit_hash,
    message: commit_message,
  })
}

async fn get_ui_defined(
  sync: &ResourceSync,
) -> anyhow::Result<RemoteResources> {
  let mut resources = ResourcesToml::default();
  let resources =
    super::deserialize_resources_toml(&sync.config.file_contents)
      .context("failed to parse resource file contents")
      .map(|more| {
        extend_resources(
          &mut resources,
          more,
          &sync.config.match_tags,
        );
        resources
      });

  Ok(RemoteResources {
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
  })
}
