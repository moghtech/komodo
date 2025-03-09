use std::collections::HashSet;

use anyhow::Context;
use komodo_client::entities::{SystemCommand, update::Update};

use super::query::VariablesAndSecrets;

pub fn interpolate_variables_secrets_into_extra_args(
  VariablesAndSecrets { variables, secrets }: &VariablesAndSecrets,
  extra_args: &mut Vec<String>,
  global_replacers: &mut HashSet<(String, String)>,
  secret_replacers: &mut HashSet<(String, String)>,
) -> anyhow::Result<()> {
  for arg in extra_args {
    if arg.is_empty() {
      continue;
    }

    // first pass - global variables
    let (res, more_replacers) = svi::interpolate_variables(
      arg,
      variables,
      svi::Interpolator::DoubleBrackets,
      false,
    )
    .with_context(|| {
      format!(
        "failed to interpolate global variables into extra arg '{arg}'",
      )
    })?;
    global_replacers.extend(more_replacers);

    // second pass - core secrets
    let (res, more_replacers) = svi::interpolate_variables(
      &res,
      secrets,
      svi::Interpolator::DoubleBrackets,
      false,
    )
    .with_context(|| {
      format!(
        "failed to interpolate core secrets into extra arg '{arg}'",
      )
    })?;
    secret_replacers.extend(more_replacers);

    // set arg with the result
    *arg = res;
  }

  Ok(())
}

pub fn interpolate_variables_secrets_into_string(
  VariablesAndSecrets { variables, secrets }: &VariablesAndSecrets,
  target: &mut String,
  global_replacers: &mut HashSet<(String, String)>,
  secret_replacers: &mut HashSet<(String, String)>,
) -> anyhow::Result<()> {
  if target.is_empty() {
    return Ok(());
  }

  // first pass - global variables
  let (res, more_replacers) = svi::interpolate_variables(
    target,
    variables,
    svi::Interpolator::DoubleBrackets,
    false,
  )
  .context("Failed to interpolate core variables")?;
  global_replacers.extend(more_replacers);

  // second pass - core secrets
  let (res, more_replacers) = svi::interpolate_variables(
    &res,
    secrets,
    svi::Interpolator::DoubleBrackets,
    false,
  )
  .context("Failed to interpolate core secrets")?;
  secret_replacers.extend(more_replacers);

  // set command with the result
  *target = res;

  Ok(())
}

pub fn interpolate_variables_secrets_into_system_command(
  VariablesAndSecrets { variables, secrets }: &VariablesAndSecrets,
  command: &mut SystemCommand,
  global_replacers: &mut HashSet<(String, String)>,
  secret_replacers: &mut HashSet<(String, String)>,
) -> anyhow::Result<()> {
  if command.command.is_empty() {
    return Ok(());
  }

  // first pass - global variables
  let (res, more_replacers) = svi::interpolate_variables(
    &command.command,
    variables,
    svi::Interpolator::DoubleBrackets,
    false,
  )
  .with_context(|| {
    format!(
      "failed to interpolate global variables into command '{}'",
      command.command
    )
  })?;
  global_replacers.extend(more_replacers);

  // second pass - core secrets
  let (res, more_replacers) = svi::interpolate_variables(
    &res,
    secrets,
    svi::Interpolator::DoubleBrackets,
    false,
  )
  .with_context(|| {
    format!(
      "failed to interpolate core secrets into command '{}'",
      command.command
    )
  })?;
  secret_replacers.extend(more_replacers);

  // set command with the result
  command.command = res;

  Ok(())
}

pub fn add_interp_update_log(
  update: &mut Update,
  global_replacers: &HashSet<(String, String)>,
  secret_replacers: &HashSet<(String, String)>,
) {
  // Show which variables were interpolated
  if !global_replacers.is_empty() {
    update.push_simple_log(
      "interpolate global variables",
      global_replacers
        .iter()
        .map(|(value, variable)| format!("<span class=\"text-muted-foreground\">{variable} =></span> {value}"))
        .collect::<Vec<_>>()
        .join("\n"),
    );
  }

  // Only show names of interpolated secrets
  if !secret_replacers.is_empty() {
    update.push_simple_log(
      "interpolate core secrets",
      secret_replacers
        .iter()
        .map(|(_, variable)| format!("<span class=\"text-muted-foreground\">replaced:</span> {variable}"))
        .collect::<Vec<_>>()
        .join("\n"),
    );
  }
}
