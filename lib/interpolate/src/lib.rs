use std::collections::{HashMap, HashSet};

use anyhow::Context;
use komodo_client::entities::{
  EnvironmentVar, build::Build, deployment::Deployment,
  environment_vars_from_str, repo::Repo, stack::Stack, update::Log,
};

/// The marker used when a secret value is removed from persisted output.
pub const REDACTED: &str = "<redacted>";

/// Returns replacement pairs for values in a Stack environment whose key is
/// conventionally sensitive. Stack environment entries do not carry the
/// `is_secret` metadata available to global Komodo Variables, so this is the
/// fail-safe seam for direct environment values.
pub fn stack_environment_secret_replacers(
  environment: &str,
) -> anyhow::Result<HashSet<(String, String)>> {
  let variables = environment_vars_from_str(environment)
    .context("failed to parse Stack environment for redaction")?;
  Ok(
    variables
      .into_iter()
      .filter(|variable| {
        !variable.value.is_empty()
          && sensitive_environment_key(&variable.variable)
      })
      .map(|variable| (variable.value, variable.variable))
      .collect(),
  )
}

fn sensitive_environment_key(key: &str) -> bool {
  let words = key
    .split(|character: char| !character.is_ascii_alphanumeric())
    .filter(|word| !word.is_empty())
    .map(str::to_ascii_uppercase)
    .collect::<Vec<_>>();

  words.iter().any(|word| {
    matches!(
      word.as_str(),
      "PASSWORD"
        | "PASSWD"
        | "SECRET"
        | "TOKEN"
        | "CREDENTIAL"
        | "CREDENTIALS"
        | "DSN"
    )
  }) || words.windows(2).any(|pair| {
    matches!(
      (pair[0].as_str(), pair[1].as_str()),
      ("API", "KEY")
        | ("PRIVATE", "KEY")
        | ("ACCESS", "KEY")
        | ("CONNECTION", "STRING")
    )
  })
}

pub struct Interpolator<'a> {
  variables: Option<&'a HashMap<String, String>>,
  secrets: &'a HashMap<String, String>,
  variable_replacers: HashSet<(String, String)>,
  pub secret_replacers: HashSet<(String, String)>,
}

impl<'a> Interpolator<'a> {
  pub fn new(
    variables: Option<&'a HashMap<String, String>>,
    secrets: &'a HashMap<String, String>,
  ) -> Interpolator<'a> {
    Interpolator {
      variables,
      secrets,
      variable_replacers: Default::default(),
      secret_replacers: Default::default(),
    }
  }

  pub fn interpolate_stack(
    &mut self,
    stack: &mut Stack,
  ) -> anyhow::Result<&mut Self> {
    if stack.config.skip_secret_interp {
      self.secret_replacers.extend(
        stack_environment_secret_replacers(
          &stack.config.environment,
        )?,
      );
      return Ok(self);
    }
    self
      .interpolate_string(&mut stack.config.file_contents)?
      .interpolate_string(&mut stack.config.environment)?
      .interpolate_string(&mut stack.config.pre_deploy.command)?
      .interpolate_string(&mut stack.config.post_deploy.command)?
      .interpolate_string(&mut stack.config.compose_cmd_wrapper)?
      .interpolate_extra_args(&mut stack.config.extra_args)?
      .interpolate_extra_args(&mut stack.config.build_extra_args)?;
    self
      .secret_replacers
      .extend(stack_environment_secret_replacers(
        &stack.config.environment,
      )?);
    Ok(self)
  }

  pub fn interpolate_repo(
    &mut self,
    repo: &mut Repo,
  ) -> anyhow::Result<&mut Self> {
    if repo.config.skip_secret_interp {
      return Ok(self);
    }
    self
      .interpolate_string(&mut repo.config.environment)?
      .interpolate_string(&mut repo.config.on_clone.command)?
      .interpolate_string(&mut repo.config.on_pull.command)
  }

  pub fn interpolate_build(
    &mut self,
    build: &mut Build,
  ) -> anyhow::Result<&mut Self> {
    if build.config.skip_secret_interp {
      return Ok(self);
    }
    self
      .interpolate_string(&mut build.config.build_args)?
      .interpolate_string(&mut build.config.secret_args)?
      .interpolate_string(&mut build.config.labels)?
      .interpolate_string(&mut build.config.pre_build.command)?
      .interpolate_string(&mut build.config.dockerfile)?
      .interpolate_extra_args(&mut build.config.extra_args)
  }

  pub fn interpolate_deployment(
    &mut self,
    deployment: &mut Deployment,
  ) -> anyhow::Result<&mut Self> {
    if deployment.config.skip_secret_interp {
      return Ok(self);
    }
    self
      .interpolate_string(&mut deployment.config.environment)?
      .interpolate_string(&mut deployment.config.ports)?
      .interpolate_string(&mut deployment.config.volumes)?
      .interpolate_string(&mut deployment.config.labels)?
      .interpolate_string(&mut deployment.config.command)?
      .interpolate_extra_args(&mut deployment.config.extra_args)
  }

  pub fn interpolate_string(
    &mut self,
    target: &mut String,
  ) -> anyhow::Result<&mut Self> {
    if target.is_empty() {
      return Ok(self);
    }

    // first pass - variables
    let res = if let Some(variables) = self.variables {
      let (res, more_replacers) = svi::interpolate_variables(
        target,
        variables,
        svi::Interpolator::DoubleBrackets,
        false,
      )
      .with_context(|| {
        format!(
          "failed to interpolate variables into target '{target}'",
        )
      })?;
      self.variable_replacers.extend(more_replacers);
      res
    } else {
      target.to_string()
    };

    // second pass - secrets
    let (res, more_replacers) = svi::interpolate_variables(
      &res,
      self.secrets,
      svi::Interpolator::DoubleBrackets,
      false,
    )
    .with_context(|| {
      format!("failed to interpolate secrets into target '{target}'",)
    })?;
    self.secret_replacers.extend(more_replacers);

    // Set with result
    *target = res;

    Ok(self)
  }

  pub fn interpolate_extra_args(
    &mut self,
    extra_args: &mut Vec<String>,
  ) -> anyhow::Result<&mut Self> {
    for arg in extra_args {
      self
        .interpolate_string(arg)
        .context("failed interpolation into extra arg")?;
    }
    Ok(self)
  }

  pub fn interpolate_env_vars(
    &mut self,
    env_vars: &mut Vec<EnvironmentVar>,
  ) -> anyhow::Result<&mut Self> {
    for var in env_vars {
      self
        .interpolate_string(&mut var.value)
        .context("failed interpolation into variable value")?;
    }
    Ok(self)
  }

  pub fn push_logs(&self, logs: &mut Vec<Log>) {
    // Show which variables / values were interpolated
    if !self.variable_replacers.is_empty() {
      logs.push(Log::simple("Interpolate Variables", self.variable_replacers
        .iter()
        .map(|(value, variable)| format!("<span class=\"text-muted-foreground\">{variable} =></span> {value}"))
        .collect::<Vec<_>>()
        .join("\n")));
    }

    // Only show names of interpolated secrets
    if !self.secret_replacers.is_empty() {
      logs.push(
        Log::simple("Interpolate Secrets",
        self.secret_replacers
          .iter()
          .map(|(_, variable)| format!("<span class=\"text-muted-foreground\">replaced:</span> {variable}"))
          .collect::<Vec<_>>()
          .join("\n"),)
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SYNTHETIC_SECRET: &str =
    "komodo-redaction-marker-4d9c1d48f7b84a9e";

  #[test]
  fn stack_environment_collects_only_sensitive_values() {
    let replacers = stack_environment_secret_replacers(&format!(
      "PUBLIC_URL=https://example.com\nAPI_TOKEN={SYNTHETIC_SECRET}\nDATABASE_CONNECTION_STRING=postgres://user:{SYNTHETIC_SECRET}@db/app\nEMPTY_SECRET=\n"
    ))
    .unwrap();

    assert!(replacers.contains(&(
      SYNTHETIC_SECRET.to_string(),
      "API_TOKEN".to_string()
    )));
    assert!(
      replacers
        .iter()
        .any(|(_, key)| { key == "DATABASE_CONNECTION_STRING" })
    );
    assert!(!replacers.iter().any(|(_, key)| key == "PUBLIC_URL"));
    assert!(!replacers.iter().any(|(value, _)| value.is_empty()));
    assert!(!sensitive_environment_key("WIKI_ACCESS_AUDIENCE"));
    assert!(sensitive_environment_key("SQL_DSN"));
    for key in [
      "STRIPE_API_SECRET",
      "STRIPE_WEBHOOK_SECRET",
      "SESSION_SECRET",
      "CRYPTO_SECRET",
      "POSTGRES_PASSWORD",
      "REDIS_PASSWORD",
      "LIVE_BRIDGE_INTERNAL_SECRET",
      "ROUTER_LIVE_SHARED_SECRET",
      "DARIO_API_KEY",
    ] {
      assert!(sensitive_environment_key(key), "{key}");
    }
  }

  #[test]
  fn stack_interpolation_redacts_synthetic_marker() {
    let mut stack = Stack::default();
    stack.config.environment =
      format!("ROUTER_SHARED_SECRET={SYNTHETIC_SECRET}");
    let variables = HashMap::new();
    let secrets = HashMap::new();
    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);

    interpolator.interpolate_stack(&mut stack).unwrap();
    let rendered = format!(
      "environment:\n  ROUTER_SHARED_SECRET: {SYNTHETIC_SECRET}"
    );
    let sanitized = svi::replace_in_string(
      &rendered,
      &interpolator.secret_replacers,
    );

    assert!(!sanitized.contains(SYNTHETIC_SECRET));
    assert!(sanitized.contains("<ROUTER_SHARED_SECRET>"));
  }

  #[test]
  fn stack_redaction_does_not_depend_on_interpolation() {
    let mut stack = Stack::default();
    stack.config.skip_secret_interp = true;
    stack.config.environment =
      format!("ROUTER_SHARED_SECRET={SYNTHETIC_SECRET}");
    let variables = HashMap::new();
    let secrets = HashMap::new();
    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);

    interpolator.interpolate_stack(&mut stack).unwrap();

    assert!(interpolator.secret_replacers.contains(&(
      SYNTHETIC_SECRET.to_string(),
      "ROUTER_SHARED_SECRET".to_string()
    )));
  }
}
