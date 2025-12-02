use std::collections::HashMap;

use anyhow::Context;
use indexmap::IndexMap;
use komodo_client::{
  api::execute::Execution,
  entities::{
    action::Action,
    alerter::Alerter,
    build::Build,
    builder::{Builder, BuilderConfig, PartialBuilderConfig},
    deployment::{Deployment, DeploymentImage},
    procedure::Procedure,
    repo::Repo,
    resource::Resource,
    server::Server,
    stack::Stack,
    swarm::Swarm,
    sync::ResourceSync,
    tag::Tag,
    toml::ResourceToml,
  },
};
use partial_derive2::{MaybeNone, PartialDiff};

use crate::{resource::KomodoResource, state::all_resources_cache};

pub const TOML_PRETTY_OPTIONS: toml_pretty::Options =
  toml_pretty::Options {
    tab: "  ",
    skip_empty_string: true,
    // Usually we do this, but has to be changed for some cases.
    skip_empty_object: true,
    max_inline_array_length: 30,
    inline_array: false,
  };

pub trait ToToml: KomodoResource {
  /// Replace linked ids (server_id, build_id, etc) with the resource name.
  fn replace_ids(_resource: &mut Resource<Self::Config, Self::Info>) {
  }

  fn edit_config_object(
    _resource: &ResourceToml<Self::PartialConfig>,
    config: IndexMap<String, serde_json::Value>,
  ) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    Ok(config)
  }

  fn push_additional(
    _resource: ResourceToml<Self::PartialConfig>,
    _toml: &mut String,
  ) {
  }

  fn push_to_toml_string(
    mut resource: ResourceToml<Self::PartialConfig>,
    toml: &mut String,
  ) -> anyhow::Result<()> {
    resource.config =
      Self::Config::default().minimize_partial(resource.config);

    let mut resource_map: IndexMap<String, serde_json::Value> =
      serde_json::from_str(&serde_json::to_string(&resource)?)?;
    resource_map.shift_remove("config");

    let config = serde_json::from_str(&serde_json::to_string(
      &resource.config,
    )?)?;

    let config = Self::edit_config_object(&resource, config)?;

    toml.push_str(
      &toml_pretty::to_string(&resource_map, TOML_PRETTY_OPTIONS)
        .context("failed to serialize resource to toml")?,
    );

    toml.push_str(&format!(
      "\n[{}.config]\n",
      Self::resource_type().toml_header()
    ));

    toml.push_str(
      &toml_pretty::to_string(&config, TOML_PRETTY_OPTIONS)
        .context("failed to serialize resource config to toml")?,
    );

    Self::push_additional(resource, toml);

    Ok(())
  }
}

pub fn resource_toml_to_toml_string<R: ToToml>(
  resource: ResourceToml<R::PartialConfig>,
) -> anyhow::Result<String> {
  let mut toml = String::new();
  toml
    .push_str(&format!("[[{}]]\n", R::resource_type().toml_header()));
  R::push_to_toml_string(resource, &mut toml)?;
  Ok(toml)
}

pub fn resource_push_to_toml<R: ToToml>(
  mut resource: Resource<R::Config, R::Info>,
  deploy: bool,
  after: Vec<String>,
  toml: &mut String,
  all_tags: &HashMap<String, Tag>,
) -> anyhow::Result<()> {
  R::replace_ids(&mut resource);
  if !toml.is_empty() {
    toml.push_str("\n\n##\n\n");
  }
  toml
    .push_str(&format!("[[{}]]\n", R::resource_type().toml_header()));
  R::push_to_toml_string(
    convert_resource::<R>(resource, deploy, after, all_tags),
    toml,
  )?;
  Ok(())
}

pub fn resource_to_toml<R: ToToml>(
  resource: Resource<R::Config, R::Info>,
  deploy: bool,
  after: Vec<String>,
  all_tags: &HashMap<String, Tag>,
) -> anyhow::Result<String> {
  let mut toml = String::new();
  resource_push_to_toml::<R>(
    resource, deploy, after, &mut toml, all_tags,
  )?;
  Ok(toml)
}

pub fn convert_resource<R: KomodoResource>(
  resource: Resource<R::Config, R::Info>,
  deploy: bool,
  after: Vec<String>,
  all_tags: &HashMap<String, Tag>,
) -> ResourceToml<R::PartialConfig> {
  ResourceToml {
    name: resource.name,
    description: resource.description,
    template: resource.template,
    tags: resource
      .tags
      .iter()
      .filter_map(|t| all_tags.get(t).map(|t| t.name.clone()))
      .collect(),
    deploy,
    after,
    // The config still needs to be minimized.
    // This happens in ToToml::push_to_toml
    config: resource.config.into(),
  }
}

// These have no linked resource ids to replace
impl ToToml for Alerter {}
impl ToToml for Server {}
impl ToToml for Action {}

impl ToToml for ResourceSync {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    resource.config.linked_repo.clone_from(
      all
        .repos
        .get(&resource.config.linked_repo)
        .map(|r| &r.name)
        .unwrap_or(&String::new()),
    );
  }
}

impl ToToml for Swarm {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    let mut res =
      Vec::with_capacity(resource.config.server_ids.capacity());
    for server_id in &resource.config.server_ids {
      res.push(
        all
          .servers
          .get(server_id)
          .map(|s| s.name.clone())
          .unwrap_or_default(),
      );
    }
  }

  fn edit_config_object(
    _resource: &ResourceToml<Self::PartialConfig>,
    config: IndexMap<String, serde_json::Value>,
  ) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    config
      .into_iter()
      .map(|(key, value)| {
        #[allow(clippy::single_match)]
        match key.as_str() {
          "server_ids" => {
            return Ok((String::from("servers"), value));
          }
          _ => {}
        }
        Ok((key, value))
      })
      .collect()
  }
}

impl ToToml for Stack {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    resource.config.server_id.clone_from(
      all
        .servers
        .get(&resource.config.server_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
    resource.config.linked_repo.clone_from(
      all
        .repos
        .get(&resource.config.linked_repo)
        .map(|r| &r.name)
        .unwrap_or(&String::new()),
    );
  }

  fn edit_config_object(
    _resource: &ResourceToml<Self::PartialConfig>,
    config: IndexMap<String, serde_json::Value>,
  ) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    config
      .into_iter()
      .map(|(key, value)| {
        #[allow(clippy::single_match)]
        match key.as_str() {
          "server_id" => return Ok((String::from("server"), value)),
          _ => {}
        }
        Ok((key, value))
      })
      .collect()
  }
}

impl ToToml for Deployment {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    resource.config.server_id.clone_from(
      all
        .servers
        .get(&resource.config.server_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
    if let DeploymentImage::Build { build_id, .. } =
      &mut resource.config.image
    {
      build_id.clone_from(
        all
          .builds
          .get(build_id)
          .map(|b| &b.name)
          .unwrap_or(&String::new()),
      );
    }
  }

  fn edit_config_object(
    resource: &ResourceToml<Self::PartialConfig>,
    config: IndexMap<String, serde_json::Value>,
  ) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    config
      .into_iter()
      .map(|(key, mut value)| {
        match key.as_str() {
          "server_id" => return Ok((String::from("server"), value)),
          "image" => {
            if let Some(DeploymentImage::Build { version, .. }) =
              &resource.config.image
            {
              let image = value
                .get_mut("params")
                .context("deployment image has no params")?
                .as_object_mut()
                .context("deployment image params is not object")?;
              if let Some(build) = image.remove("build_id") {
                image.insert(String::from("build"), build);
              }
              if version.is_none() {
                image.remove("version");
              } else {
                image.insert(
                  "version".to_string(),
                  serde_json::Value::String(version.to_string()),
                );
              }
            }
          }
          _ => {}
        }
        Ok((key, value))
      })
      .collect()
  }
}

impl ToToml for Build {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    resource.config.builder_id.clone_from(
      all
        .builders
        .get(&resource.config.builder_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
    resource.config.linked_repo.clone_from(
      all
        .repos
        .get(&resource.config.linked_repo)
        .map(|r| &r.name)
        .unwrap_or(&String::new()),
    );
  }

  fn edit_config_object(
    resource: &ResourceToml<Self::PartialConfig>,
    config: IndexMap<String, serde_json::Value>,
  ) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    config
      .into_iter()
      .map(|(key, value)| match key.as_str() {
        "builder_id" => Ok((String::from("builder"), value)),
        "version" => {
          match (
            &resource.config.version,
            resource.config.auto_increment_version,
          ) {
            (None, _) => Ok((key, value)),
            (_, Some(true)) | (_, None) => {
              // The toml shouldn't have a version attached if auto incrementing.
              // Empty string will be filtered out in final toml.
              Ok((key, serde_json::Value::String(String::new())))
            }
            (Some(version), _) => Ok((
              key,
              serde_json::Value::String(version.to_string()),
            )),
          }
        }
        _ => Ok((key, value)),
      })
      .collect()
  }
}

impl ToToml for Repo {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    resource.config.server_id.clone_from(
      all
        .servers
        .get(&resource.config.server_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
    resource.config.builder_id.clone_from(
      all
        .builders
        .get(&resource.config.builder_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
  }

  fn edit_config_object(
    _resource: &ResourceToml<Self::PartialConfig>,
    config: IndexMap<String, serde_json::Value>,
  ) -> anyhow::Result<IndexMap<String, serde_json::Value>> {
    config
      .into_iter()
      .map(|(key, value)| {
        match key.as_str() {
          "server_id" => return Ok((String::from("server"), value)),
          "builder_id" => {
            return Ok((String::from("builder"), value));
          }
          _ => {}
        }
        Ok((key, value))
      })
      .collect()
  }
}

impl ToToml for Builder {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    if let BuilderConfig::Server(config) = &mut resource.config {
      let all = all_resources_cache().load();
      config.server_id.clone_from(
        all
          .servers
          .get(&config.server_id)
          .map(|s| &s.name)
          .unwrap_or(&String::new()),
      )
    }
  }

  fn push_additional(
    resource: ResourceToml<Self::PartialConfig>,
    toml: &mut String,
  ) {
    let empty_params = match resource.config {
      PartialBuilderConfig::Aws(config) => config.is_none(),
      PartialBuilderConfig::Server(config) => config.is_none(),
      PartialBuilderConfig::Url(config) => config.is_none(),
    };
    if empty_params {
      // toml_pretty will remove empty map
      // but in this case its needed to deserialize the enums.
      toml.push_str("\nparams = {}");
    }
  }
}

impl ToToml for Procedure {
  fn replace_ids(resource: &mut Resource<Self::Config, Self::Info>) {
    let all = all_resources_cache().load();
    for stage in &mut resource.config.stages {
      for execution in &mut stage.executions {
        match &mut execution.execution {
          Execution::RunProcedure(exec) => exec.procedure.clone_from(
            all
              .procedures
              .get(&exec.procedure)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchRunProcedure(_exec) => {}
          Execution::RunAction(exec) => exec.action.clone_from(
            all
              .actions
              .get(&exec.action)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchRunAction(_exec) => {}
          Execution::RunBuild(exec) => exec.build.clone_from(
            all
              .builds
              .get(&exec.build)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchRunBuild(_exec) => {}
          Execution::CancelBuild(exec) => exec.build.clone_from(
            all
              .builds
              .get(&exec.build)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::Deploy(exec) => exec.deployment.clone_from(
            all
              .deployments
              .get(&exec.deployment)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchDeploy(_exec) => {}
          Execution::PullDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::StartDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::RestartDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::PauseDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::UnpauseDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::StopDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::DestroyDeployment(exec) => {
            exec.deployment.clone_from(
              all
                .deployments
                .get(&exec.deployment)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::BatchDestroyDeployment(_exec) => {}
          Execution::CloneRepo(exec) => exec.repo.clone_from(
            all
              .repos
              .get(&exec.repo)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchCloneRepo(_exec) => {}
          Execution::PullRepo(exec) => exec.repo.clone_from(
            all
              .repos
              .get(&exec.repo)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchPullRepo(_exec) => {}
          Execution::BuildRepo(exec) => exec.repo.clone_from(
            all
              .repos
              .get(&exec.repo)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchBuildRepo(_exec) => {}
          Execution::CancelRepoBuild(exec) => exec.repo.clone_from(
            all
              .repos
              .get(&exec.repo)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::StartContainer(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::RestartContainer(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::PauseContainer(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::UnpauseContainer(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::StopContainer(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::DestroyContainer(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::StartAllContainers(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::RestartAllContainers(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::PauseAllContainers(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::UnpauseAllContainers(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::StopAllContainers(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::PruneContainers(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::DeleteNetwork(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::PruneNetworks(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::DeleteImage(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::PruneImages(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::DeleteVolume(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::PruneVolumes(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::PruneDockerBuilders(exec) => {
            exec.server.clone_from(
              all
                .servers
                .get(&exec.server)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::PruneBuildx(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::PruneSystem(exec) => exec.server.clone_from(
            all
              .servers
              .get(&exec.server)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::RunSync(exec) => exec.sync.clone_from(
            all
              .syncs
              .get(&exec.sync)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::CommitSync(exec) => exec.sync.clone_from(
            all
              .syncs
              .get(&exec.sync)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::DeployStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchDeployStack(_exec) => {}
          Execution::DeployStackIfChanged(exec) => {
            exec.stack.clone_from(
              all
                .stacks
                .get(&exec.stack)
                .map(|r| &r.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::BatchDeployStackIfChanged(_exec) => {}
          Execution::PullStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchPullStack(_exec) => {}
          Execution::StartStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::RestartStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::RunStackService(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::PauseStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::UnpauseStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::StopStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::DestroyStack(exec) => exec.stack.clone_from(
            all
              .stacks
              .get(&exec.stack)
              .map(|r| &r.name)
              .unwrap_or(&String::new()),
          ),
          Execution::BatchDestroyStack(_exec) => {}
          Execution::TestAlerter(exec) => exec.alerter.clone_from(
            all
              .alerters
              .get(&exec.alerter)
              .map(|a| &a.name)
              .unwrap_or(&String::new()),
          ),
          Execution::SendAlert(exec) => {
            exec.alerters.iter_mut().for_each(|a| {
              a.clone_from(
                all
                  .alerters
                  .get(a)
                  .map(|a| &a.name)
                  .unwrap_or(&String::new()),
              )
            })
          }
          Execution::RemoveSwarmNodes(exec) => exec.swarm.clone_from(
            all
              .swarms
              .get(&exec.swarm)
              .map(|a| &a.name)
              .unwrap_or(&String::new()),
          ),
          Execution::RemoveSwarmStacks(exec) => {
            exec.swarm.clone_from(
              all
                .swarms
                .get(&exec.swarm)
                .map(|a| &a.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::RemoveSwarmServices(exec) => {
            exec.swarm.clone_from(
              all
                .swarms
                .get(&exec.swarm)
                .map(|a| &a.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::RemoveSwarmConfigs(exec) => {
            exec.swarm.clone_from(
              all
                .swarms
                .get(&exec.swarm)
                .map(|a| &a.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::RemoveSwarmSecrets(exec) => {
            exec.swarm.clone_from(
              all
                .swarms
                .get(&exec.swarm)
                .map(|a| &a.name)
                .unwrap_or(&String::new()),
            )
          }
          Execution::None(_)
          | Execution::Sleep(_)
          | Execution::ClearRepoCache(_)
          | Execution::BackupCoreDatabase(_)
          | Execution::GlobalAutoUpdate(_)
          | Execution::RotateAllServerKeys(_)
          | Execution::RotateCoreKeys(_) => {}
        }
      }
    }
  }

  fn push_to_toml_string(
    mut resource: ResourceToml<Self::PartialConfig>,
    toml: &mut String,
  ) -> anyhow::Result<()> {
    resource.config =
      Self::Config::default().minimize_partial(resource.config);

    let mut parsed: IndexMap<String, serde_json::Value> =
      serde_json::from_str(&serde_json::to_string(&resource)?)?;

    let config = parsed
      .get_mut("config")
      .context("procedure has no config?")?
      .as_object_mut()
      .context("config is not object?")?;

    let stages = config.remove("stages");

    toml.push_str(
      &toml_pretty::to_string(&parsed, TOML_PRETTY_OPTIONS)
        .context("failed to serialize procedures to toml")?,
    );

    if let Some(stages) = stages {
      let stages =
        stages.as_array().context("stages is not array")?;
      for stage in stages {
        toml.push_str("\n\n[[procedure.config.stage]]\n");
        toml.push_str(
          &toml_pretty::to_string(
            stage,
            // If the execution.params are fully missing,
            // deserialization will fail.
            TOML_PRETTY_OPTIONS.skip_empty_object(false),
          )
          .context("failed to serialize procedures to toml")?,
        );
      }
    }

    Ok(())
  }
}
