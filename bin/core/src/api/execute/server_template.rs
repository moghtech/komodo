use anyhow::{anyhow, Context};
use formatting::format_serror;
use komodo_client::{
  api::{execute::LaunchServer, write::CreateServer},
  entities::{
    permission::PermissionLevel,
    server::PartialServerConfig,
    server_template::{ServerTemplate, ServerTemplateConfig},
    update::Update,
  },
};
use mungos::mongodb::bson::doc;
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs,
  cloud::{
    aws::ec2::launch_ec2_instance, hetzner::launch_hetzner_server,
  },
  helpers::update::update_update,
  resource,
  state::db_client,
};

use super::ExecuteArgs;

impl Resolve<ExecuteArgs> for LaunchServer {
  #[instrument(name = "LaunchServer", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    // validate name isn't already taken by another server
    if db_client()
      .servers
      .find_one(doc! {
        "name": &self.name
      })
      .await
      .context("failed to query db for servers")?
      .is_some()
    {
      return Err(anyhow!("name is already taken").into());
    }

    let template = resource::get_check_permissions::<ServerTemplate>(
      &self.server_template,
      user,
      PermissionLevel::Execute,
    )
    .await?;

    let mut update = update.clone();

    update.push_simple_log(
      "launching server",
      format!("{:#?}", template.config),
    );
    update_update(update.clone()).await?;

    let config = match template.config {
      ServerTemplateConfig::Aws(config) => {
        let region = config.region.clone();
        let use_https = config.use_https;
        let port = config.port;
        let instance =
          match launch_ec2_instance(&self.name, config).await {
            Ok(instance) => instance,
            Err(e) => {
              update.push_error_log(
                "launch server",
                format!("failed to launch aws instance\n\n{e:#?}"),
              );
              update.finalize();
              update_update(update.clone()).await?;
              return Ok(update);
            }
          };
        update.push_simple_log(
          "launch server",
          format!(
            "successfully launched server {} on ip {}",
            self.name, instance.ip
          ),
        );
        let protocol = if use_https { "https" } else { "http" };
        PartialServerConfig {
          address: format!("{protocol}://{}:{port}", instance.ip)
            .into(),
          region: region.into(),
          ..Default::default()
        }
      }
      ServerTemplateConfig::Hetzner(config) => {
        let datacenter = config.datacenter;
        let use_https = config.use_https;
        let port = config.port;
        let server =
          match launch_hetzner_server(&self.name, config).await {
            Ok(server) => server,
            Err(e) => {
              update.push_error_log(
                "launch server",
                format!("failed to launch hetzner server\n\n{e:#?}"),
              );
              update.finalize();
              update_update(update.clone()).await?;
              return Ok(update);
            }
          };
        update.push_simple_log(
          "launch server",
          format!(
            "successfully launched server {} on ip {}",
            self.name, server.ip
          ),
        );
        let protocol = if use_https { "https" } else { "http" };
        PartialServerConfig {
          address: format!("{protocol}://{}:{port}", server.ip)
            .into(),
          region: datacenter.as_ref().to_string().into(),
          ..Default::default()
        }
      }
    };

    match (CreateServer {
      name: self.name,
      config,
    })
    .resolve(&WriteArgs { user: user.clone() })
    .await
    {
      Ok(server) => {
        update.push_simple_log(
          "create server",
          format!("created server {} ({})", server.name, server.id),
        );
        update.other_data = server.id;
      }
      Err(e) => {
        update.push_error_log(
          "create server",
          format_serror(
            &e.error.context("failed to create server").into(),
          ),
        );
      }
    };

    update.finalize();
    update_update(update.clone()).await?;
    Ok(update)
  }
}
