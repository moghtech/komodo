use anyhow::Context;
use database::mungos::mongodb::Collection;
use formatting::format_serror;
use indexmap::IndexSet;
use komodo_client::{
  api::write::RefreshStackCache,
  entities::{
    Operation, ResourceTarget, ResourceTargetVariant,
    permission::{PermissionLevel, SpecificPermission},
    repo::Repo,
    resource::Resource,
    server::Server,
    stack::{
      PartialStackConfig, Stack, StackConfig, StackConfigDiff,
      StackInfo, StackListItem, StackListItemInfo,
      StackQuerySpecifics, StackServiceWithUpdate, StackState,
    },
    to_docker_compatible_name,
    update::Update,
    user::{User, stack_user},
  },
};
use periphery_client::api::compose::ComposeExecution;
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs,
  config::core_config,
  helpers::{periphery_client, query::get_stack_state, repo_link},
  monitor::update_cache_for_server,
  state::{
    action_states, all_resources_cache, db_client,
    server_status_cache, stack_status_cache,
  },
};

use super::get_check_permissions;

impl super::KomodoResource for Stack {
  type Config = StackConfig;
  type PartialConfig = PartialStackConfig;
  type ConfigDiff = StackConfigDiff;
  type Info = StackInfo;
  type ListItem = StackListItem;
  type QuerySpecifics = StackQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Stack
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Stack(id.into())
  }

  fn validated_name(name: &str) -> String {
    to_docker_compatible_name(name)
  }

  fn creator_specific_permissions() -> IndexSet<SpecificPermission> {
    [
      SpecificPermission::Inspect,
      SpecificPermission::Logs,
      SpecificPermission::Terminal,
    ]
    .into_iter()
    .collect()
  }

  fn inherit_specific_permissions_from(
    _self: &Resource<Self::Config, Self::Info>,
  ) -> Option<ResourceTarget> {
    ResourceTarget::Server(_self.config.server_id.clone()).into()
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().stacks
  }

  async fn to_list_item(
    stack: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let status = stack_status_cache().get(&stack.id).await;
    let state = if action_states()
      .stack
      .get(&stack.id)
      .await
      .map(|s| s.get().map(|s| s.deploying))
      .transpose()
      .ok()
      .flatten()
      .unwrap_or_default()
    {
      StackState::Deploying
    } else {
      status.as_ref().map(|s| s.curr.state).unwrap_or_default()
    };
    let project_name = stack.project_name(false);
    let services = status
      .as_ref()
      .map(|s| {
        s.curr
          .services
          .iter()
          .map(|service| StackServiceWithUpdate {
            service: service.service.clone(),
            image: service.image.clone(),
            update_available: service.update_available,
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    let default_git = (
      stack.config.git_provider,
      stack.config.repo,
      stack.config.branch,
      stack.config.git_https,
    );
    let (git_provider, repo, branch, git_https) =
      if stack.config.linked_repo.is_empty() {
        default_git
      } else {
        all_resources_cache()
          .load()
          .repos
          .get(&stack.config.linked_repo)
          .map(|r| {
            (
              r.config.git_provider.clone(),
              r.config.repo.clone(),
              r.config.branch.clone(),
              r.config.git_https,
            )
          })
          .unwrap_or(default_git)
      };

    // This is only true if it is KNOWN to be true. so other cases are false.
    let (project_missing, status) =
      if stack.config.server_id.is_empty()
        || matches!(state, StackState::Down | StackState::Unknown)
      {
        (false, None)
      } else if let Some(status) = server_status_cache()
        .get(&stack.config.server_id)
        .await
        .as_ref()
      {
        if let Some(projects) = &status.projects {
          if let Some(project) = projects
            .iter()
            .find(|project| project.name == project_name)
          {
            (false, project.status.clone())
          } else {
            // The project doesn't exist
            (true, None)
          }
        } else {
          (false, None)
        }
      } else {
        (false, None)
      };

    StackListItem {
      name: stack.name,
      id: stack.id,
      template: stack.template,
      tags: stack.tags,
      resource_type: ResourceTargetVariant::Stack,
      info: StackListItemInfo {
        state,
        status,
        services,
        project_missing,
        file_contents: !stack.config.file_contents.is_empty(),
        server_id: stack.config.server_id,
        linked_repo: stack.config.linked_repo,
        missing_files: stack.info.missing_files,
        files_on_host: stack.config.files_on_host,
        repo_link: repo_link(
          &git_provider,
          &repo,
          &branch,
          git_https,
        ),
        git_provider,
        repo,
        branch,
        latest_hash: stack.info.latest_hash,
        deployed_hash: stack.info.deployed_hash,
      },
    }
  }

  async fn busy(id: &String) -> anyhow::Result<bool> {
    action_states()
      .stack
      .get(id)
      .await
      .unwrap_or_default()
      .busy()
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateStack
  }

  fn user_can_create(user: &User) -> bool {
    user.admin || !core_config().disable_non_admin_create
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_create(
    created: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    if let Err(e) = (RefreshStackCache {
      stack: created.name.clone(),
    })
    .resolve(&WriteArgs {
      user: stack_user().to_owned(),
    })
    .await
    {
      update.push_error_log(
        "Refresh stack cache",
        format_serror(&e.error.context("The stack cache has failed to refresh. This may be due to a misconfiguration of the Stack").into())
      );
    };
    if created.config.server_id.is_empty() {
      return Ok(());
    }
    let Ok(server) = super::get::<Server>(&created.config.server_id)
      .await
      .inspect_err(|e| {
        warn!(
          "Failed to get Server for Stack {} | {e:#}",
          created.name
        )
      })
    else {
      return Ok(());
    };
    update_cache_for_server(&server, true).await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateStack
  }

  async fn validate_update_config(
    _id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_update(
    updated: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    Self::post_create(updated, update).await
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameStack
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteStack
  }

  async fn pre_delete(
    stack: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    // If it is Up, it should be taken down
    let state = get_stack_state(stack)
      .await
      .context("failed to get stack state")?;
    if matches!(state, StackState::Down | StackState::Unknown) {
      return Ok(());
    }
    // stack needs to be destroyed
    let server =
      match super::get::<Server>(&stack.config.server_id).await {
        Ok(server) => server,
        Err(e) => {
          update.push_error_log(
            "destroy stack",
            format_serror(
              &e.context(format!(
                "failed to retrieve server at {} from db.",
                stack.config.server_id
              ))
              .into(),
            ),
          );
          return Ok(());
        }
      };

    if !server.config.enabled {
      update.push_simple_log(
        "destroy stack",
        "skipping stack destroy, server is disabled.",
      );
      return Ok(());
    }

    let periphery = match periphery_client(&server) {
      Ok(periphery) => periphery,
      Err(e) => {
        // This case won't ever happen, as periphery_client only fallible if the server is disabled.
        // Leaving it for completeness sake
        update.push_error_log(
          "destroy stack",
          format_serror(
            &e.context("failed to get periphery client").into(),
          ),
        );
        return Ok(());
      }
    };

    match periphery
      .request(ComposeExecution {
        project: stack.project_name(false),
        command: String::from("down --remove-orphans"),
      })
      .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_simple_log(
        "Failed to destroy stack",
        format_serror(
          &e.context(
            "failed to destroy stack on periphery server before delete",
          )
          .into(),
        ),
      ),
    };

    Ok(())
  }

  async fn post_delete(
    resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    stack_status_cache().remove(&resource.id).await;
    Ok(())
  }
}

#[instrument(skip(user))]
async fn validate_config(
  config: &mut PartialStackConfig,
  user: &User,
) -> anyhow::Result<()> {
  if let Some(server_id) = &config.server_id
    && !server_id.is_empty()
  {
    let server = get_check_permissions::<Server>(
      server_id,
      user,
      PermissionLevel::Read.attach(),
    )
    .await
    .context("Cannot attach Stack to this Server")?;
    // in case it comes in as name
    config.server_id = Some(server.id);
  }
  if let Some(linked_repo) = &config.linked_repo
    && !linked_repo.is_empty()
  {
    let repo = get_check_permissions::<Repo>(
      linked_repo,
      user,
      PermissionLevel::Read.attach(),
    )
    .await
    .context("Cannot attach Repo to this Stack")?;
    // in case it comes in as name
    config.linked_repo = Some(repo.id);
  }
  Ok(())
}
