use anyhow::Context;
use command::run_standard_command;
use komodo_client::entities::RepoExecutionArgs;

/// Structured view of the remote repository's state,
/// used to decide whether a pull/clone can succeed or
/// whether we can bootstrap from an empty remote.
#[derive(Debug, Clone)]
pub enum RemoteState {
  /// Remote is reachable and the configured branch ref exists.
  BranchExists,
  /// Remote is reachable and has refs, but not the configured branch.
  BranchMissing { available_branches: Vec<String> },
  /// Remote is reachable but has zero refs (freshly created empty repo).
  Empty,
  /// Remote is not reachable. Could be: repo not found, auth failure,
  /// network error. `stderr` is scrubbed of the access token.
  Unreachable { stderr: String },
}

/// Cheap read-only probe of the remote via `git ls-remote`.
/// Uses the raw `run_standard_command` so the token-embedded URL
/// never reaches a log record. `stderr` returned in the Unreachable
/// variant is scrubbed.
pub async fn check_remote_state(
  args: &RepoExecutionArgs,
  access_token: Option<&str>,
) -> anyhow::Result<RemoteState> {
  let repo_url = args
    .remote_url(access_token)
    .context("Failed to build remote URL for ls-remote probe")?;

  let output = run_standard_command(
    &format!("git ls-remote {repo_url}"),
    None,
  )
  .await;

  if !output.success() {
    let mut stderr = output.stderr;
    if let Some(token) = access_token {
      stderr = stderr.replace(token, "<TOKEN>");
    }
    return Ok(RemoteState::Unreachable { stderr });
  }

  let stdout = output.stdout.trim();
  if stdout.is_empty() {
    return Ok(RemoteState::Empty);
  }

  let branch_ref = format!("refs/heads/{}", args.branch);
  let mut has_branch = false;
  let mut available_branches = Vec::new();
  for line in stdout.lines() {
    let Some(refname) = line.split_whitespace().nth(1) else {
      continue;
    };
    if refname == branch_ref {
      has_branch = true;
    }
    if let Some(branch) = refname.strip_prefix("refs/heads/") {
      available_branches.push(branch.to_string());
    }
  }

  if has_branch {
    Ok(RemoteState::BranchExists)
  } else {
    Ok(RemoteState::BranchMissing { available_branches })
  }
}
