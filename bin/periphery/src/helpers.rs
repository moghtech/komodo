use std::path::PathBuf;

use anyhow::{Context, anyhow};
use komodo_client::{
  entities::{
    CloneArgs, EnvironmentVar, SearchCombinator, stack::Stack,
    to_komodo_name,
  },
  parsers::QUOTE_PATTERN,
};
use periphery_client::api::git::PullOrCloneRepo;
use resolver_api::Resolve;

use crate::config::periphery_config;

pub fn git_token(
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
    .with_context(|| format!("did not find token in config for git account {account_username} | domain {domain}"))
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

/// Returns path to root directory of the stack repo.
pub async fn pull_or_clone_stack(
  stack: &Stack,
  git_token: Option<String>,
) -> anyhow::Result<PathBuf> {
  if stack.config.files_on_host {
    return Err(anyhow!(
      "Wrong method called for files on host stack"
    ));
  }
  if stack.config.repo.is_empty() {
    return Err(anyhow!("Repo is not configured"));
  }

  let root = periphery_config()
    .stack_dir()
    .join(to_komodo_name(&stack.name));

  let mut args: CloneArgs = stack.into();
  // Set the clone destination to the one created for this run
  args.destination = Some(root.display().to_string());

  let git_token = match git_token {
    Some(token) => Some(token),
    None => {
      if !stack.config.git_account.is_empty() {
        match crate::helpers::git_token(
          &stack.config.git_provider,
          &stack.config.git_account,
        ) {
          Ok(token) => Some(token.to_string()),
          Err(e) => {
            return Err(
              e.context("Failed to find required git token"),
            );
          }
        }
      } else {
        None
      }
    }
  };

  PullOrCloneRepo {
    args,
    git_token,
    environment: vec![],
    env_file_path: stack.config.env_file_path.clone(),
    skip_secret_interp: stack.config.skip_secret_interp,
    // repo replacer only needed for on_clone / on_pull,
    // which aren't available for stacks
    replacers: Default::default(),
  }
  .resolve(&crate::api::Args)
  .await
  .map_err(|e| e.error)?;

  Ok(root)
}
