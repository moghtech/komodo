use anyhow::Context;
use database::mungos::find::find_collect;
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
    swarm::Swarm, sync::ResourceSync, toml::ResourcesToml,
    user::User,
  },
};
use mogh_resolver::Resolve;

use crate::{
  helpers::query::{
    get_all_tags, get_id_to_tags, get_user_user_group_ids,
  },
  permission::get_check_permissions,
  resource,
  state::db_client,
  sync::{
    toml::{ToToml, convert_resource},
    user_groups::{convert_user_groups, user_group_to_toml},
    variables::variable_to_toml,
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

  macro_rules! extend_targets {
    ($($Type:ident),* $(,)?) => {
      $(
        targets.extend(
          resource::list_full_for_user::<$Type>(
            ResourceQuery::builder().tags(tags).build(),
            user,
            PermissionLevel::Read.into(),
            &all_tags,
          )
          .await?
          .into_iter()
          .map(|resource| ResourceTarget::$Type(resource.id)),
        );
      )*
    };
  }

  extend_targets!(
    Alerter,
    Builder,
    Server,
    Swarm,
    Stack,
    Deployment,
    Build,
    Repo,
    Procedure,
    Action,
    ResourceSync,
  );

  Ok(targets)
}

impl Resolve<ReadArgs> for ExportAllResourcesToToml {
  async fn resolve(
    self,
    args: &ReadArgs,
  ) -> mogh_error::Result<ExportAllResourcesToTomlResponse> {
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
      existing: self.existing,
    }
    .resolve(args)
    .await
  }
}

impl Resolve<ReadArgs> for ExportResourcesToToml {
  async fn resolve(
    self,
    args: &ReadArgs,
  ) -> mogh_error::Result<ExportResourcesToTomlResponse> {
    let ExportResourcesToToml {
      targets,
      user_groups,
      include_variables,
      existing,
    } = self;
    let mut res = ResourcesToml::default();
    let id_to_tags = get_id_to_tags(None).await?;
    let ReadArgs { user } = args;
    macro_rules! convert_target {
      ($id:expr, $Type:ident, $field:ident) => {{
        let mut resource = get_check_permissions::<$Type>(
          &$id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
        $Type::replace_ids(&mut resource);
        let (deploy, after) = existing
          .as_ref()
          .and_then(|e| {
            e.$field.iter().find(|r| r.name == resource.name)
          })
          .map(|r| (r.deploy, r.after.clone()))
          .unwrap_or_default();
        res.$field.push(convert_resource::<$Type>(
          resource,
          deploy,
          after,
          &id_to_tags,
        ));
      }};
    }
    for target in targets {
      match target {
        ResourceTarget::Server(id) => {
          convert_target!(id, Server, servers)
        }
        ResourceTarget::Swarm(id) => {
          convert_target!(id, Swarm, swarms)
        }
        ResourceTarget::Stack(id) => {
          convert_target!(id, Stack, stacks)
        }
        ResourceTarget::Deployment(id) => {
          convert_target!(id, Deployment, deployments)
        }
        ResourceTarget::Build(id) => {
          convert_target!(id, Build, builds)
        }
        ResourceTarget::Repo(id) => convert_target!(id, Repo, repos),
        ResourceTarget::Procedure(id) => {
          convert_target!(id, Procedure, procedures)
        }
        ResourceTarget::Action(id) => {
          convert_target!(id, Action, actions)
        }
        ResourceTarget::Builder(id) => {
          convert_target!(id, Builder, builders)
        }
        ResourceTarget::Alerter(id) => {
          convert_target!(id, Alerter, alerters)
        }
        ResourceTarget::ResourceSync(id) => {
          let mut sync = get_check_permissions::<ResourceSync>(
            &id,
            user,
            PermissionLevel::Read.into(),
          )
          .await?;
          if sync.config.file_contents.is_empty()
            && (sync.config.files_on_host
              || !sync.config.repo.is_empty()
              || !sync.config.linked_repo.is_empty())
          {
            ResourceSync::replace_ids(&mut sync);
            res.resource_syncs.push(convert_resource::<ResourceSync>(
              sync,
              false,
              vec![],
              &id_to_tags,
            ))
          }
        }
        ResourceTarget::System(_) => continue,
      };
    }

    add_user_groups(user_groups, &mut res, args)
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
  convert_user_groups(user_groups, &mut ug).await?;
  res.user_groups = ug.into_iter().map(|ug| ug.1).collect();

  Ok(())
}

fn serialize_resources_toml(
  resources: ResourcesToml,
) -> anyhow::Result<String> {
  let mut toml = String::new();

  macro_rules! serialize_resources {
    ($(($Type:ident, $field:ident, $header:literal)),* $(,)?) => {
      $(
        for resource in resources.$field {
          if !toml.is_empty() {
            toml.push_str("\n\n##\n\n");
          }
          toml.push_str(concat!("[[",$header,"]]\n"));
          $Type::push_to_toml_string(resource, &mut toml)?;
        }
      )*
    };
  }
  serialize_resources!(
    (Server, servers, "server"),
    (Swarm, swarms, "swarm"),
    (Stack, stacks, "stack"),
    (Deployment, deployments, "deployment"),
    (Build, builds, "build"),
    (Repo, repos, "repo"),
    (Procedure, procedures, "procedure"),
    (Action, actions, "action"),
    (Alerter, alerters, "alerter"),
    (Builder, builders, "builder"),
    (ResourceSync, resource_syncs, "resource_sync"),
  );

  for variable in &resources.variables {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str(&variable_to_toml(variable)?);
  }

  for user_group in resources.user_groups {
    if !toml.is_empty() {
      toml.push_str("\n\n##\n\n");
    }
    toml.push_str(&user_group_to_toml(user_group)?);
  }

  Ok(toml)
}
