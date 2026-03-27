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
