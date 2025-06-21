use std::path::PathBuf;

use command::run_komodo_command_with_sanitization;
use environment::write_env_file;
use interpolate::Interpolator;
use komodo_client::entities::{
  EnvironmentVar, RepoExecutionResponse, SystemCommand,
  all_logs_success,
};
use periphery_client::api::git::PeripheryRepoExecutionResponse;

use crate::config::periphery_config;

pub async fn handle_post_repo_execution(
  mut res: RepoExecutionResponse,
  mut environment: Vec<EnvironmentVar>,
  env_file_path: &str,
  mut on_clone: Option<SystemCommand>,
  mut on_pull: Option<SystemCommand>,
  skip_secret_interp: bool,
  mut replacers: Vec<(String, String)>,
) -> anyhow::Result<PeripheryRepoExecutionResponse> {
  if !skip_secret_interp {
    let mut interpolotor =
      Interpolator::new(None, &periphery_config().secrets);
    interpolotor.interpolate_env_vars(&mut environment)?;
    if let Some(on_clone) = on_clone.as_mut() {
      interpolotor.interpolate_string(&mut on_clone.command)?;
    }
    if let Some(on_pull) = on_pull.as_mut() {
      interpolotor.interpolate_string(&mut on_pull.command)?;
    }
    replacers.extend(interpolotor.secret_replacers);
  }

  let env_file_path = write_env_file(
    &environment,
    &res.path,
    env_file_path,
    &mut res.logs,
  )
  .await;

  let mut res = PeripheryRepoExecutionResponse { res, env_file_path };

  if let Some(on_clone) = on_clone {
    if !on_clone.is_none() {
      let path = res
        .res
        .path
        .join(on_clone.path)
        .components()
        .collect::<PathBuf>();
      if let Some(log) = run_komodo_command_with_sanitization(
        "On Clone",
        path.as_path(),
        on_clone.command,
        true,
        &replacers,
      )
      .await
      {
        res.res.logs.push(log);
        if !all_logs_success(&res.res.logs) {
          return Ok(res);
        }
      }
    }
  }

  if let Some(on_pull) = on_pull {
    if !on_pull.is_none() {
      let path = res
        .res
        .path
        .join(on_pull.path)
        .components()
        .collect::<PathBuf>();
      if let Some(log) = run_komodo_command_with_sanitization(
        "On Pull",
        path.as_path(),
        on_pull.command,
        true,
        &replacers,
      )
      .await
      {
        res.res.logs.push(log);
      }
    }
  }

  Ok(res)
}
