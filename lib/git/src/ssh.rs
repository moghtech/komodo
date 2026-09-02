use std::{
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};

use anyhow::Context;
use komodo_client::entities::komodo_timestamp;

/// A Komodo-managed ssh private key, written to a temporary file for
/// the duration of a git command and removed afterwards.
///
/// When no key is configured on the git account, none of this applies -
/// git is invoked plainly and picks up the ssh config of the host it
/// runs on.
pub struct SshKeyFile {
  path: PathBuf,
}

impl SshKeyFile {
  /// Writes the key to a private temp file. Returns `Ok(None)` if there
  /// is no key to write, in which case the host's ssh config is used.
  pub async fn maybe_write(
    key: Option<&str>,
  ) -> anyhow::Result<Option<SshKeyFile>> {
    let Some(key) = key.map(str::trim).filter(|key| !key.is_empty())
    else {
      return Ok(None);
    };

    static COUNT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
      "komodo-git-key-{}-{}-{}",
      std::process::id(),
      komodo_timestamp(),
      COUNT.fetch_add(1, Ordering::Relaxed),
    ));

    // Create with 0600 up front - never let the key exist world readable,
    // even briefly.
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
      .open(&path)
      .await
      .context("Failed to create temporary file for ssh key")?;

    let res = async {
      use tokio::io::AsyncWriteExt;
      file.write_all(key.as_bytes()).await?;
      // ssh rejects a key file without a trailing newline.
      if !key.ends_with('\n') {
        file.write_all(b"\n").await?;
      }
      file.flush().await
    }
    .await
    .context("Failed to write temporary ssh key file");

    if res.is_err() {
      let _ = tokio::fs::remove_file(&path).await;
    }
    res?;

    Ok(Some(SshKeyFile { path }))
  }
}

impl Drop for SshKeyFile {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.path);
  }
}

/// The git binary to invoke - plain `git`, or `git` configured to use a
/// Komodo-managed key. The key path (never the key itself) appears in the
/// command, so it is safe to log.
pub fn git(key: Option<&SshKeyFile>) -> String {
  match key {
    Some(SshKeyFile { path }) => format!(
      "git -c core.sshCommand=\"ssh -i {} -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new\"",
      path.display()
    ),
    None => String::from("git"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn no_key_means_plain_git() {
    assert_eq!(git(None), "git");
  }

  #[tokio::test]
  async fn key_file_is_private_and_cleaned_up() {
    assert!(SshKeyFile::maybe_write(None).await.unwrap().is_none());
    assert!(
      SshKeyFile::maybe_write(Some("  ")).await.unwrap().is_none()
    );

    let key = SshKeyFile::maybe_write(Some("PRIVATE KEY"))
      .await
      .unwrap()
      .expect("key should have been written");
    let path = key.path.clone();

    let contents = tokio::fs::read_to_string(&path).await.unwrap();
    // Trailing newline is added, ssh rejects the key without it.
    assert_eq!(contents, "PRIVATE KEY\n");

    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mode = tokio::fs::metadata(&path)
        .await
        .unwrap()
        .permissions()
        .mode();
      assert_eq!(mode & 0o777, 0o600);
    }

    // The key path, never the key itself, ends up in the command.
    let command = git(Some(&key));
    assert!(command.contains(&path.display().to_string()));
    assert!(!command.contains("PRIVATE KEY"));

    drop(key);
    assert!(!path.exists(), "key file should be removed on drop");
  }
}
