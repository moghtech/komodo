//! Extra headers for the outbound websocket request to Core,
//! read from `core_headers_file`. Header values are secrets
//! (eg. Cloudflare Access service tokens), so they are marked
//! sensitive and never included in errors or logs.

use std::{
  fs::File,
  io::Read as _,
  os::unix::fs::{MetadataExt as _, PermissionsExt as _},
  path::Path,
};

use anyhow::{Context as _, anyhow, bail};
use axum::http::{HeaderMap, HeaderName, HeaderValue};

use crate::config::periphery_config;

/// Headers the websocket handshake manages itself.
const RESERVED_HEADERS: &[&str] = &[
  "host",
  "connection",
  "upgrade",
  "content-length",
  "transfer-encoding",
];

/// Reads the configured headers file, or returns no headers
/// when `core_headers_file` is not set.
pub fn core_headers() -> anyhow::Result<HeaderMap> {
  match &periphery_config().core_headers_file {
    Some(path) => read_headers_file(path, current_uid()),
    None => Ok(HeaderMap::new()),
  }
}

fn current_uid() -> u32 {
  // SAFETY: geteuid has no preconditions and cannot fail.
  unsafe { libc::geteuid() }
}

fn read_headers_file(
  path: &Path,
  expected_owner: u32,
) -> anyhow::Result<HeaderMap> {
  let context =
    || format!("Invalid core headers file {}", path.display());
  let mut file = File::open(path)
    .with_context(|| format!("Failed to open {}", path.display()))
    .with_context(context)?;
  let metadata = file
    .metadata()
    .context("Failed to read file metadata")
    .with_context(context)?;
  if !metadata.is_file() {
    return Err(anyhow!("Not a regular file")).with_context(context);
  }
  if metadata.uid() != expected_owner {
    return Err(anyhow!(
      "File is owned by uid {}, but Periphery runs as uid {expected_owner}",
      metadata.uid()
    ))
    .with_context(context);
  }
  let mode = metadata.permissions().mode() & 0o777;
  if mode & 0o077 != 0 {
    return Err(anyhow!(
      "File mode is {mode:o}, it must not be accessible by group or others (chmod 600)"
    ))
    .with_context(context);
  }
  let mut contents = String::new();
  file
    .read_to_string(&mut contents)
    .context("Failed to read file")
    .with_context(context)?;
  parse_headers(&contents).with_context(context)
}

fn parse_headers(contents: &str) -> anyhow::Result<HeaderMap> {
  let mut headers = HeaderMap::new();
  for (index, line) in contents.lines().enumerate() {
    let line_number = index + 1;
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    // Errors below never include the value.
    let Some((name, value)) = line.split_once(':') else {
      bail!("Line {line_number}: expected 'Name: value'");
    };
    let name = HeaderName::from_bytes(name.trim().as_bytes())
      .with_context(|| {
        format!("Line {line_number}: invalid header name")
      })?;
    if RESERVED_HEADERS.contains(&name.as_str())
      || name.as_str().starts_with("sec-websocket-")
    {
      bail!(
        "Line {line_number}: header '{name}' is set by the websocket handshake"
      );
    }
    let value = value.trim();
    if value.is_empty() {
      bail!("Line {line_number}: header '{name}' has an empty value");
    }
    let mut value = HeaderValue::from_str(value).map_err(|_| {
      anyhow!(
        "Line {line_number}: header '{name}' has an invalid value"
      )
    })?;
    value.set_sensitive(true);
    headers.append(name, value);
  }
  if headers.is_empty() {
    bail!("File contains no headers");
  }
  Ok(headers)
}

#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;

  const SECRET: &str = "super-secret-value";

  fn write_file(
    mode: u32,
    contents: &str,
  ) -> tempfile_path::TempPath {
    let path = tempfile_path::TempPath::new();
    fs::write(&path.0, contents).unwrap();
    fs::set_permissions(&path.0, fs::Permissions::from_mode(mode))
      .unwrap();
    path
  }

  #[test]
  fn parses_headers_ignoring_blank_lines_and_comments() {
    let headers = parse_headers(&format!(
      "# Cloudflare Access\n\nCF-Access-Client-Id: abc.access\n  CF-Access-Client-Secret : {SECRET}  \n"
    ))
    .unwrap();
    assert_eq!(headers.len(), 2);
    assert_eq!(headers["cf-access-client-id"], "abc.access");
    assert_eq!(headers["cf-access-client-secret"], SECRET);
    assert!(headers["cf-access-client-secret"].is_sensitive());
  }

  #[test]
  fn keeps_colons_in_values() {
    let headers = parse_headers("X-Token: a:b:c").unwrap();
    assert_eq!(headers["x-token"], "a:b:c");
  }

  #[test]
  fn rejects_invalid_lines_without_leaking_values() {
    for contents in [
      SECRET.to_string(),
      format!("Bad Name: {SECRET}"),
      format!("X-Token: {SECRET}\u{7f}"),
      format!("Host: {SECRET}"),
      format!("Sec-WebSocket-Key: {SECRET}"),
      "X-Token:".to_string(),
      "# only a comment".to_string(),
    ] {
      let error =
        format!("{:#}", parse_headers(&contents).unwrap_err());
      assert!(!error.contains(SECRET), "{error}");
    }
  }

  #[test]
  fn debug_output_redacts_values() {
    let headers =
      parse_headers(&format!("X-Token: {SECRET}")).unwrap();
    assert!(!format!("{headers:?}").contains(SECRET));
  }

  #[test]
  fn reads_owner_only_file() {
    let file = write_file(0o600, &format!("X-Token: {SECRET}"));
    let headers = read_headers_file(&file.0, current_uid()).unwrap();
    assert_eq!(headers["x-token"], SECRET);
    let file = write_file(0o400, &format!("X-Token: {SECRET}"));
    assert!(read_headers_file(&file.0, current_uid()).is_ok());
  }

  #[test]
  fn rejects_group_or_other_access() {
    for mode in [0o640, 0o604, 0o620, 0o602, 0o644, 0o660] {
      let file = write_file(mode, &format!("X-Token: {SECRET}"));
      let error = format!(
        "{:#}",
        read_headers_file(&file.0, current_uid()).unwrap_err()
      );
      assert!(error.contains("chmod 600"), "{mode:o}: {error}");
      assert!(!error.contains(SECRET));
    }
  }

  #[test]
  fn rejects_file_owned_by_another_user() {
    let file = write_file(0o600, &format!("X-Token: {SECRET}"));
    let error = format!(
      "{:#}",
      read_headers_file(&file.0, current_uid() + 1).unwrap_err()
    );
    assert!(error.contains("owned by uid"), "{error}");
  }

  #[test]
  fn rejects_missing_file() {
    let file = tempfile_path::TempPath::new();
    assert!(read_headers_file(&file.0, current_uid()).is_err());
  }

  /// Minimal temp file path that is removed on drop.
  mod tempfile_path {
    use std::{
      path::PathBuf,
      sync::atomic::{AtomicUsize, Ordering},
    };

    pub struct TempPath(pub PathBuf);

    impl TempPath {
      pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        TempPath(std::env::temp_dir().join(format!(
          "periphery-core-headers-{}-{}",
          std::process::id(),
          COUNTER.fetch_add(1, Ordering::Relaxed)
        )))
      }
    }

    impl Drop for TempPath {
      fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
      }
    }
  }
}
