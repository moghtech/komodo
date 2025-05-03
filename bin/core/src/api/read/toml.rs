use anyhow::Context;
use komodo_client::{
  api::read::{
    ExportAllResourcesToToml, ExportAllResourcesToTomlResponse,
    ExportResourcesToToml, ExportResourcesToTomlResponse,
    ListUserGroups,
  },
  entities::{
    ResourceTarget, action::Action, alerter::Alerter, build::Build,
    builder::Builder, deployment::Deployment,
    permission::PermissionLevel, procedure::Procedure, repo::Repo,
    resource::ResourceQuery, server::Server, stack::Stack,
    sync::ResourceSync, toml::ResourcesToml, user::User,
  },
};
use mungos::find::find_collect;
use resolver_api::Resolve;

use crate::{
  helpers::query::{
    get_all_tags, get_id_to_tags, get_user_user_group_ids,
  },
  resource,
  state::db_client,
  sync::{
    AllResourcesById,
    toml::{TOML_PRETTY_OPTIONS, ToToml, convert_resource},
    user_groups::convert_user_groups,
  },
};

use super::ReadArgs;

async fn get_all_targets(
  tags: &[String],
  user: &User,
) -> anyhow::Result<Vec<ResourceTarget>> {
  let mut targets = Vec::<ResourceTarget>::new();
  let all_tags = if tags.is_empty() {
    vec![]
  } else {
    get_all_tags(None).await?
  };
  targets.extend(
    resource::list_for_user::<Alerter>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Alerter(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Builder>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Builder(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Server>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Server(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Stack>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Stack(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Deployment>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Deployment(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Build>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Build(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Repo>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Repo(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Procedure>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Procedure(resource.id)),
  );
  targets.extend(
    resource::list_for_user::<Action>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|resource| ResourceTarget::Action(resource.id)),
  );
  targets.extend(
    resource::list_full_for_user::<ResourceSync>(
      ResourceQuery::builder().tags(tags).build(),
      user,
      &all_tags,
    )
    .await?
    .into_iter()
    // These will already be filtered by [ExportResourcesToToml]
    .map(|resource| ResourceTarget::ResourceSync(resource.id)),
  );
  Ok(targets)
}

impl Resolve<ReadArgs> for ExportAllResourcesToToml {
  async fn resolve(
    self,
    args: &ReadArgs,
  ) -> serror::Result<ExportAllResourcesToTomlResponse> {
    let targets = if self.include_resources {
      get_all_targets(&self.tags, &args.user).await?
    } else {
      Vec::new()
    };

    let user_groups = if self.include_user_groups {
      if args.user.admin {
        find_collect(&db_client().user_groups, None, None)
          .await
          .context("failed to query db for user groups")?
          .into_iter()
          .map(|user_group| user_group.id)
          .collect()
      } else {
        get_user_user_group_ids(&args.user.id).await?
      }
    } else {
      Vec::new()
    };

    ExportResourcesToToml {
      targets,
      user_groups,
      include_variables: self.include_variables,
    }
    .resolve(args)
    .await
  }
}

impl Resolve<ReadArgs> for ExportResourcesToToml {
  async fn resolve(
    self,
    args: &ReadArgs,
  ) -> serror::Result<ExportResourcesToTomlResponse> {
    let ExportResourcesToToml {
      targets,
      user_groups,
      include_variables,
    } = self;
    let mut res = ResourcesToml::default();
    let all = AllResourcesById::load().await?;
    let id_to_tags = get_id_to_tags(None).await?;
    let ReadArgs { user } = args;
    for target in targets {
      match target {
        ResourceTarget::Alerter(id) => {
          let alerter = resource::get_check_permissions::<Alerter>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          res.alerters.push(convert_resource::<Alerter>(
            alerter,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::ResourceSync(id) => {
          let sync = resource::get_check_permissions::<ResourceSync>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          if sync.config.file_contents.is_empty()
            && (sync.config.files_on_host
              || !sync.config.repo.is_empty())
          {
            res.resource_syncs.push(convert_resource::<ResourceSync>(
              sync,
              false,
              vec![],
              &id_to_tags,
            ))
          }
        }
        ResourceTarget::Server(id) => {
          let server = resource::get_check_permissions::<Server>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          res.servers.push(convert_resource::<Server>(
            server,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::Builder(id) => {
          let mut builder =
            resource::get_check_permissions::<Builder>(
              &id,
              user,
              PermissionLevel::Read,
            )
            .await?;
          Builder::replace_ids(&mut builder, &all);
          res.builders.push(convert_resource::<Builder>(
            builder,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::Build(id) => {
          let mut build = resource::get_check_permissions::<Build>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          Build::replace_ids(&mut build, &all);
          res.builds.push(convert_resource::<Build>(
            build,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::Deployment(id) => {
          let mut deployment = resource::get_check_permissions::<
            Deployment,
          >(
            &id, user, PermissionLevel::Read
          )
          .await?;
          Deployment::replace_ids(&mut deployment, &all);
          res.deployments.push(convert_resource::<Deployment>(
            deployment,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::Repo(id) => {
          let mut repo = resource::get_check_permissions::<Repo>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          Repo::replace_ids(&mut repo, &all);
          res.repos.push(convert_resource::<Repo>(
            repo,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::Stack(id) => {
          let mut stack = resource::get_check_permissions::<Stack>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          Stack::replace_ids(&mut stack, &all);
          res.stacks.push(convert_resource::<Stack>(
            stack,
            false,
            vec![],
            &id_to_tags,
          ))
        }
        ResourceTarget::Procedure(id) => {
          let mut procedure = resource::get_check_permissions::<
            Procedure,
          >(
            &id, user, PermissionLevel::Read
          )
          .await?;
          Procedure::replace_ids(&mut procedure, &all);
          res.procedures.push(convert_resource::<Procedure>(
            procedure,
            false,
            vec![],
            &id_to_tags,
          ));
        }
        ResourceTarget::Action(id) => {
          let mut action = resource::get_check_permissions::<Action>(
            &id,
            user,
            PermissionLevel::Read,
          )
          .await?;
          Action::replace_ids(&mut action, &all);
          res.actions.push(convert_resource::<Action>(
            action,
            false,
            vec![],
            &id_to_tags,
          ));
        }
        ResourceTarget::System(_) => continue,
      };
    }

    add_user_groups(user_groups, &mut res, &all, args)
      .await
      .context("failed to add user groups")?;

    if include_variables {
      res.variables =
        find_collect(&db_client().variables, None, None)
          .await
          .context("failed to get variables from db")?
          .into_iter()
          .map(|mut variable| {
            if !user.admin && variable.is_secret {
              variable.value = "#".repeat(variable.value.len())
            }
            variable
          })
          .collect();
    }

    let toml = serialize_resources_toml(res)
      .context("failed to serialize resources to toml")?;

    Ok(ExportResourcesToTomlResponse { toml })
  }
}

async fn add_user_groups(
  user_groups: Vec<String>,
  res: &mut ResourcesToml,
  all: &AllResourcesById,
  args: &ReadArgs,
) -> anyhow::Result<()> {
  let user_groups = ListUserGroups {}
    .resolve(args)
    .await
    .map_err(|e| e.error)?
    .into_iter()
    .filter(|ug| {
      user_groups.contains(&ug.name) || user_groups.contains(&ug.id)
    });
  let mut ug = Vec::with_capacity(user_groups.size_hint().0);
  convert_user_groups(user_groups, all, &mut ug).await?;
  res.user_groups = ug.into_iter().map(|ug| ug.1).collect();

  Ok(())
}

fn serialize_resources_toml(
  resources: ResourcesToml,
) -> anyhow::Result<String> {
  let mut toml = String::new();

  for server in resources.servers {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[server]]\n");
    Server::push_to_toml_string(server, &mut toml)?;
  }

  for stack in resources.stacks {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[stack]]\n");
    Stack::push_to_toml_string(stack, &mut toml)?;
  }

  for deployment in resources.deployments {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[deployment]]\n");
    Deployment::push_to_toml_string(deployment, &mut toml)?;
  }

  for build in resources.builds {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[build]]\n");
    Build::push_to_toml_string(build, &mut toml)?;
  }

  for repo in resources.repos {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[repo]]\n");
    Repo::push_to_toml_string(repo, &mut toml)?;
  }

  for procedure in resources.procedures {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[procedure]]\n");
    Procedure::push_to_toml_string(procedure, &mut toml)?;
  }

  for action in resources.actions {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[action]]\n");
    Action::push_to_toml_string(action, &mut toml)?;
  }

  for alerter in resources.alerters {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[alerter]]\n");
    Alerter::push_to_toml_string(alerter, &mut toml)?;
  }

  for builder in resources.builders {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[builder]]\n");
    Builder::push_to_toml_string(builder, &mut toml)?;
  }

  for resource_sync in resources.resource_syncs {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[resource_sync]]\n");
    ResourceSync::push_to_toml_string(resource_sync, &mut toml)?;
  }

  for variable in &resources.variables {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[variable]]\n");
    toml.push_str(
      &toml_pretty::to_string(variable, TOML_PRETTY_OPTIONS)
        .context("failed to serialize variables to toml")?,
    );
  }

  for user_group in &resources.user_groups {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str("[[user_group]]\n");
    toml.push_str(
      &toml_pretty::to_string(user_group, TOML_PRETTY_OPTIONS)
        .context("failed to serialize user_groups to toml")?,
    );
  }

  Ok(toml)
}
