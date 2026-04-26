use std::collections::HashMap;
use axum::http::HeaderMap;
use anyhow::anyhow;

use crate::api::listener::{ExtractBranch, VerifySecret};

pub struct QueryAuth;

impl VerifySecret for QueryAuth {
  fn verify_secret(
    query: &HashMap<String, String>,
    _headers: &HeaderMap,
    _body: &str,
    custom_secret: &str,
  ) -> anyhow::Result<()> {
    if query.get("secret").map(|s| s.as_str()) == Some(custom_secret) {
      Ok(())
    } else {
      Err(anyhow!("Invalid or missing 'secret' query parameter"))
    }
  }
}

impl ExtractBranch for QueryAuth {
  fn extract_branch(query: &std::collections::HashMap<String, String>, _body: &str) -> anyhow::Result<String> {
    if let Some(branch) = query.get("branch") {
      Ok(branch.to_string())
    } else {
      Ok("main".to_string())
    }
  }
}
