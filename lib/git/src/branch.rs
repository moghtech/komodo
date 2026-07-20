use std::path::Path;

use anyhow::bail;
use command::{
  CommandOptions, run_komodo_standard_command, run_standard_command,
};
use formatting::format_serror;
use komodo_client::entities::{
  RepoExecutionResponse, all_logs_success, update::Log,
};

use crate::{
  check_installed, commit::ensure_global_git_config_set,
  get_commit_hash_log,
};

/// Validate a branch name before it is interpolated
/// into a shell command. Only allows conservative
/// git ref characters.
pub fn validate_branch_name(branch: &str) -> anyhow::Result<()> {
  if branch.is_empty() {
    bail!("Branch name cannot be empty");
  }
  if branch.starts_with('-') {
    bail!("Branch name cannot start with '-'");
  }
  if branch.starts_with('/') || branch.ends_with('/') {
    bail!("Branch name cannot start or end with '/'");
  }
  if branch.contains("..") {
    bail!("Branch name cannot contain '..'");
  }
  if !branch.chars().all(|c| {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')
  }) {
    bail!(
      "Branch name can only contain alphanumeric characters and '.', '_', '-', '/'"
    );
  }
  Ok(())
}

/// Create `branch` on the remote, pointing at the local HEAD,
/// without touching the local checkout.
/// Errors if the remote branch already exists.
/// Repo must be cloned.
pub async fn push_new_branch(
  repo_dir: &Path,
  branch: &str,
) -> Vec<Log> {
  let mut logs = Vec::new();

  if let Err(e) = check_installed().await {
    logs
      .push(Log::error("Push New Branch", format_serror(&e.into())));
    return logs;
  }

  // Refuse to adopt an existing remote branch, the caller
  // expects a branch it fully owns.
  let existing = run_standard_command(
    &format!("git ls-remote --heads origin {branch}"),
    CommandOptions::default().path(repo_dir),
  )
  .await;
  if !existing.success() {
    logs.push(Log::error(
      "Check Remote Branch",
      format!(
        "Failed to list remote branches | stdout: {} | stderr: {}",
        existing.stdout, existing.stderr
      ),
    ));
    return logs;
  }
  if !existing.stdout.trim().is_empty() {
    logs.push(Log::error(
      "Check Remote Branch",
      format!(
        "Remote branch '{branch}' already exists. Delete it first, or choose a different branch name."
      ),
    ));
    return logs;
  }

  logs.push(
    run_komodo_standard_command(
      "Push New Branch",
      format!("git push origin HEAD:refs/heads/{branch}"),
      CommandOptions::default().path(repo_dir),
    )
    .await,
  );

  logs
}

/// Escape `s` for interpolation inside a double-quoted
/// shell string.
fn escape_double_quoted(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for c in s.chars() {
    if matches!(c, '\\' | '"' | '`' | '$') {
      out.push('\\');
    }
    out.push(c);
  }
  out
}

/// Squash merge `source_branch` into the currently checked out
/// `base_branch`, commit, and push.
///
/// The commit subject is `[Komodo] {username}: {message}` when a
/// custom `message` is passed, otherwise the default
/// `[Komodo] {username}: Merge Stack Edits: update {files}`,
/// where `{files}` is the affected file path (or file count)
/// taken from the staged merge result.
///
/// On merge conflict, the working tree is reset so the repo dir
/// is left clean, and the failure is reflected in the logs.
/// Repo must be cloned with `base_branch` checked out and up to date.
pub async fn squash_merge(
  repo_dir: &Path,
  source_branch: &str,
  base_branch: &str,
  username: &str,
  message: Option<&str>,
) -> RepoExecutionResponse {
  let mut res = RepoExecutionResponse {
    path: repo_dir.to_path_buf(),
    logs: Vec::new(),
    commit_hash: None,
    commit_message: None,
  };

  if let Err(e) = check_installed().await {
    res
      .logs
      .push(Log::error("Squash Merge", format_serror(&e.into())));
    return res;
  }

  ensure_global_git_config_set().await;

  let fetch_log = run_komodo_standard_command(
    "Fetch Branch",
    format!("git fetch origin {source_branch}"),
    CommandOptions::default().path(repo_dir),
  )
  .await;
  res.logs.push(fetch_log);
  if !all_logs_success(&res.logs) {
    return res;
  }

  let merge_log = run_komodo_standard_command(
    "Squash Merge",
    "git merge --squash FETCH_HEAD",
    CommandOptions::default().path(repo_dir),
  )
  .await;
  if !merge_log.success {
    res.logs.push(merge_log);
    // Leave the disposable clone clean for later operations.
    // Overall failure is already reflected by the merge log.
    res.logs.push(
      run_komodo_standard_command(
        "Reset Working Tree",
        "git reset --hard HEAD",
        CommandOptions::default().path(repo_dir),
      )
      .await,
    );
    // `reset --hard` only restores tracked files. Untracked files
    // brought in by the squash must be cleaned as well.
    res.logs.push(
      run_komodo_standard_command(
        "Clean Working Tree",
        "git clean -fd",
        CommandOptions::default().path(repo_dir),
      )
      .await,
    );
    return res;
  }
  res.logs.push(merge_log);

  let message = match message {
    Some(message) => format!("{username}: {message}"),
    None => {
      // Describe what the merge actually changed,
      // matching the house `update {file}` message style.
      let staged = run_standard_command(
        "git diff --cached --name-only",
        CommandOptions::default().path(repo_dir),
      )
      .await;
      let files = staged
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
      match files.as_slice() {
        [] => format!("{username}: Merge Stack Edits"),
        [file] => {
          format!("{username}: Merge Stack Edits: update {file}")
        }
        files => format!(
          "{username}: Merge Stack Edits: update {} files",
          files.len()
        ),
      }
    }
  };

  let commit_log = run_komodo_standard_command(
    "Commit",
    format!(
      r#"git commit -m "[Komodo] {}""#,
      escape_double_quoted(&message)
    ),
    CommandOptions::default().path(repo_dir),
  )
  .await;
  let mut nothing_to_commit = false;
  if !commit_log.success {
    // The source branch may have no changes relative to base.
    // Still push below: a previous run may have committed
    // successfully and failed only on the push.
    if commit_log.stdout.contains("nothing to commit") {
      nothing_to_commit = true;
      res.logs.push(Log::simple(
        "Commit",
        String::from(
          "Nothing to commit, source branch has no changes",
        ),
      ));
    } else {
      res.logs.push(commit_log);
      return res;
    }
  } else {
    res.logs.push(commit_log);
  }

  if !nothing_to_commit {
    match get_commit_hash_log(repo_dir).await {
      Ok((log, hash, message)) => {
        res.logs.push(log);
        res.commit_hash = Some(hash);
        res.commit_message = Some(message);
      }
      Err(e) => {
        res.logs.push(Log::error(
          "Get commit hash",
          format_serror(&e.into()),
        ));
        return res;
      }
    };
  }

  res.logs.push(
    run_komodo_standard_command(
      "Push",
      format!("git push --set-upstream origin {base_branch}"),
      CommandOptions::default().path(repo_dir),
    )
    .await,
  );

  res
}

/// Delete `branch` on the remote.
/// Repo must be cloned.
pub async fn delete_remote_branch(
  repo_dir: &Path,
  branch: &str,
) -> Log {
  if let Err(e) = check_installed().await {
    return Log::error(
      "Delete Remote Branch",
      format_serror(&e.into()),
    );
  }
  run_komodo_standard_command(
    "Delete Remote Branch",
    format!("git push origin --delete {branch}"),
    CommandOptions::default().path(repo_dir),
  )
  .await
}

#[cfg(test)]
mod tests {
  use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
  };

  use super::*;

  static COUNTER: AtomicUsize = AtomicUsize::new(0);

  struct TestRepos {
    root: PathBuf,
  }

  impl TestRepos {
    fn remote(&self) -> PathBuf {
      self.root.join("remote.git")
    }
    fn work(&self) -> PathBuf {
      self.root.join("work")
    }
    /// Second clone, used to push commits to a branch
    /// independently of the main working clone.
    fn clone_branch(&self, branch: &str) -> PathBuf {
      let dir = self
        .root
        .join(format!("clone-{}", branch.replace('/', "-")));
      git(
        &self.root,
        &[
          "clone",
          "-b",
          branch,
          self.remote().to_str().unwrap(),
          dir.to_str().unwrap(),
        ],
      );
      set_git_user(&dir);
      dir
    }
  }

  impl Drop for TestRepos {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.root);
    }
  }

  fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
      .args(args)
      .current_dir(dir)
      .output()
      .expect("git should run");
    assert!(
      out.status.success(),
      "git {args:?} failed: {}",
      String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
  }

  fn set_git_user(dir: &Path) {
    git(dir, &["config", "user.email", "test@komo.do"]);
    git(dir, &["config", "user.name", "test"]);
  }

  fn commit_file(dir: &Path, file: &str, contents: &str, msg: &str) {
    std::fs::write(dir.join(file), contents).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-m", msg]);
    git(dir, &["push"]);
  }

  /// Bare remote + working clone with one
  /// initial commit pushed to main.
  fn setup() -> TestRepos {
    let root = std::env::temp_dir().join(format!(
      "komodo-git-branch-test-{}-{}",
      std::process::id(),
      COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&root).unwrap();
    let repos = TestRepos { root };
    git(
      &repos.root,
      &[
        "init",
        "--bare",
        "-b",
        "main",
        repos.remote().to_str().unwrap(),
      ],
    );
    git(
      &repos.root,
      &[
        "clone",
        repos.remote().to_str().unwrap(),
        repos.work().to_str().unwrap(),
      ],
    );
    set_git_user(&repos.work());
    commit_file(
      &repos.work(),
      "compose.yaml",
      "services: {}\n",
      "init",
    );
    repos
  }

  const EDIT_BRANCH: &str = "komodo/edit/test_stack";

  fn remote_heads(repos: &TestRepos, branch: &str) -> String {
    git(&repos.work(), &["ls-remote", "--heads", "origin", branch])
  }

  #[tokio::test]
  async fn push_new_branch_creates_remote_branch() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");
    assert!(
      remote_heads(&repos, EDIT_BRANCH)
        .contains(&format!("refs/heads/{EDIT_BRANCH}"))
    );
  }

  #[tokio::test]
  async fn push_new_branch_refuses_existing_branch() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(!all_logs_success(&logs), "{logs:?}");
    assert!(
      logs.iter().any(|log| log.stderr.contains("already exists")),
      "{logs:?}"
    );
  }

  #[tokio::test]
  async fn squash_merge_folds_commits_into_one() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    // Two commits on the edit branch from an independent clone.
    let edit = repos.clone_branch(EDIT_BRANCH);
    commit_file(&edit, "compose.yaml", "services:\n  a: {}\n", "one");
    commit_file(
      &edit,
      "compose.yaml",
      "services:\n  a: {}\n  b: {}\n",
      "two",
    );

    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      None,
    )
    .await;
    assert!(all_logs_success(&res.logs), "{res:?}");
    assert!(res.commit_hash.is_some());
    let message = res.commit_message.as_deref().unwrap();
    assert!(
      message.contains(
        "[Komodo] tester: Merge Stack Edits: update compose.yaml"
      ),
      "{message}"
    );

    // init + single squash commit on remote main.
    let count =
      git(&repos.work(), &["rev-list", "--count", "origin/main"]);
    assert_eq!(count.trim(), "2");
    let contents =
      std::fs::read_to_string(repos.work().join("compose.yaml"))
        .unwrap();
    assert!(contents.contains("b: {}"));
  }

  #[tokio::test]
  async fn squash_merge_conflict_fails_and_leaves_clean_tree() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    // Conflicting change on the edit branch.
    let edit = repos.clone_branch(EDIT_BRANCH);
    commit_file(
      &edit,
      "compose.yaml",
      "services:\n  a: {}\n",
      "edit",
    );

    // Conflicting change directly on main.
    commit_file(
      &repos.work(),
      "compose.yaml",
      "services:\n  z: {}\n",
      "base",
    );

    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      None,
    )
    .await;
    assert!(!all_logs_success(&res.logs), "{res:?}");

    // Working tree left clean, main not advanced.
    let status = git(&repos.work(), &["status", "--porcelain"]);
    assert_eq!(status.trim(), "");
    let count =
      git(&repos.work(), &["rev-list", "--count", "origin/main"]);
    assert_eq!(count.trim(), "2");
  }

  #[tokio::test]
  async fn squash_merge_with_no_changes_succeeds() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      None,
    )
    .await;
    assert!(all_logs_success(&res.logs), "{res:?}");
    assert!(res.commit_hash.is_none());
  }

  #[tokio::test]
  async fn squash_merge_default_message_counts_multiple_files() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    // One commit on the edit branch touching two files.
    let edit = repos.clone_branch(EDIT_BRANCH);
    std::fs::write(edit.join("compose.yaml"), "services:\n  a: {}\n")
      .unwrap();
    std::fs::write(edit.join(".env"), "FOO=bar\n").unwrap();
    git(&edit, &["add", "-A"]);
    git(&edit, &["commit", "-m", "two files"]);
    git(&edit, &["push"]);

    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      None,
    )
    .await;
    assert!(all_logs_success(&res.logs), "{res:?}");
    let message = res.commit_message.as_deref().unwrap();
    assert!(
      message.contains(
        "[Komodo] tester: Merge Stack Edits: update 2 files"
      ),
      "{message}"
    );
  }

  /// A previous merge may have committed locally but failed to
  /// push. The retry hits "nothing to commit", and must still
  /// push the orphan commit instead of dropping it.
  #[tokio::test]
  async fn squash_merge_pushes_orphan_commit_when_nothing_new() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    let edit = repos.clone_branch(EDIT_BRANCH);
    commit_file(
      &edit,
      "compose.yaml",
      "services:\n  a: {}\n",
      "edit",
    );

    // Simulate a prior run where the squash commit succeeded
    // locally but the push to origin failed.
    let work = repos.work();
    git(&work, &["fetch", "origin", EDIT_BRANCH]);
    git(&work, &["merge", "--squash", "FETCH_HEAD"]);
    git(&work, &["commit", "-m", "orphan"]);

    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      None,
    )
    .await;
    assert!(all_logs_success(&res.logs), "{res:?}");

    // The orphan commit must have reached the remote.
    let count =
      git(&repos.work(), &["rev-list", "--count", "origin/main"]);
    assert_eq!(count.trim(), "2");
  }

  /// A conflicting squash must also clean up untracked files
  /// it brought into the working tree, or they linger in the
  /// cache dir and pollute later reads.
  #[tokio::test]
  async fn squash_merge_conflict_cleans_untracked_files() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    // Edit branch: conflicting change + a brand new file.
    let edit = repos.clone_branch(EDIT_BRANCH);
    std::fs::write(edit.join("compose.yaml"), "services:\n  a: {}\n")
      .unwrap();
    std::fs::write(edit.join("extra.yaml"), "extra: true\n").unwrap();
    git(&edit, &["add", "-A"]);
    git(&edit, &["commit", "-m", "edit"]);
    git(&edit, &["push"]);

    // Conflicting change directly on main.
    commit_file(
      &repos.work(),
      "compose.yaml",
      "services:\n  z: {}\n",
      "base",
    );

    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      None,
    )
    .await;
    assert!(!all_logs_success(&res.logs), "{res:?}");

    // No untracked leftovers, clean status.
    assert!(!repos.work().join("extra.yaml").exists());
    let status = git(&repos.work(), &["status", "--porcelain"]);
    assert_eq!(status.trim(), "");
  }

  #[tokio::test]
  async fn squash_merge_custom_message_escapes_shell_chars() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    let edit = repos.clone_branch(EDIT_BRANCH);
    commit_file(
      &edit,
      "compose.yaml",
      "services:\n  a: {}\n",
      "edit",
    );

    // Message with shell-special characters must be
    // committed literally, not interpreted.
    let res = squash_merge(
      &repos.work(),
      EDIT_BRANCH,
      "main",
      "tester",
      Some(r#"add service "a" $now `ok`"#),
    )
    .await;
    assert!(all_logs_success(&res.logs), "{res:?}");
    let message = res.commit_message.as_deref().unwrap();
    assert!(
      message
        .contains(r#"[Komodo] tester: add service "a" $now `ok`"#),
      "{message}"
    );
  }

  #[tokio::test]
  async fn delete_remote_branch_removes_branch() {
    let repos = setup();
    let logs = push_new_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(all_logs_success(&logs), "{logs:?}");

    let log = delete_remote_branch(&repos.work(), EDIT_BRANCH).await;
    assert!(log.success, "{log:?}");
    assert_eq!(remote_heads(&repos, EDIT_BRANCH).trim(), "");
  }

  #[test]
  fn validate_branch_name_accepts_valid_names() {
    for name in [
      "main",
      "komodo/edit/my-stack",
      "feature/x.y_z",
      "release-1.2.3",
    ] {
      assert!(
        validate_branch_name(name).is_ok(),
        "{name} should be valid"
      );
    }
  }

  #[test]
  fn validate_branch_name_rejects_invalid_names() {
    for name in [
      "",
      "-flag",
      "has space",
      "a;b",
      "a..b",
      "/leading",
      "trailing/",
      "$(injection)",
      "a\nb",
    ] {
      assert!(
        validate_branch_name(name).is_err(),
        "{name:?} should be invalid"
      );
    }
  }
}
