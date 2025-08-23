use std::{collections::HashMap, time::Duration};

use anyhow::{Context, anyhow};
use formatting::{Color, bold, colored, format_serror, muted};
use futures::future::join_all;
use komodo_client::{
  api::{
    execute::{Deploy, DeployStack},
    read::ListBuildVersions,
  },
  entities::{
    ResourceTarget,
    deployment::{
      Deployment, DeploymentConfig, DeploymentImage, DeploymentState,
      PartialDeploymentConfig,
    },
    stack::{
      PartialStackConfig, Stack, StackConfig,
      StackRemoteFileContents, StackState,
    },
    sync::SyncDeployUpdate,
    toml::ResourceToml,
    update::Log,
    user::sync_user,
  },
};
use resolver_api::Resolve;

use crate::{
  api::{
    execute::{ExecuteArgs, ExecuteRequest},
    read::ReadArgs,
  },
  helpers::update::init_execution_update,
  state::{deployment_status_cache, stack_status_cache},
};

use super::ResourceSyncTrait;

/// All entries in here are due to be deployed,
/// after the given dependencies,
/// with the given reason.
pub type ToDeployCache =
  Vec<(ResourceTarget, String, Vec<ResourceTarget>)>;

#[derive(Clone, Copy)]
pub struct SyncDeployParams<'a> {
  pub deployments: &'a [ResourceToml<PartialDeploymentConfig>],
  // Names to deployments
  pub deployment_map: &'a HashMap<String, Deployment>,
  pub stacks: &'a [ResourceToml<PartialStackConfig>],
  // Names to stacks
  pub stack_map: &'a HashMap<String, Stack>,
}

pub async fn deploy_from_cache(
  mut to_deploy: ToDeployCache,
  logs: &mut Vec<Log>,
) {
  if to_deploy.is_empty() {
    return;
  }
  let mut log = format!(
    "{}: running executions to sync deployment / stack state",
    muted("INFO")
  );
  let mut round = 1;
  let user = sync_user();

  while !to_deploy.is_empty() {
    // Collect all waiting deployments without waiting dependencies.
    let good_to_deploy = to_deploy
      .iter()
      .filter(|(_, _, after)| {
        to_deploy
          .iter()
          .all(|(target, _, _)| !after.contains(target))
      })
      // The target / reason need the be cloned out to to_deploy is not borrowed from.
      // to_deploy will be mutably accessed later.
      .map(|(target, reason, _)| (target.clone(), reason.clone()))
      .collect::<HashMap<_, _>>();

    // Deploy the ones ready for deployment
    let res = join_all(good_to_deploy.iter().map(
      |(target, reason)| async move {
        let res = async {
          match &target {
            ResourceTarget::Deployment(name) => {
              let req = ExecuteRequest::Deploy(Deploy {
                deployment: name.to_string(),
                stop_signal: None,
                stop_time: None,
              });

              let update = init_execution_update(&req, user).await?;
              let ExecuteRequest::Deploy(req) = req else {
                unreachable!()
              };
              req
                .resolve(&ExecuteArgs {
                  user: user.to_owned(),
                  update,
                })
                .await
            }
            ResourceTarget::Stack(name) => {
              let req = ExecuteRequest::DeployStack(DeployStack {
                stack: name.to_string(),
                services: Vec::new(),
                stop_time: None,
              });

              let update = init_execution_update(&req, user).await?;
              let ExecuteRequest::DeployStack(req) = req else {
                unreachable!()
              };
              req
                .resolve(&ExecuteArgs {
                  user: user.to_owned(),
                  update,
                })
                .await
            }
            _ => unreachable!(),
          }
        }
        .await;
        (target, reason, res)
      },
    ))
    .await;

    let mut has_error = false;

    // Log results of deploy
    for (target, reason, res) in res {
      let (resource, name) = target.extract_variant_id();
      if let Err(e) = res {
        has_error = true;
        log.push_str(&format!(
          "\n{}: failed to deploy {resource} '{}' in round {} | {:#}",
          colored("ERROR", Color::Red),
          bold(name),
          bold(round),
          e.error
        ));
      } else {
        log.push_str(&format!(
          "\n{}: deployed {resource} '{}' in round {} with reason: {reason}",
          muted("INFO"),
          bold(name),
          bold(round)
        ));
      }
    }

    // Early exit if any deploy has errors
    if has_error {
      log.push_str(&format!(
        "\n{}: exited in round {} {}",
        muted("INFO"),
        bold(round),
        colored("with errors", Color::Red)
      ));
      logs.push(Log::error("Sync Deploy", log));
      return;
    }

    // Remove the deployed ones from 'to_deploy'
    to_deploy
      .retain(|(target, _, _)| !good_to_deploy.contains_key(target));

    // If there must be another round, these are dependent on the first round.
    // Sleep for 1s to allow for first round to startup
    if !to_deploy.is_empty() {
      // Increment the round
      round += 1;
      tokio::time::sleep(Duration::from_secs(1)).await;
    }
  }

  log.push_str(&format!(
    "\n{}: finished after {} round{}",
    muted("INFO"),
    bold(round),
    if round > 1 { "s" } else { Default::default() }
  ));

  logs.push(Log::simple("Sync Deploy", log));
}

pub async fn get_updates_for_view(
  params: SyncDeployParams<'_>,
) -> SyncDeployUpdate {
  let inner = async {
    let mut update = SyncDeployUpdate {
      to_deploy: 0,
      log: String::from("Deploy Updates\n-------------------\n"),
    };
    let mut lines = Vec::<String>::new();
    for (target, reason, after) in build_deploy_cache(params).await? {
      update.to_deploy += 1;
      let mut line = format!(
        "{}: {}. reason: {reason}",
        colored("Deploy", Color::Green),
        bold(format!("{target:?}")),
      );
      if !after.is_empty() {
        line.push_str(&format!(
          "\n{}: {}",
          colored("After", Color::Blue),
          after
            .iter()
            .map(|target| format!("{target:?}"))
            .collect::<Vec<_>>()
            .join(", ")
        ))
      }
      lines.push(line);
    }

    update.log.push_str(&lines.join("\n-------------------\n"));

    anyhow::Ok(update)
  };
  match inner.await {
    Ok(res) => res,
    Err(e) => SyncDeployUpdate {
      to_deploy: 0,
      log: format_serror(
        &e.context("failed to get deploy updates for view").into(),
      ),
    },
  }
}

/// Entries are keyed by ResourceTargets wrapping "name" instead of "id".
/// If entry is None, it is confirmed no-deploy.
/// If it is Some, it is confirmed deploy with provided reason and dependencies.
///
/// Used to build up resources to deploy earlier in the sync.
type ToDeployCacheInner =
  HashMap<ResourceTarget, Option<(String, Vec<ResourceTarget>)>>;

/// Maps build ids to latest versions as string.
type BuildVersionCache = HashMap<String, String>;

pub async fn build_deploy_cache(
  params: SyncDeployParams<'_>,
) -> anyhow::Result<ToDeployCache> {
  let mut cache = ToDeployCacheInner::new();
  let mut build_version_cache = BuildVersionCache::new();

  // Just ensure they are all in the cache by looping through them all
  for deployment in params.deployments {
    build_cache_for_deployment(
      deployment,
      params,
      &mut cache,
      &mut build_version_cache,
    )
    .await?;
  }
  for stack in params.stacks {
    build_cache_for_stack(
      stack,
      params,
      &mut cache,
      &mut build_version_cache,
    )
    .await?;
  }

  let cache = cache
    .into_iter()
    .filter_map(|(target, entry)| {
      let (reason, after) = entry?;
      Some((target, (reason, after)))
    })
    .collect::<HashMap<_, _>>();

  // Have to clone here to use it after 'into_iter' below.
  // All entries in cache at this point are deploying.
  let clone = cache.clone();

  Ok(
    cache
      .into_iter()
      .map(|(target, (reason, mut after))| {
        // Only keep targets which are deploying.
        after.retain(|target| clone.contains_key(target));
        (target, reason, after)
      })
      .collect(),
  )
}

type BuildRes<'a> = std::pin::Pin<
  Box<
    dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a,
  >,
>;

fn build_cache_for_deployment<'a>(
  deployment: &'a ResourceToml<PartialDeploymentConfig>,
  SyncDeployParams {
    deployments,
    deployment_map,
    stacks,
    stack_map,
  }: SyncDeployParams<'a>,
  cache: &'a mut ToDeployCacheInner,
  build_version_cache: &'a mut BuildVersionCache,
) -> BuildRes<'a> {
  Box::pin(async move {
    let target = ResourceTarget::Deployment(deployment.name.clone());

    // First check existing, and continue if already handled.
    if cache.contains_key(&target) {
      return Ok(());
    }

    // Check if deployment doesn't have "deploy" enabled.
    if !deployment.deploy {
      cache.insert(target, None);
      return Ok(());
    }

    let after = get_after_as_resource_targets(
      &deployment.name,
      &deployment.after,
      deployment_map,
      deployments,
      stack_map,
      stacks,
    )?;

    let Some(original) = deployment_map.get(&deployment.name) else {
      // This block is the None case, deployment is not created, should definitely deploy
      cache.insert(
        target,
        Some((String::from("deploy on creation"), after)),
      );
      return Ok(());
    };

    let status = &deployment_status_cache()
      .get_or_insert_default(&original.id)
      .await
      .curr;
    let state = status.state;

    match state {
      DeploymentState::Unknown => {
        // Can't do anything with unknown state
        cache.insert(target, None);
        return Ok(());
      }
      DeploymentState::Running => {
        // Here can diff the changes, to see if they merit a redeploy.

        // First merge toml resource config (partial) onto default resource config.
        // Makes sure things that aren't defined in toml (come through as None) actually get removed.
        let config: DeploymentConfig =
          deployment.config.clone().into();
        let mut config: PartialDeploymentConfig = config.into();

        Deployment::validate_partial_config(&mut config);

        let mut diff =
          Deployment::get_diff(original.config.clone(), config)?;

        Deployment::validate_diff(&mut diff);
        // Needs to only check config fields that affect docker run
        let changed = diff.server_id.is_some()
          || diff.image.is_some()
          || diff.image_registry_account.is_some()
          || diff.skip_secret_interp.is_some()
          || diff.network.is_some()
          || diff.restart.is_some()
          || diff.command.is_some()
          || diff.extra_args.is_some()
          || diff.ports.is_some()
          || diff.volumes.is_some()
          || diff.environment.is_some()
          || diff.labels.is_some();
        if changed {
          cache.insert(
            target,
            Some((
              String::from("deployment config has changed"),
              after,
            )),
          );
          return Ok(());
        }
      }
      // All other cases will require Deploy to enter Running state.
      _ => {
        cache.insert(
          target,
          Some((
            format!(
              "deployment has {} state",
              colored(state, Color::Red)
            ),
            after,
          )),
        );
        return Ok(());
      }
    };

    // We know the config hasn't changed at this point, but still need
    // to check if attached build has updated. Can check original for this (know it hasn't changed)
    if let DeploymentImage::Build { build_id, version } =
      &original.config.image
    {
      // check if version is none, ie use latest build
      if !version.is_none() {
        let deployed_version = status
          .container
          .as_ref()
          .and_then(|c| c.image.as_ref()?.split(':').next_back())
          .unwrap_or("0.0.0");
        match build_version_cache.get(build_id) {
          Some(version) if deployed_version != version => {
            cache.insert(
              target,
              Some((
                format!("build has new version: {version}"),
                after,
              )),
            );
            return Ok(());
          }
          // Build version is the same, still need to check 'after'
          Some(_) => {}
          None => {
            let Some(version) = (ListBuildVersions {
              build: build_id.to_string(),
              limit: Some(1),
              ..Default::default()
            })
            .resolve(&ReadArgs {
              user: sync_user().to_owned(),
            })
            .await
            .map_err(|e| e.error)
            .context("failed to get build versions")?
            .pop() else {
              // The build has never been built.
              // Skip deploy regardless of 'after' (it can't be deployed)
              // Not sure how this would be reached on Running deployment...
              cache.insert(target, None);
              return Ok(());
            };
            let version = version.version.to_string();
            build_version_cache
              .insert(build_id.to_string(), version.clone());
            if deployed_version != version {
              // Same as 'Some' case out of the cache
              cache.insert(
                target,
                Some((
                  format!("build has new version: {version}"),
                  after,
                )),
              );
              return Ok(());
            }
          }
        }
      }
    };

    // Check 'after' to see if they deploy.
    insert_target_using_after_list(
      target,
      after,
      SyncDeployParams {
        deployments,
        deployment_map,
        stacks,
        stack_map,
      },
      cache,
      build_version_cache,
    )
    .await
  })
}

fn build_cache_for_stack<'a>(
  stack: &'a ResourceToml<PartialStackConfig>,
  SyncDeployParams {
    deployments,
    deployment_map,
    stacks,
    stack_map,
  }: SyncDeployParams<'a>,
  cache: &'a mut ToDeployCacheInner,
  build_version_cache: &'a mut BuildVersionCache,
) -> BuildRes<'a> {
  Box::pin(async move {
    let target = ResourceTarget::Stack(stack.name.clone());

    // First check existing, and continue if already handled.
    if cache.contains_key(&target) {
      return Ok(());
    }

    // Check if stack doesn't have "deploy" enabled.
    if !stack.deploy {
      cache.insert(target, None);
      return Ok(());
    }

    let after = get_after_as_resource_targets(
      &stack.name,
      &stack.after,
      deployment_map,
      deployments,
      stack_map,
      stacks,
    )?;

    let Some(original) = stack_map.get(&stack.name) else {
      // This block is the None case, deployment is not created, should definitely deploy
      cache.insert(
        target,
        Some((String::from("deploy on creation"), after)),
      );
      return Ok(());
    };

    let status = &stack_status_cache()
      .get_or_insert_default(&original.id)
      .await
      .curr;
    let state = status.state;

    match state {
      StackState::Unknown => {
        // Can't do anything with unknown state
        cache.insert(target, None);
        return Ok(());
      }
      StackState::Running => {
        // Here can diff the changes, to see if they merit a redeploy.

        // See if any remote contents don't match deployed contents
        #[allow(clippy::single_match)]
        match (
          &original.info.deployed_contents,
          &original.info.remote_contents,
        ) {
          (Some(deployed_contents), Some(remote_contents)) => {
            for StackRemoteFileContents {
              path,
              contents,
              services: _services,
              requires: _requires,
            } in remote_contents
            {
              if let Some(deployed) =
                deployed_contents.iter().find(|c| &c.path == path)
              {
                if &deployed.contents != contents {
                  cache.insert(
                    target,
                    Some((
                      format!(
                        "File contents for {path} have changed"
                      ),
                      after,
                    )),
                  );
                  return Ok(());
                }
              } else {
                cache.insert(
                  target,
                  Some((
                    format!("New file contents at {path}"),
                    after,
                  )),
                );
                return Ok(());
              }
            }
          }
          // Maybe should handle other cases
          _ => {}
        }

        // Merge toml resource config (partial) onto default resource config.
        // Makes sure things that aren't defined in toml (come through as None) actually get removed.
        let config: StackConfig = stack.config.clone().into();
        let mut config: PartialStackConfig = config.into();

        Stack::validate_partial_config(&mut config);

        let mut diff =
          Stack::get_diff(original.config.clone(), config)?;

        Stack::validate_diff(&mut diff);
        // Needs to only check config fields that affect docker compose command
        let changed = diff.server_id.is_some()
          || diff.project_name.is_some()
          || diff.run_directory.is_some()
          || diff.file_paths.is_some()
          || diff.file_contents.is_some()
          || diff.skip_secret_interp.is_some()
          || diff.extra_args.is_some()
          || diff.environment.is_some()
          || diff.env_file_path.is_some()
          || diff.repo.is_some()
          || diff.branch.is_some()
          || diff.commit.is_some();
        if changed {
          cache.insert(
            target,
            Some((String::from("stack config has changed"), after)),
          );
          return Ok(());
        }
      }
      // All other cases will require Deploy to enter Running state.
      _ => {
        cache.insert(
          target,
          Some((
            format!("stack has {} state", colored(state, Color::Red)),
            after,
          )),
        );
        return Ok(());
      }
    };

    // Check 'after' to see if they deploy.
    insert_target_using_after_list(
      target,
      after,
      SyncDeployParams {
        deployments,
        deployment_map,
        stacks,
        stack_map,
      },
      cache,
      build_version_cache,
    )
    .await
  })
}

async fn insert_target_using_after_list<'a>(
  target: ResourceTarget,
  after: Vec<ResourceTarget>,
  SyncDeployParams {
    deployments,
    deployment_map,
    stacks,
    stack_map,
  }: SyncDeployParams<'a>,
  cache: &'a mut ToDeployCacheInner,
  build_version_cache: &'a mut BuildVersionCache,
) -> anyhow::Result<()> {
  for parent in &after {
    match cache.get(parent) {
      Some(Some(_)) => {
        // a parent will deploy
        let (variant, name) = parent.extract_variant_id();
        cache.insert(
          target.to_owned(),
          Some((
            format!(
              "{variant} parent dependency '{}' is deploying",
              bold(name)
            ),
            after,
          )),
        );
        return Ok(());
      }
      // The parent will not deploy, do nothing here.
      Some(None) => {}
      None => {
        match parent {
          ResourceTarget::Deployment(name) => {
            let Some(parent_deployment) =
              deployments.iter().find(|d| &d.name == name)
            else {
              // The parent is not in the sync, so won't be deploying
              // Note that cross-sync deploy dependencies are not currently supported.
              continue;
            };
            // Recurse to add the parent to cache, then check again.
            build_cache_for_deployment(
              parent_deployment,
              SyncDeployParams {
                deployments,
                deployment_map,
                stacks,
                stack_map,
              },
              cache,
              build_version_cache,
            )
            .await?;
            match cache.get(parent) {
              Some(Some(_)) => {
                // Same as the 'Some' case above
                let (variant, name) = parent.extract_variant_id();
                cache.insert(
                  target.to_owned(),
                  Some((
                    format!(
                      "{variant} parent dependency '{}' is deploying",
                      bold(name)
                    ),
                    after,
                  )),
                );
                return Ok(());
              }
              // The parent will not deploy, do nothing here.
              Some(None) => {}
              None => {
                return Err(anyhow!(
                  "Did not find parent in cache after build recursion. This should not happen."
                ));
              }
            }
          }
          ResourceTarget::Stack(name) => {
            let Some(parent_stack) =
              stacks.iter().find(|d| &d.name == name)
            else {
              // The parent is not in the sync, so won't be deploying
              // Note that cross-sync deploy dependencies are not currently supported.
              continue;
            };
            // Recurse to add the parent to cache, then check again.
            build_cache_for_stack(
              parent_stack,
              SyncDeployParams {
                deployments,
                deployment_map,
                stacks,
                stack_map,
              },
              cache,
              build_version_cache,
            )
            .await?;
            match cache.get(parent) {
              Some(Some(_)) => {
                // Same as the 'Some' case above
                let (variant, name) = parent.extract_variant_id();
                cache.insert(
                  target.to_owned(),
                  Some((
                    format!(
                      "{variant} parent dependency '{}' is deploying",
                      bold(name)
                    ),
                    after,
                  )),
                );
                return Ok(());
              }
              // The parent will not deploy, do nothing here.
              Some(None) => {}
              None => {
                return Err(anyhow!(
                  "Did not find parent in cache after build recursion. This should not happen."
                ));
              }
            }
          }
          _ => unreachable!(),
        }
      }
    }
  }

  // If it has reached here, its not deploying
  cache.insert(target, None);
  Ok(())
}

fn get_after_as_resource_targets(
  resource_name: &str,
  after: &[String],
  // Names to deployments
  deployment_map: &HashMap<String, Deployment>,
  deployments: &[ResourceToml<PartialDeploymentConfig>],
  // Names to stacks
  stack_map: &HashMap<String, Stack>,
  stacks: &[ResourceToml<PartialStackConfig>],
) -> anyhow::Result<Vec<ResourceTarget>> {
  after
    .iter()
    .map(|name| match deployment_map.get(name) {
      Some(_) => Ok(ResourceTarget::Deployment(name.clone())),
      None => {
        if deployments
          .iter()
          .any(|deployment| deployment.name.as_str() == resource_name)
        {
          Ok(ResourceTarget::Deployment(name.clone()))
        } else {
          match stack_map.get(name) {
            Some(_) => Ok(ResourceTarget::Stack(name.clone())),
            None => {
              if stacks
                .iter()
                .any(|stack| stack.name.as_str() == resource_name)
              {
                Ok(ResourceTarget::Stack(name.clone()))
              } else {
                Err(anyhow!("failed to match deploy dependency in 'after' list | resource: {resource_name} | dependency: {name}"))
              }
            }
          }
        }
      }
    })
    .collect()
}
