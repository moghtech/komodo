use std::{fmt::Write, path::PathBuf};

use anyhow::{Context, anyhow};
use command::run_komodo_command;
use formatting::format_serror;
use git::{GitRes, write_commit_file};
use komodo_client::entities::{
  FileContents, stack::ComposeProject, to_path_compatible_name,
  update::Log,
};
use periphery_client::api::{compose::*, git::RepoActionResponse};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
  compose::{WriteStackRes, compose_up, docker_compose, write_stack},
  config::periphery_config,
  docker::docker_login,
  helpers::{log_grep, pull_or_clone_stack},
};

impl Resolve<super::Args> for ListComposeProjects {
  #[instrument(name = "ComposeInfo", level = "debug", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<ComposeProject>> {
    let docker_compose = docker_compose();
    let res = run_komodo_command(
      "List Projects",
      None,
      format!("{docker_compose} ls --all --format json"),
    )
    .await;

    if !res.success {
      return Err(
        anyhow!("{}", res.combined())
          .context(format!(
            "failed to list compose projects using {docker_compose} ls"
          ))
          .into(),
      );
    }

    let res =
      serde_json::from_str::<Vec<DockerComposeLsItem>>(&res.stdout)
        .with_context(|| res.stdout.clone())
        .with_context(|| {
          format!(
            "failed to parse '{docker_compose} ls' response to json"
          )
        })?
        .into_iter()
        .filter(|item| !item.name.is_empty())
        .map(|item| ComposeProject {
          name: item.name,
          status: item.status,
          compose_files: item
            .config_files
            .split(',')
            .map(str::to_string)
            .collect(),
        })
        .collect();

    Ok(res)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeLsItem {
  #[serde(default, alias = "Name")]
  pub name: String,
  #[serde(alias = "Status")]
  pub status: Option<String>,
  /// Comma seperated list of paths
  #[serde(default, alias = "ConfigFiles")]
  pub config_files: String,
}

//

impl Resolve<super::Args> for GetComposeLog {
  #[instrument(name = "GetComposeLog", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let GetComposeLog {
      project,
      services,
      tail,
      timestamps,
    } = self;
    let docker_compose = docker_compose();
    let timestamps =
      timestamps.then_some(" --timestamps").unwrap_or_default();
    let command = format!(
      "{docker_compose} -p {project} logs --tail {tail}{timestamps} {}",
      services.join(" ")
    );
    Ok(run_komodo_command("get stack log", None, command).await)
  }
}

impl Resolve<super::Args> for GetComposeLogSearch {
  #[instrument(name = "GetComposeLogSearch", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let GetComposeLogSearch {
      project,
      services,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let docker_compose = docker_compose();
    let grep = log_grep(&terms, combinator, invert);
    let timestamps =
      timestamps.then_some(" --timestamps").unwrap_or_default();
    let command = format!(
      "{docker_compose} -p {project} logs --tail 5000{timestamps} {} 2>&1 | {grep}",
      services.join(" ")
    );
    Ok(run_komodo_command("Get stack log grep", None, command).await)
  }
}

//

impl Resolve<super::Args> for GetComposeContentsOnHost {
  #[instrument(name = "GetComposeContentsOnHost", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<GetComposeContentsOnHostResponse> {
    let GetComposeContentsOnHost {
      name,
      run_directory,
      file_paths,
    } = self;
    let root = periphery_config()
      .stack_dir()
      .join(to_path_compatible_name(&name));
    let run_directory =
      root.join(&run_directory).components().collect::<PathBuf>();

    if !run_directory.exists() {
      fs::create_dir_all(&run_directory)
        .await
        .context("Failed to initialize run directory")?;
    }

    let mut res = GetComposeContentsOnHostResponse::default();

    for path in file_paths {
      let full_path =
        run_directory.join(&path).components().collect::<PathBuf>();
      match fs::read_to_string(&full_path).await.with_context(|| {
        format!(
          "Failed to read compose file contents at {full_path:?}"
        )
      }) {
        Ok(contents) => {
          // The path we store here has to be the same as incoming file path in the array,
          // in order for WriteComposeContentsToHost to write to the correct path.
          res.contents.push(FileContents { path, contents });
        }
        Err(e) => {
          res.errors.push(FileContents {
            path,
            contents: format_serror(&e.into()),
          });
        }
      }
    }

    Ok(res)
  }
}

//

impl Resolve<super::Args> for WriteComposeContentsToHost {
  #[instrument(
    name = "WriteComposeContentsToHost",
    skip_all,
    fields(
      stack = &self.name,
      run_directory = &self.run_directory,
      file_path = &self.file_path,
    )
  )]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let WriteComposeContentsToHost {
      name,
      run_directory,
      file_path,
      contents,
    } = self;
    let file_path = periphery_config()
      .stack_dir()
      .join(to_path_compatible_name(&name))
      .join(&run_directory)
      .join(file_path)
      .components()
      .collect::<PathBuf>();
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
      fs::create_dir_all(&parent)
        .await
        .with_context(|| format!("Failed to initialize compose file parent directory {parent:?}"))?;
    }
    fs::write(&file_path, contents).await.with_context(|| {
      format!(
        "Failed to write compose file contents to {file_path:?}"
      )
    })?;
    Ok(Log::simple(
      "Write contents to host",
      format!("File contents written to {file_path:?}"),
    ))
  }
}

//

impl Resolve<super::Args> for WriteCommitComposeContents {
  #[instrument(
    name = "WriteCommitComposeContents",
    skip_all,
    fields(
      stack = &self.stack.name,
      username = &self.username,
      file_path = &self.file_path,
    )
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<RepoActionResponse> {
    let WriteCommitComposeContents {
      stack,
      username,
      file_path,
      contents,
      git_token,
    } = self;

    let root = pull_or_clone_stack(&stack, git_token).await?;

    let file_path = stack
      .config
      .run_directory
      .parse::<PathBuf>()
      .context("Run directory is not a valid path")?
      .join(&file_path);

    let msg = if let Some(username) = username {
      format!("{}: Write Compose File", username)
    } else {
      "Write Compose File".to_string()
    };

    let GitRes {
      logs,
      path,
      hash,
      message,
      ..
    } = write_commit_file(
      &msg,
      &root,
      &file_path,
      &contents,
      &stack.config.branch,
    )
    .await?;

    Ok(RepoActionResponse {
      logs,
      path,
      commit_hash: hash,
      commit_message: message,
      env_file_path: None,
    })
  }
}

//

impl WriteStackRes for &mut ComposePullResponse {
  fn logs(&mut self) -> &mut Vec<Log> {
    &mut self.logs
  }
}

impl Resolve<super::Args> for ComposePull {
  #[instrument(
    name = "ComposePull",
    skip_all,
    fields(
      stack = &self.stack.name,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<ComposePullResponse> {
    let ComposePull {
      stack,
      services,
      git_token,
      registry_token,
    } = self;
    let mut res = ComposePullResponse::default();

    let (run_directory, env_file_path, _replacers) =
      match write_stack(&stack, git_token, &mut res).await {
        Ok(res) => res,
        Err(e) => {
          res.logs.push(Log::error(
            "Write Stack",
            format_serror(&e.into()),
          ));
          return Ok(res);
        }
      };

    // Canonicalize the path to ensure it exists, and is the cleanest path to the run directory.
    let run_directory = run_directory.canonicalize().context(
      "Failed to validate run directory on host after stack write (canonicalize error)",
    )?;

    let file_paths = stack
      .file_paths()
      .iter()
      .map(|path| {
        (
          path,
          // This will remove any intermediate uneeded '/./' in the path
          run_directory.join(path).components().collect::<PathBuf>(),
        )
      })
      .collect::<Vec<_>>();

    for (path, full_path) in &file_paths {
      if !full_path.exists() {
        return Err(anyhow!("Missing compose file at {path}").into());
      }
    }

    let docker_compose = docker_compose();

    let service_args = if services.is_empty() {
      String::new()
    } else {
      format!(" {}", services.join(" "))
    };

    let file_args = if stack.config.file_paths.is_empty() {
      String::from("compose.yaml")
    } else {
      stack.config.file_paths.join(" -f ")
    };

    // Login to the registry to pull private images, if provider / account are set
    if !stack.config.registry_provider.is_empty()
      && !stack.config.registry_account.is_empty()
    {
      docker_login(
        &stack.config.registry_provider,
        &stack.config.registry_account,
        registry_token.as_deref(),
      )
      .await
      .with_context(|| {
        format!(
          "domain: {} | account: {}",
          stack.config.registry_provider,
          stack.config.registry_account
        )
      })
      .context("failed to login to image registry")?;
    }

    let env_file = env_file_path
      .map(|path| format!(" --env-file {path}"))
      .unwrap_or_default();

    let additional_env_files = stack
      .config
      .additional_env_files
      .iter()
      .fold(String::new(), |mut output, file| {
        let _ = write!(output, " --env-file {file}");
        output
      });

    let project_name = stack.project_name(false);

    let log = run_komodo_command(
      "Compose Pull",
      run_directory.as_ref(),
      format!(
        "{docker_compose} -p {project_name} -f {file_args}{additional_env_files}{env_file} pull{service_args}",
      ),
    )
    .await;

    res.logs.push(log);

    Ok(res)
  }
}

//

impl Resolve<super::Args> for ComposeUp {
  #[instrument(
    name = "ComposeUp",
    skip_all,
    fields(
      stack = &self.stack.name,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<ComposeUpResponse> {
    let ComposeUp {
      stack,
      services,
      git_token,
      registry_token,
      replacers,
    } = self;
    let mut res = ComposeUpResponse::default();
    if let Err(e) = compose_up(
      stack,
      services,
      git_token,
      registry_token,
      &mut res,
      replacers,
    )
    .await
    {
      res.logs.push(Log::error(
        "Compose Up - Failed",
        format_serror(&e.into()),
      ));
    };
    Ok(res)
  }
}

//

impl Resolve<super::Args> for ComposeExecution {
  #[instrument(name = "ComposeExecution")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let ComposeExecution { project, command } = self;
    let docker_compose = docker_compose();
    let log = run_komodo_command(
      "Compose Command",
      None,
      format!("{docker_compose} -p {project} {command}"),
    )
    .await;
    Ok(log)
  }
}
