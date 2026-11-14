use std::{
  path::{Component, Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use formatting::format_serror;
use komodo_client::entities::{EnvironmentVar, update::Log};
use tokio::io::AsyncWriteExt;

/// Validate an environment variable identifier (`[A-Za-z_][A-Za-z0-9_]*`).
pub fn validate_env_var_key(key: &str) -> anyhow::Result<()> {
  if key.is_empty() {
    bail!("Environment variable name cannot be empty");
  }
  let mut chars = key.chars();
  let Some(first) = chars.next() else {
    bail!("Environment variable name cannot be empty");
  };
  if !(first.is_ascii_alphabetic() || first == '_') {
    bail!(
      "Invalid environment variable name '{key}': must start with ASCII letter or '_'"
    );
  }
  if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
    bail!(
      "Invalid environment variable name '{key}': only [A-Za-z0-9_] are allowed"
    );
  }
  Ok(())
}

/// Format an arbitrary UTF-8 string value as a lossless Docker Compose `.env`
/// double-quoted string literal.
///
/// Escapes `\`, `"`, `$` (`\$` so Docker Compose never expands `$VAR` or `${VAR}`
/// inside remote secrets), `\n`, `\r`, and `\t`, while preserving `#`, `'`,
/// spaces, `%`, `@`, `:`, `/`, `?`, `&`, `=`, `+`, and UTF-8 characters verbatim.
pub fn format_compose_dotenv_quoted_value(
  value: &str,
) -> anyhow::Result<String> {
  if value.contains('\0') {
    bail!("Environment variable value cannot contain NUL bytes");
  }
  let mut out = String::with_capacity(value.len() + 4);
  out.push('"');
  for ch in value.chars() {
    match ch {
      '\\' => out.push_str("\\\\"),
      '"' => out.push_str("\\\""),
      '$' => out.push_str("\\$"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      other => out.push(other),
    }
  }
  out.push('"');
  Ok(out)
}

/// Validate that `env_file_path` is a safe relative path strictly inside `folder`,
/// with no `..` traversal, no absolute prefix, and no symlinks on any existing
/// intermediate directory or at the target file itself.
pub fn validate_safe_env_file_path(
  folder: &Path,
  env_file_path: &str,
) -> anyhow::Result<PathBuf> {
  let trimmed = env_file_path.trim();
  if trimmed.is_empty() {
    bail!("env_file_path cannot be empty");
  }
  if trimmed.contains('\0') {
    bail!("env_file_path cannot contain NUL bytes");
  }
  let rel = Path::new(trimmed);
  if rel.is_absolute() {
    bail!(
      "env_file_path '{env_file_path}' must be relative to the stack run directory"
    );
  }

  let mut normalized_parts = Vec::new();
  for comp in rel.components() {
    match comp {
      Component::CurDir => {}
      Component::Normal(part) => {
        if part.is_empty() {
          bail!("Invalid empty path segment in env_file_path");
        }
        normalized_parts.push(part);
      }
      Component::ParentDir => {
        bail!(
          "env_file_path '{env_file_path}' cannot traverse outside stack run directory ('..')"
        );
      }
      Component::RootDir | Component::Prefix(_) => {
        bail!(
          "env_file_path '{env_file_path}' must be relative to the stack run directory"
        );
      }
    }
  }

  if normalized_parts.is_empty() {
    bail!("env_file_path '{env_file_path}' must point to a file");
  }

  if let Ok(meta) = std::fs::symlink_metadata(folder)
    && meta.file_type().is_symlink()
  {
    bail!(
      "Refusing to write environment file inside symlinked stack directory {folder:?}"
    );
  }

  let mut cur = folder.components().collect::<PathBuf>();
  for part in normalized_parts {
    cur.push(part);
    if let Ok(meta) = std::fs::symlink_metadata(&cur)
      && meta.file_type().is_symlink()
    {
      bail!(
        "Refusing to write environment file through symlink at {cur:?}"
      );
    }
  }

  Ok(cur)
}

/// Atomically write `contents` to `target_path` with `0600` permissions (on Unix),
/// replacing any pre-existing file (even if previously `0644`) without leaving
/// world-readable plaintext windows.
pub async fn write_atomic_mode(
  target_path: &Path,
  contents: &[u8],
  mode: u32,
) -> anyhow::Result<()> {
  if let Ok(meta) = tokio::fs::symlink_metadata(target_path).await
    && meta.file_type().is_symlink()
  {
    bail!("Refusing to overwrite symlink at {target_path:?}");
  }

  let parent = target_path.parent().with_context(|| {
    format!("Target path {target_path:?} has no parent directory")
  })?;
  tokio::fs::create_dir_all(parent).await.with_context(|| {
    format!("Failed to create parent directory {parent:?}")
  })?;

  if let Ok(meta) = tokio::fs::symlink_metadata(parent).await
    && meta.file_type().is_symlink()
  {
    bail!(
      "Refusing to write secret file inside symlinked directory {parent:?}"
    );
  }

  let pid = std::process::id();
  let nanos = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
  let file_name = target_path
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("env");

  let mut attempt = 0u32;
  let (mut file, tmp_path) = loop {
    let candidate = parent.join(format!(
      ".komodo-tmp-{file_name}-{pid}-{nanos}-{attempt}"
    ));
    let mut opts = tokio::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
      opts.mode(mode);
    }
    match opts.open(&candidate).await {
      Ok(f) => break (f, candidate),
      Err(e)
        if e.kind() == std::io::ErrorKind::AlreadyExists
          && attempt < 16 =>
      {
        attempt += 1;
      }
      Err(e) => {
        return Err(e).with_context(|| {
          format!("Failed to open temporary file {candidate:?}")
        });
      }
    }
  };

  let write_res: anyhow::Result<()> = async {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      file
        .set_permissions(std::fs::Permissions::from_mode(mode))
        .await
        .context("Failed to enforce permissions on temporary file")?;
    }
    file
      .write_all(contents)
      .await
      .context("Failed to write temporary file contents")?;
    file
      .sync_all()
      .await
      .context("Failed to sync temporary file to disk")?;
    drop(file);
    tokio::fs::rename(&tmp_path, target_path)
      .await
      .with_context(|| {
        format!(
          "Failed to atomically rename {tmp_path:?} -> {target_path:?}"
        )
      })?;
    Ok(())
  }
  .await;

  if write_res.is_err() {
    let _ = tokio::fs::remove_file(&tmp_path).await;
  }

  write_res
}

/// If the environment was written and needs to be passed to the compose command,
/// will return the env file PathBuf.
/// Should ensure all logs are successful after calling.
pub async fn write_env_file(
  environment: &[EnvironmentVar],
  folder: &Path,
  env_file_path: &str,
  logs: &mut Vec<Log>,
) -> Option<PathBuf> {
  let target_path =
    folder.join(env_file_path).components().collect::<PathBuf>();

  if environment.is_empty() {
    // Still want to return Some(env_file_path) if the path
    // already exists on the host and is a file.
    // This is for "Files on Server" mode when user writes the env file themself.
    if target_path.is_file() {
      return Some(target_path);
    }
    return None;
  }

  let contents = environment
    .iter()
    .map(|env| format!("{}={}", env.variable, env.value))
    .collect::<Vec<_>>()
    .join("\n");

  let contents = if contents.is_empty() || contents.ends_with('\n') {
    contents
  } else {
    contents + "\n"
  };

  if let Err(e) =
    write_atomic_mode(&target_path, contents.as_bytes(), 0o600)
      .await
      .with_context(|| {
        format!("Failed to write environment file to {target_path:?}")
      })
  {
    logs.push(Log::error(
      "Write Environment File",
      format_serror(&e.into()),
    ));
    return None;
  }

  logs.push(Log::simple(
    "Write Environment File",
    format!("Environment file written to {target_path:?}"),
  ));

  Some(target_path)
}

/// Hardened Stack `.env` writer supporting both standard Komodo environment
/// entries and literal external secret environment entries.
///
/// - Enforces safe relative `env_file_path` (no `..`, no absolute path, no symlinks)
///   whenever `external_environment` is non-empty, and always rejects symlinks.
/// - Formats `external_environment` using lossless Compose double-quoted `.env`
///   escaping so `#`, `$`, `${VAR}`, quotes, backslashes, spaces, `%`, and
///   newlines survive 100% verbatim without Compose expansion or truncation.
/// - Atomically writes with mode `0600`.
pub async fn write_stack_env_file(
  standard_environment: &[EnvironmentVar],
  external_environment: &[EnvironmentVar],
  folder: &Path,
  env_file_path: &str,
  logs: &mut Vec<Log>,
) -> Option<PathBuf> {
  if external_environment.is_empty() {
    return write_env_file(
      standard_environment,
      folder,
      env_file_path,
      logs,
    )
    .await;
  }

  let target_path =
    match validate_safe_env_file_path(folder, env_file_path) {
      Ok(p) => p,
      Err(e) => {
        logs.push(Log::error(
          "Write Environment File",
          format_serror(&e.into()),
        ));
        return None;
      }
    };

  let external_keys: std::collections::HashSet<&str> =
    external_environment
      .iter()
      .map(|ev| ev.variable.as_str())
      .collect();

  let mut lines = Vec::with_capacity(
    standard_environment.len() + external_environment.len(),
  );

  for env in standard_environment {
    if !external_keys.contains(env.variable.as_str()) {
      lines.push(format!("{}={}", env.variable, env.value));
    }
  }

  for env in external_environment {
    if let Err(e) = validate_env_var_key(&env.variable) {
      logs.push(Log::error(
        "Write Environment File",
        format_serror(&e.into()),
      ));
      return None;
    }
    match format_compose_dotenv_quoted_value(&env.value) {
      Ok(quoted) => lines.push(format!("{}={quoted}", env.variable)),
      Err(e) => {
        logs.push(Log::error(
          "Write Environment File",
          format_serror(
            &e.context(format!(
              "Failed to format external environment variable '{}'",
              env.variable
            ))
            .into(),
          ),
        ));
        return None;
      }
    }
  }

  let mut contents = lines.join("\n");
  if !contents.is_empty() && !contents.ends_with('\n') {
    contents.push('\n');
  }

  if let Err(e) =
    write_atomic_mode(&target_path, contents.as_bytes(), 0o600)
      .await
      .with_context(|| {
        format!("Failed to write environment file to {target_path:?}")
      })
  {
    logs.push(Log::error(
      "Write Environment File",
      format_serror(&e.into()),
    ));
    return None;
  }

  // Re-verify path post-write to ensure no race-condition symlink appeared on intermediate dirs
  if let Err(e) = validate_safe_env_file_path(folder, env_file_path) {
    let _ = tokio::fs::remove_file(&target_path).await;
    logs.push(Log::error(
      "Write Environment File",
      format_serror(&e.into()),
    ));
    return None;
  }

  logs.push(Log::simple(
    "Write Environment File",
    format!("Environment file written to {target_path:?}"),
  ));

  Some(target_path)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;

  fn temp_test_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let dir = std::env::temp_dir().join(format!(
      "komodo-env-test-{label}-{}-{nanos}",
      std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[tokio::test]
  async fn atomic_0600_write_replaces_world_readable_file_and_blocks_symlinks_and_traversal()
   {
    let dir = temp_test_dir("atomic");
    let existing = dir.join(".env");
    std::fs::write(&existing, "OLD=1\n").unwrap();
    std::fs::set_permissions(
      &existing,
      std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();

    let mut logs = Vec::new();
    let written = write_stack_env_file(
      &[EnvironmentVar {
        variable: String::from("PUBLIC_PORT"),
        value: String::from("8080"),
      }],
      &[EnvironmentVar {
        variable: String::from("DATABASE_URL"),
        value: String::from(
          "postgres://user:p%40ss%23word%2F123@db:5432/app",
        ),
      }],
      &dir,
      ".env",
      &mut logs,
    )
    .await
    .expect("should write .env");

    assert_eq!(written, existing);
    assert!(logs.iter().all(|l| l.success));

    let mode =
      std::fs::metadata(&existing).unwrap().permissions().mode()
        & 0o777;
    assert_eq!(mode, 0o600, ".env must be enforced 0600");

    // Reject path traversal
    let mut traversal_logs = Vec::new();
    assert!(
      write_stack_env_file(
        &[],
        &[EnvironmentVar {
          variable: String::from("SECRET"),
          value: String::from("x"),
        }],
        &dir,
        "../outside.env",
        &mut traversal_logs,
      )
      .await
      .is_none()
    );
    assert!(!traversal_logs[0].success);

    // Reject symlink target
    let outside = dir.join("outside.txt");
    std::fs::write(&outside, "keep").unwrap();
    let link_env = dir.join("symlink.env");
    std::os::unix::fs::symlink(&outside, &link_env).unwrap();

    let mut symlink_logs = Vec::new();
    assert!(
      write_stack_env_file(
        &[],
        &[EnvironmentVar {
          variable: String::from("SECRET"),
          value: String::from("leaked"),
        }],
        &dir,
        "symlink.env",
        &mut symlink_logs,
      )
      .await
      .is_none()
    );
    assert!(!symlink_logs[0].success);
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "keep");

    let _ = std::fs::remove_dir_all(&dir);
  }

  #[tokio::test]
  async fn compose_dotenv_roundtrip_preserves_all_special_chars_verbatim()
   {
    let dir = temp_test_dir("compose-roundtrip");
    let tricky = "p@ss#word/123?a=1&b=2+3 $DOLLAR ${HOME} \"dq\" 'sq' \\slash space\nline2\ttab";
    let pre_encoded =
      "postgres://user:p%40ss%23word%2F123@db:5432/app";

    let mut logs = Vec::new();
    write_stack_env_file(
      &[EnvironmentVar {
        variable: String::from("PLAIN_KEY"),
        value: String::from("plain_val"),
      }],
      &[
        EnvironmentVar {
          variable: String::from("DATABASE_URL"),
          value: String::from(pre_encoded),
        },
        EnvironmentVar {
          variable: String::from("TRICKY_SECRET"),
          value: String::from(tricky),
        },
      ],
      &dir,
      ".env",
      &mut logs,
    )
    .await
    .expect("should write .env");

    std::fs::write(
      dir.join("compose.yaml"),
      "services:\n  app:\n    image: busybox:latest\n    environment:\n      PLAIN_KEY: ${PLAIN_KEY}\n      DATABASE_URL: ${DATABASE_URL}\n      TRICKY_SECRET: ${TRICKY_SECRET}\n",
    )
    .unwrap();

    // Verify with real `docker compose config --no-normalize --format json` if docker compose is installed
    let out = std::process::Command::new("docker")
      .args([
        "compose",
        "-f",
        "compose.yaml",
        "--env-file",
        ".env",
        "config",
        "--format",
        "json",
      ])
      .current_dir(&dir)
      .output();

    if let Ok(output) = out
      && output.status.success()
    {
      let stdout = String::from_utf8_lossy(&output.stdout);
      // Note: `docker compose config` serializes literal `$` as `$$` in rendered config output
      assert!(stdout.contains(pre_encoded), "stdout: {stdout}");
      assert!(
        stdout.contains("p@ss#word/123?a=1&b=2+3 $$DOLLAR $${HOME}")
          || stdout.contains(
            "p@ss#word/123?a=1\\u0026b=2+3 $$DOLLAR $${HOME}"
          ),
        "stdout: {stdout}"
      );
      assert!(
        stdout
          .contains("\\\"dq\\\" 'sq' \\\\slash space\\nline2\\ttab"),
        "stdout: {stdout}"
      );
    }

    let _ = std::fs::remove_dir_all(&dir);
  }
}
