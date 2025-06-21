use anyhow::Context;
use komodo_client::{
  entities::{EnvironmentVar, RepoExecutionArgs, SearchCombinator},
  parsers::QUOTE_PATTERN,
};

use crate::config::periphery_config;

pub fn git_token_simple(
  domain: &str,
  account_username: &str,
) -> anyhow::Result<&'static str> {
  periphery_config()
    .git_providers
    .iter()
    .find(|provider| provider.domain == domain)
    .and_then(|provider| {
      provider.accounts.iter().find(|account| account.username == account_username).map(|account| account.token.as_str())
    })
    .with_context(|| format!("Did not find token in config for git account {account_username} | domain {domain}"))
}

pub fn git_token(
  core_token: Option<String>,
  args: &RepoExecutionArgs,
) -> anyhow::Result<Option<String>> {
  if core_token.is_some() {
    return Ok(core_token);
  }
  let Some(account) = &args.account else {
    return Ok(None);
  };
  let token = git_token_simple(&args.provider, account)?;
  Ok(Some(token.to_string()))
}

pub fn registry_token(
  domain: &str,
  account_username: &str,
) -> anyhow::Result<&'static str> {
  periphery_config()
    .docker_registries
    .iter()
    .find(|registry| registry.domain == domain)
    .and_then(|registry| {
      registry.accounts.iter().find(|account| account.username == account_username).map(|account| account.token.as_str())
    })
    .with_context(|| format!("did not find token in config for docker registry account {account_username} | domain {domain}"))
}

pub fn parse_extra_args(extra_args: &[String]) -> String {
  let args = extra_args.join(" ");
  if !args.is_empty() {
    format!(" {args}")
  } else {
    args
  }
}

pub fn parse_labels(labels: &[EnvironmentVar]) -> String {
  labels
    .iter()
    .map(|p| {
      if p.value.starts_with(QUOTE_PATTERN)
        && p.value.ends_with(QUOTE_PATTERN)
      {
        // If the value already wrapped in quotes, don't wrap it again
        format!(" --label {}={}", p.variable, p.value)
      } else {
        format!(" --label {}=\"{}\"", p.variable, p.value)
      }
    })
    .collect::<Vec<_>>()
    .join("")
}

pub fn log_grep(
  terms: &[String],
  combinator: SearchCombinator,
  invert: bool,
) -> String {
  let maybe_invert = invert.then_some(" -v").unwrap_or_default();
  match combinator {
    SearchCombinator::Or => {
      format!("grep{maybe_invert} -E '{}'", terms.join("|"))
    }
    SearchCombinator::And => {
      format!(
        "grep{maybe_invert} -P '^(?=.*{})'",
        terms.join(")(?=.*")
      )
    }
  }
}
