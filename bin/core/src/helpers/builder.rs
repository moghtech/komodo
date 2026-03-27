use std::time::Duration;

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::oid::ObjectId;
use formatting::muted;
use komodo_client::entities::{
  Version,
  builder::{AwsBuilderConfig, Builder, BuilderConfig},
  komodo_timestamp,
  server::Server,
  update::{Log, Update},
};
use periphery_client::api::{self, GetVersionResponse};

use crate::{
  cloud::{
    BuildCleanupData,
    aws::ec2::{
      Ec2Instance, launch_ec2_instance,
      terminate_ec2_instance_with_retry,
    },
  },
  connection::PeripheryConnectionArgs,
  helpers::update::update_update,
  periphery::PeripheryClient,
  resource,
};

use super::periphery_client;

const BUILDER_POLL_RATE_SECS: u64 = 2;
const BUILDER_POLL_MAX_TRIES: usize = 60;

#[instrument(
  "ConnectBuilderPeriphery",
  skip_all,
  fields(
    resource_name,
    builder_id = builder.id,
    update_id = update.id
  )
)]
pub async fn connect_builder_periphery(
  // build: &Build,
  resource_name: String,
  version: Option<Version>,
  builder: Builder,
  update: &mut Update,
) -> anyhow::Result<(PeripheryClient, BuildCleanupData)> {
  match builder.config {
    BuilderConfig::Url(config) => {
      if config.address.is_empty() {
        return Err(anyhow!(
          "Builder has not yet configured an address"
        ));
      }
      // TODO: Dont use builder id, or will be problems
      // with simultaneous spawned builders.
      let periphery = PeripheryClient::new(
        PeripheryConnectionArgs::from_url_builder(
          &ObjectId::new().to_hex(),
          &config,
        ),
        config.insecure_tls,
      )
      .await?;
      periphery
        .health_check()
        .await
        .context("Url Builder failed health check")?;
      Ok((periphery, BuildCleanupData::Url))
    }
    BuilderConfig::Server(config) => {
      if config.server_id.is_empty() {
        return Err(anyhow!("Builder has not configured a server"));
      }
      let server = resource::get::<Server>(&config.server_id).await?;
      let periphery = periphery_client(&server).await?;
      Ok((periphery, BuildCleanupData::Server))
    }
    BuilderConfig::Aws(config) => {
      get_aws_builder(&resource_name, version, config, update).await
    }
  }
}

#[instrument(
  "GetAwsBuilder",
  skip_all,
  fields(
    resource_name,
    update_id = update.id,
  )
)]
async fn get_aws_builder(
  resource_name: &str,
  version: Option<Version>,
  config: AwsBuilderConfig,
  update: &mut Update,
) -> anyhow::Result<(PeripheryClient, BuildCleanupData)> {
  let start_create_ts = komodo_timestamp();

  let version = version.map(|v| format!("-v{v}")).unwrap_or_default();
  let instance_name = format!("BUILDER-{resource_name}{version}");
  let Ec2Instance { instance_id, ip } =
    launch_ec2_instance(&instance_name, &config).await?;

  let log = Log {
    stage: "Start Build Instance".to_string(),
    success: true,
    stdout: start_aws_builder_log(&instance_id, &ip, &config),
    start_ts: start_create_ts,
    end_ts: komodo_timestamp(),
    ..Default::default()
  };

  update.logs.push(log);

  update_update(update.clone()).await?;

  let protocol = if config.use_https { "wss" } else { "ws" };

  // TODO: Handle ad-hoc (non server) periphery connections. These don't have ids.
  let periphery_address =
    format!("{protocol}://{ip}:{}", config.port);
  let periphery = PeripheryClient::new(
    PeripheryConnectionArgs::from_aws_builder(
      &ObjectId::new().to_hex(),
      &periphery_address,
      &config,
    ),
    config.insecure_tls,
  )
  .await?;

  let start_connect_ts = komodo_timestamp();
  let mut res = Ok(GetVersionResponse {
    version: String::new(),
  });
  for _ in 0..BUILDER_POLL_MAX_TRIES {
    let version = periphery
      .request(api::GetVersion {})
      .await
      .context("failed to reach periphery client on builder");
    if let Ok(GetVersionResponse { version }) = &version {
      let connect_log = Log {
        stage: "build instance connected".to_string(),
        success: true,
        stdout: format!(
          "established contact with periphery on builder\nperiphery version: v{version}"
        ),
        start_ts: start_connect_ts,
        end_ts: komodo_timestamp(),
        ..Default::default()
      };
      update.logs.push(connect_log);
      update_update(update.clone()).await?;
      return Ok((
        periphery,
        BuildCleanupData::Aws {
          instance_id,
          region: config.region,
        },
      ));
    }
    res = version;
    tokio::time::sleep(Duration::from_secs(BUILDER_POLL_RATE_SECS))
      .await;
  }

  // Spawn terminate task in failure case (if loop is passed without return)
  tokio::spawn(async move {
    let _ =
      terminate_ec2_instance_with_retry(config.region, &instance_id)
        .await;
  });

  // Unwrap is safe, only way to get here is after check Ok / early return, so it must be err
  Err(
    res.err().unwrap().context(
      "failed to start usable builder. terminating instance.",
    ),
  )
}

#[instrument(
  "CleanupBuilderInstance",
  skip_all,
  fields(update_id = update.id)
)]
pub async fn cleanup_builder_instance(
  periphery: PeripheryClient,
  cleanup_data: BuildCleanupData,
  update: &mut Update,
) {
  match cleanup_data {
    BuildCleanupData::Server => {
      // Nothing to clean up
    }
    BuildCleanupData::Url => {
      periphery.cleanup().await;
    }
    BuildCleanupData::Aws {
      instance_id,
      region,
    } => {
      periphery.cleanup().await;
      let _instance_id = instance_id.clone();
      tokio::spawn(async move {
        let _ =
          terminate_ec2_instance_with_retry(region, &_instance_id)
            .await;
      });
      update.push_simple_log(
        "terminate instance",
        format!("termination queued for instance id {instance_id}"),
      );
    }
  }
}

pub fn start_aws_builder_log(
  instance_id: &str,
  ip: &str,
  config: &AwsBuilderConfig,
) -> String {
  let AwsBuilderConfig {
    ami_id,
    instance_type,
    volume_gb,
    subnet_id,
    assign_public_ip,
    security_group_ids,
    use_public_ip,
    use_https,
    ..
  } = config;

  let readable_sec_group_ids = security_group_ids.join(", ");

  [
    format!("{}: {instance_id}", muted("instance id")),
    format!("{}: {ip}", muted("ip")),
    format!("{}: {ami_id}", muted("ami id")),
    format!("{}: {instance_type}", muted("instance type")),
    format!("{}: {volume_gb} GB", muted("volume size")),
    format!("{}: {subnet_id}", muted("subnet id")),
    format!("{}: {readable_sec_group_ids}", muted("security groups")),
    format!("{}: {assign_public_ip}", muted("assign public ip")),
    format!("{}: {use_public_ip}", muted("use public ip")),
    format!("{}: {use_https}", muted("use https")),
  ]
  .join("\n")
}
