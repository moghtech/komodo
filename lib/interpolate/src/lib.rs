use std::collections::{HashMap, HashSet};

use anyhow::Context;
use komodo_client::entities::{
  EnvironmentVar, build::Build, deployment::Deployment, repo::Repo,
  stack::Stack, update::Log,
};

pub struct Interpolator<'a> {
  variables: Option<&'a HashMap<String, String>>,
  secrets: &'a HashMap<String, String>,
  variable_replacers: HashSet<(String, String)>,
  pub secret_replacers: HashSet<(String, String)>,
  /// Secrets that were referenced but carry an empty value: they interpolate
  /// to nothing and are excluded from the replacers, so operators still see
  /// why a value came out blank (#1429).
  pub empty_secrets: HashSet<String>,
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
      empty_secrets: Default::default(),
    }
  }

  pub fn interpolate_stack(
    &mut self,
    stack: &mut Stack,
  ) -> anyhow::Result<&mut Self> {
    if stack.config.skip_secret_interp {
      return Ok(self);
    }
    self
      .interpolate_string(&mut stack.config.file_contents)?
      .interpolate_string(&mut stack.config.environment)?
      .interpolate_string(&mut stack.config.pre_deploy.command)?
      .interpolate_string(&mut stack.config.post_deploy.command)?
      .interpolate_string(&mut stack.config.compose_cmd_wrapper)?
      .interpolate_extra_args(&mut stack.config.extra_args)?
      .interpolate_extra_args(&mut stack.config.build_extra_args)
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
      self.variable_replacers.extend(
        // An empty value as a replace search inserts the name at every
        // character boundary, so the sanitized logs balloon and corrupt
        // instead of masking (komodo#1429). Empty values interpolate to
        // nothing but never sanitize.
        more_replacers
          .into_iter()
          .filter(|(value, _)| !value.is_empty()),
      );
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
    for (value, variable) in more_replacers {
      // Same empty-search corruption as the variables above; an empty
      // secret interpolates to nothing, and the operator still learns
      // which ones were empty from the interpolated-secrets log.
      if value.is_empty() {
        self.empty_secrets.insert(variable);
      } else {
        self.secret_replacers.insert((value, variable));
      }
    }

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

    if !self.empty_secrets.is_empty() {
      logs.push(Log::simple(
        "Interpolate Secrets",
        format!(
          "<span class=\"text-muted-foreground\">empty value, interpolated to nothing:</span> {}",
          {
            let mut names: Vec<String> =
              self.empty_secrets.iter().cloned().collect();
            names.sort();
            names.join(", ")
          }
        ),
      ));
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  fn secrets(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect()
  }

  #[test]
  fn an_empty_secret_interpolates_to_nothing_and_never_sanitizes() {
    let map =
      secrets(&[("EMPTY_SECRET", ""), ("REAL_SECRET", "hunter2")]);
    let mut interpolator = Interpolator::new(None, &map);
    let mut target =
      "pass [[EMPTY_SECRET]] then [[REAL_SECRET]] end".to_string();
    interpolator
      .interpolate_string(&mut target)
      .expect("interpolates");
    assert_eq!(target, "pass  then hunter2 end");
    // An empty search string in `replace` inserts at every character
    // boundary, so the empty pair must stay out of the replacers.
    assert_eq!(
      interpolator.secret_replacers,
      HashSet::from([(
        "hunter2".to_string(),
        "REAL_SECRET".to_string()
      )])
    );
    assert_eq!(
      interpolator.empty_secrets,
      HashSet::from(["EMPTY_SECRET".to_string()])
    );
    // The whole point of #1429: sanitizing with these replacers cannot grow
    // the log.
    let replacers: Vec<(String, String)> =
      interpolator.secret_replacers.iter().cloned().collect();
    let sanitized =
      svi::replace_in_string("a [[EMPTY_SECRET]] b", &replacers);
    assert_eq!(sanitized, "a [[EMPTY_SECRET]] b");
  }

  #[test]
  fn an_empty_variable_interpolates_and_is_not_sanitized() {
    let map = secrets(&[("EMPTY_VAR", "")]);
    let no_secrets = HashMap::new();
    let mut interpolator = Interpolator::new(Some(&map), &no_secrets);
    let mut target = "x=[[EMPTY_VAR]]y".to_string();
    interpolator
      .interpolate_string(&mut target)
      .expect("interpolates");
    assert_eq!(target, "x=y");
    assert!(interpolator.variable_replacers.is_empty());
  }

  #[test]
  fn a_real_secret_still_masks_in_sanitized_logs() {
    let map = secrets(&[("REAL_SECRET", "hunter2")]);
    let mut interpolator = Interpolator::new(None, &map);
    let mut target = "token=[[REAL_SECRET]]".to_string();
    interpolator
      .interpolate_string(&mut target)
      .expect("interpolates");
    let replacers: Vec<(String, String)> =
      interpolator.secret_replacers.iter().cloned().collect();
    let sanitized =
      svi::replace_in_string("echo token=hunter2", &replacers);
    assert_eq!(sanitized, "echo token=<REAL_SECRET>");
  }
}
