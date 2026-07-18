use std::path::Path;

use command::{CommandOptions, run_komodo_standard_command};
use formatting::format_serror;
use komodo_client::entities::{
  RepoExecutionArgs, all_logs_success, update::Log,
};

use crate::check_installed;

pub async fn init_folder_as_repo(
  folder_path: &Path,
  args: &RepoExecutionArgs,
  access_token: Option<&str>,
  logs: &mut Vec<Log>,
) {
  if let Err(e) = check_installed().await {
    logs.push(Log::error("Git Init", format_serror(&e.into())));
    return;
  };

  // Initialize the folder as a git repo
  let init_repo = run_komodo_standard_command(
    "Git Init",
    "git init",
    CommandOptions::default().path(folder_path),
  )
  .await;
  logs.push(init_repo);
  if !all_logs_success(logs) {
    return;
  }

  let repo_url = match args.remote_url(access_token) {
    Ok(url) => url,
    Err(e) => {
      logs
        .push(Log::error("Add git remote", format_serror(&e.into())));
      return;
    }
  };

  // Set remote url
  let mut set_remote = run_komodo_standard_command(
    "Add git remote",
    format!("git remote add origin {repo_url}"),
    CommandOptions::default().path(folder_path),
  )
  .await;
  // Sanitize the output
  if let Some(token) = &access_token {
    set_remote.command = set_remote.command.replace(token, "<TOKEN>");
    set_remote.stdout = set_remote.stdout.replace(token, "<TOKEN>");
    set_remote.stderr = set_remote.stderr.replace(token, "<TOKEN>");
  }
  if !set_remote.success {
    logs.push(set_remote);
    return;
  }

  // Set branch.
  let init_repo = run_komodo_standard_command(
    "Set Branch",
    format!("git switch -c {}", args.branch),
    CommandOptions::default().path(folder_path),
  )
  .await;
  if !init_repo.success {
    logs.push(init_repo);
  }
}
