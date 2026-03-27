use std::time::Duration;

use anyhow::{Context, anyhow};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_ec2::{
  Client,
  types::{
    BlockDeviceMapping, EbsBlockDevice,
    InstanceNetworkInterfaceSpecification, InstanceStateChange,
    InstanceStateName, InstanceStatus, InstanceType, ResourceType,
    Tag, TagSpecification,
  },
};
use data_encoding::BASE64_NOPAD;
use komodo_client::entities::{
  ResourceTarget,
  alert::{Alert, AlertData, SeverityLevel},
  builder::AwsBuilderConfig,
  komodo_timestamp,
};

use crate::{alert::send_alerts, config::core_config};

const POLL_RATE_SECS: u64 = 2;
const MAX_POLL_TRIES: usize = 30;

pub struct Ec2Instance {
  pub instance_id: String,
  pub ip: String,
}

/// Provides credentials in the core config file to the AWS client
#[derive(Debug)]
struct CredentialsFromConfig;

impl aws_credential_types::provider::ProvideCredentials
  for CredentialsFromConfig
{
  fn provide_credentials<'a>(
    &'a self,
  ) -> aws_credential_types::provider::future::ProvideCredentials<'a>
  where
    Self: 'a,
  {
    aws_credential_types::provider::future::ProvideCredentials::new(
      async {
        let config = core_config();
        Ok(aws_credential_types::Credentials::new(
          &config.aws.access_key_id,
          &config.aws.secret_access_key,
          None,
          None,
          "komodo-config",
        ))
      },
    )
  }
}

async fn create_ec2_client(region: String) -> Client {
  let region = Region::new(region);
  let config = aws_config::defaults(BehaviorVersion::latest())
    .region(region)
    .credentials_provider(CredentialsFromConfig)
    .load()
    .await;
  Client::new(&config)
}

#[instrument("LaunchEc2Instance")]
pub async fn launch_ec2_instance(
  name: &str,
  config: &AwsBuilderConfig,
) -> anyhow::Result<Ec2Instance> {
  let AwsBuilderConfig {
    region,
    instance_type,
    volume_gb,
    ami_id,
    subnet_id,
    security_group_ids,
    key_pair_name,
    assign_public_ip,
    use_public_ip,
    user_data,
    periphery_public_key: _,
    insecure_tls: _,
    port: _,
    use_https: _,
    git_providers: _,
    docker_registries: _,
    secrets: _,
  } = config;
  let instance_type = handle_unknown_instance_type(
    InstanceType::from(instance_type.as_str()),
  )?;
  let client = create_ec2_client(region.clone()).await;
  let req = client
    .run_instances()
    .image_id(ami_id)
    .instance_type(instance_type)
    .network_interfaces(
      InstanceNetworkInterfaceSpecification::builder()
        .subnet_id(subnet_id)
        .associate_public_ip_address(*assign_public_ip)
        .set_groups(security_group_ids.to_vec().into())
        .device_index(0)
        .build(),
    )
    .key_name(key_pair_name)
    .tag_specifications(
      TagSpecification::builder()
        .tags(Tag::builder().key("Name").value(name).build())
        .resource_type(ResourceType::Instance)
        .build(),
    )
    .block_device_mappings(
      BlockDeviceMapping::builder()
        .set_device_name("/dev/sda1".to_string().into())
        .set_ebs(
          EbsBlockDevice::builder()
            .volume_size(*volume_gb)
            .build()
            .into(),
        )
        .build(),
    )
    .min_count(1)
    .max_count(1)
    .user_data(BASE64_NOPAD.encode(user_data.as_bytes()));

  let res = req
    .send()
    .await
    .context("failed to start builder ec2 instance")?;

  let instance = res
    .instances()
    .first()
    .context("instances array is empty")?;

  let instance_id = instance
    .instance_id()
    .context("instance does not have instance_id")?
    .to_string();

  for _ in 0..MAX_POLL_TRIES {
    let state_name =
      get_ec2_instance_state_name(&client, &instance_id).await?;
    if state_name == Some(InstanceStateName::Running) {
      let ip = if *use_public_ip {
        get_ec2_instance_public_ip(&client, &instance_id).await?
      } else {
        instance
          .private_ip_address()
          .ok_or(anyhow!("instance does not have private ip"))?
          .to_string()
      };
      return Ok(Ec2Instance { instance_id, ip });
    }
    tokio::time::sleep(Duration::from_secs(POLL_RATE_SECS)).await;
  }
  Err(anyhow!("instance not running after polling"))
}

const MAX_TERMINATION_TRIES: usize = 5;
const TERMINATION_WAIT_SECS: u64 = 15;

#[instrument("TerminateEc2Instance")]
pub async fn terminate_ec2_instance_with_retry(
  region: String,
  instance_id: &str,
) -> anyhow::Result<InstanceStateChange> {
  let client = create_ec2_client(region).await;
  for i in 0..MAX_TERMINATION_TRIES {
    match terminate_ec2_instance_inner(&client, instance_id).await {
      Ok(res) => {
        info!("instance {instance_id} successfully terminated.");
        return Ok(res);
      }
      Err(e) => {
        if i == MAX_TERMINATION_TRIES - 1 {
          error!("failed to terminate aws instance {instance_id}.");
          let alert = Alert {
            id: Default::default(),
            ts: komodo_timestamp(),
            resolved: false,
            level: SeverityLevel::Critical,
            target: ResourceTarget::system(),
            data: AlertData::AwsBuilderTerminationFailed {
              instance_id: instance_id.to_string(),
              message: format!("{e:#}"),
            },
            resolved_ts: None,
          };
          send_alerts(&[alert]).await;
          return Err(e);
        }
        tokio::time::sleep(Duration::from_secs(
          TERMINATION_WAIT_SECS,
        ))
        .await;
      }
    }
  }
  unreachable!()
}

#[instrument("TerminateEc2InstanceInner", skip_all)]
async fn terminate_ec2_instance_inner(
  client: &Client,
  instance_id: &str,
) -> anyhow::Result<InstanceStateChange> {
  let res = client
    .terminate_instances()
    .instance_ids(instance_id)
    .send()
    .await
    .context("failed to terminate instance from aws")?
    .terminating_instances()
    .first()
    .context("terminating instances is empty")?
    .to_owned();
  Ok(res)
}

/// Automatically retries 5 times, waiting 2 sec in between
async fn get_ec2_instance_status(
  client: &Client,
  instance_id: &str,
) -> anyhow::Result<Option<InstanceStatus>> {
  let mut try_count = 1;
  loop {
    match async {
      anyhow::Ok(
        client
          .describe_instance_status()
          .instance_ids(instance_id)
          .send()
          .await
          .context("failed to describe instance status from aws")?
          .instance_statuses()
          .first()
          .cloned(),
      )
    }
    .await
    {
      Ok(res) => return Ok(res),
      Err(e) if try_count > 4 => return Err(e),
      Err(_) => {
        tokio::time::sleep(Duration::from_secs(2)).await;
        try_count += 1;
      }
    }
  }
}

async fn get_ec2_instance_state_name(
  client: &Client,
  instance_id: &str,
) -> anyhow::Result<Option<InstanceStateName>> {
  let status = get_ec2_instance_status(client, instance_id).await?;
  if status.is_none() {
    return Ok(None);
  }
  let state = status
    .unwrap()
    .instance_state()
    .ok_or(anyhow!("instance state is None"))?
    .name()
    .ok_or(anyhow!("instance state name is None"))?
    .to_owned();
  Ok(Some(state))
}

/// Automatically retries 5 times, waiting 2 sec in between
async fn get_ec2_instance_public_ip(
  client: &Client,
  instance_id: &str,
) -> anyhow::Result<String> {
  let mut try_count = 1;
  loop {
    match async {
      anyhow::Ok(
        client
          .describe_instances()
          .instance_ids(instance_id)
          .send()
          .await
          .context("failed to describe instances from aws")?
          .reservations()
          .first()
          .context("instance reservations is empty")?
          .instances()
          .first()
          .context("instances is empty")?
          .public_ip_address()
          .context("instance has no public ip")?
          .to_string(),
      )
    }
    .await
    {
      Ok(res) => return Ok(res),
      Err(e) if try_count > 4 => return Err(e),
      Err(_) => {
        tokio::time::sleep(Duration::from_secs(2)).await;
        try_count += 1;
      }
    }
  }
}

fn handle_unknown_instance_type(
  instance_type: InstanceType,
) -> anyhow::Result<InstanceType> {
  match instance_type {
    InstanceType::A12xlarge
    | InstanceType::A14xlarge
    | InstanceType::A1Large
    | InstanceType::A1Medium
    | InstanceType::A1Metal
    | InstanceType::A1Xlarge
    | InstanceType::C1Medium
    | InstanceType::C1Xlarge
    | InstanceType::C32xlarge
    | InstanceType::C34xlarge
    | InstanceType::C38xlarge
    | InstanceType::C3Large
    | InstanceType::C3Xlarge
    | InstanceType::C42xlarge
    | InstanceType::C44xlarge
    | InstanceType::C48xlarge
    | InstanceType::C4Large
    | InstanceType::C4Xlarge
    | InstanceType::C512xlarge
    | InstanceType::C518xlarge
    | InstanceType::C524xlarge
    | InstanceType::C52xlarge
    | InstanceType::C54xlarge
    | InstanceType::C59xlarge
    | InstanceType::C5Large
    | InstanceType::C5Metal
    | InstanceType::C5Xlarge
    | InstanceType::C5a12xlarge
    | InstanceType::C5a16xlarge
    | InstanceType::C5a24xlarge
    | InstanceType::C5a2xlarge
    | InstanceType::C5a4xlarge
    | InstanceType::C5a8xlarge
    | InstanceType::C5aLarge
    | InstanceType::C5aXlarge
    | InstanceType::C5ad12xlarge
    | InstanceType::C5ad16xlarge
    | InstanceType::C5ad24xlarge
    | InstanceType::C5ad2xlarge
    | InstanceType::C5ad4xlarge
    | InstanceType::C5ad8xlarge
    | InstanceType::C5adLarge
    | InstanceType::C5adXlarge
    | InstanceType::C5d12xlarge
    | InstanceType::C5d18xlarge
    | InstanceType::C5d24xlarge
    | InstanceType::C5d2xlarge
    | InstanceType::C5d4xlarge
    | InstanceType::C5d9xlarge
    | InstanceType::C5dLarge
    | InstanceType::C5dMetal
    | InstanceType::C5dXlarge
    | InstanceType::C5n18xlarge
    | InstanceType::C5n2xlarge
    | InstanceType::C5n4xlarge
    | InstanceType::C5n9xlarge
    | InstanceType::C5nLarge
    | InstanceType::C5nMetal
    | InstanceType::C5nXlarge
    | InstanceType::C6a12xlarge
    | InstanceType::C6a16xlarge
    | InstanceType::C6a24xlarge
    | InstanceType::C6a2xlarge
    | InstanceType::C6a32xlarge
    | InstanceType::C6a48xlarge
    | InstanceType::C6a4xlarge
    | InstanceType::C6a8xlarge
    | InstanceType::C6aLarge
    | InstanceType::C6aMetal
    | InstanceType::C6aXlarge
    | InstanceType::C6g12xlarge
    | InstanceType::C6g16xlarge
    | InstanceType::C6g2xlarge
    | InstanceType::C6g4xlarge
    | InstanceType::C6g8xlarge
    | InstanceType::C6gLarge
    | InstanceType::C6gMedium
    | InstanceType::C6gMetal
    | InstanceType::C6gXlarge
    | InstanceType::C6gd12xlarge
    | InstanceType::C6gd16xlarge
    | InstanceType::C6gd2xlarge
    | InstanceType::C6gd4xlarge
    | InstanceType::C6gd8xlarge
    | InstanceType::C6gdLarge
    | InstanceType::C6gdMedium
    | InstanceType::C6gdMetal
    | InstanceType::C6gdXlarge
    | InstanceType::C6gn12xlarge
    | InstanceType::C6gn16xlarge
    | InstanceType::C6gn2xlarge
    | InstanceType::C6gn4xlarge
    | InstanceType::C6gn8xlarge
    | InstanceType::C6gnLarge
    | InstanceType::C6gnMedium
    | InstanceType::C6gnXlarge
    | InstanceType::C6i12xlarge
    | InstanceType::C6i16xlarge
    | InstanceType::C6i24xlarge
    | InstanceType::C6i2xlarge
    | InstanceType::C6i32xlarge
    | InstanceType::C6i4xlarge
    | InstanceType::C6i8xlarge
    | InstanceType::C6iLarge
    | InstanceType::C6iMetal
    | InstanceType::C6iXlarge
    | InstanceType::C6id12xlarge
    | InstanceType::C6id16xlarge
    | InstanceType::C6id24xlarge
    | InstanceType::C6id2xlarge
    | InstanceType::C6id32xlarge
    | InstanceType::C6id4xlarge
    | InstanceType::C6id8xlarge
    | InstanceType::C6idLarge
    | InstanceType::C6idMetal
    | InstanceType::C6idXlarge
    | InstanceType::C6in12xlarge
    | InstanceType::C6in16xlarge
    | InstanceType::C6in24xlarge
    | InstanceType::C6in2xlarge
    | InstanceType::C6in32xlarge
    | InstanceType::C6in4xlarge
    | InstanceType::C6in8xlarge
    | InstanceType::C6inLarge
    | InstanceType::C6inMetal
    | InstanceType::C6inXlarge
    | InstanceType::C7a12xlarge
    | InstanceType::C7a16xlarge
    | InstanceType::C7a24xlarge
    | InstanceType::C7a2xlarge
    | InstanceType::C7a32xlarge
    | InstanceType::C7a48xlarge
    | InstanceType::C7a4xlarge
    | InstanceType::C7a8xlarge
    | InstanceType::C7aLarge
    | InstanceType::C7aMedium
    | InstanceType::C7aMetal48xl
    | InstanceType::C7aXlarge
    | InstanceType::C7g12xlarge
    | InstanceType::C7g16xlarge
    | InstanceType::C7g2xlarge
    | InstanceType::C7g4xlarge
    | InstanceType::C7g8xlarge
    | InstanceType::C7gLarge
    | InstanceType::C7gMedium
    | InstanceType::C7gMetal
    | InstanceType::C7gXlarge
    | InstanceType::C7gd12xlarge
    | InstanceType::C7gd16xlarge
    | InstanceType::C7gd2xlarge
    | InstanceType::C7gd4xlarge
    | InstanceType::C7gd8xlarge
    | InstanceType::C7gdLarge
    | InstanceType::C7gdMedium
    | InstanceType::C7gdXlarge
    | InstanceType::C7gn12xlarge
    | InstanceType::C7gn16xlarge
    | InstanceType::C7gn2xlarge
    | InstanceType::C7gn4xlarge
    | InstanceType::C7gn8xlarge
    | InstanceType::C7gnLarge
    | InstanceType::C7gnMedium
    | InstanceType::C7gnXlarge
    | InstanceType::C7i12xlarge
    | InstanceType::C7i16xlarge
    | InstanceType::C7i24xlarge
    | InstanceType::C7i2xlarge
    | InstanceType::C7i48xlarge
    | InstanceType::C7i4xlarge
    | InstanceType::C7i8xlarge
    | InstanceType::C7iLarge
    | InstanceType::C7iMetal24xl
    | InstanceType::C7iMetal48xl
    | InstanceType::C7iXlarge
    | InstanceType::Cc14xlarge
    | InstanceType::Cc28xlarge
    | InstanceType::Cg14xlarge
    | InstanceType::Cr18xlarge
    | InstanceType::D22xlarge
    | InstanceType::D24xlarge
    | InstanceType::D28xlarge
    | InstanceType::D2Xlarge
    | InstanceType::D32xlarge
    | InstanceType::D34xlarge
    | InstanceType::D38xlarge
    | InstanceType::D3Xlarge
    | InstanceType::D3en12xlarge
    | InstanceType::D3en2xlarge
    | InstanceType::D3en4xlarge
    | InstanceType::D3en6xlarge
    | InstanceType::D3en8xlarge
    | InstanceType::D3enXlarge
    | InstanceType::Dl124xlarge
    | InstanceType::Dl2q24xlarge
    | InstanceType::F116xlarge
    | InstanceType::F12xlarge
    | InstanceType::F14xlarge
    | InstanceType::G22xlarge
    | InstanceType::G28xlarge
    | InstanceType::G316xlarge
    | InstanceType::G34xlarge
    | InstanceType::G38xlarge
    | InstanceType::G3sXlarge
    | InstanceType::G4ad16xlarge
    | InstanceType::G4ad2xlarge
    | InstanceType::G4ad4xlarge
    | InstanceType::G4ad8xlarge
    | InstanceType::G4adXlarge
    | InstanceType::G4dn12xlarge
    | InstanceType::G4dn16xlarge
    | InstanceType::G4dn2xlarge
    | InstanceType::G4dn4xlarge
    | InstanceType::G4dn8xlarge
    | InstanceType::G4dnMetal
    | InstanceType::G4dnXlarge
    | InstanceType::G512xlarge
    | InstanceType::G516xlarge
    | InstanceType::G524xlarge
    | InstanceType::G52xlarge
    | InstanceType::G548xlarge
    | InstanceType::G54xlarge
    | InstanceType::G58xlarge
    | InstanceType::G5Xlarge
    | InstanceType::G5g16xlarge
    | InstanceType::G5g2xlarge
    | InstanceType::G5g4xlarge
    | InstanceType::G5g8xlarge
    | InstanceType::G5gMetal
    | InstanceType::G5gXlarge
    | InstanceType::H116xlarge
    | InstanceType::H12xlarge
    | InstanceType::H14xlarge
    | InstanceType::H18xlarge
    | InstanceType::Hi14xlarge
    | InstanceType::Hpc6a48xlarge
    | InstanceType::Hpc6id32xlarge
    | InstanceType::Hpc7a12xlarge
    | InstanceType::Hpc7a24xlarge
    | InstanceType::Hpc7a48xlarge
    | InstanceType::Hpc7a96xlarge
    | InstanceType::Hpc7g16xlarge
    | InstanceType::Hpc7g4xlarge
    | InstanceType::Hpc7g8xlarge
    | InstanceType::Hs18xlarge
    | InstanceType::I22xlarge
    | InstanceType::I24xlarge
    | InstanceType::I28xlarge
    | InstanceType::I2Xlarge
    | InstanceType::I316xlarge
    | InstanceType::I32xlarge
    | InstanceType::I34xlarge
    | InstanceType::I38xlarge
    | InstanceType::I3Large
    | InstanceType::I3Metal
    | InstanceType::I3Xlarge
    | InstanceType::I3en12xlarge
    | InstanceType::I3en24xlarge
    | InstanceType::I3en2xlarge
    | InstanceType::I3en3xlarge
    | InstanceType::I3en6xlarge
    | InstanceType::I3enLarge
    | InstanceType::I3enMetal
    | InstanceType::I3enXlarge
    | InstanceType::I4g16xlarge
    | InstanceType::I4g2xlarge
    | InstanceType::I4g4xlarge
    | InstanceType::I4g8xlarge
    | InstanceType::I4gLarge
    | InstanceType::I4gXlarge
    | InstanceType::I4i12xlarge
    | InstanceType::I4i16xlarge
    | InstanceType::I4i24xlarge
    | InstanceType::I4i2xlarge
    | InstanceType::I4i32xlarge
    | InstanceType::I4i4xlarge
    | InstanceType::I4i8xlarge
    | InstanceType::I4iLarge
    | InstanceType::I4iMetal
    | InstanceType::I4iXlarge
    | InstanceType::Im4gn16xlarge
    | InstanceType::Im4gn2xlarge
    | InstanceType::Im4gn4xlarge
    | InstanceType::Im4gn8xlarge
    | InstanceType::Im4gnLarge
    | InstanceType::Im4gnXlarge
    | InstanceType::Inf124xlarge
    | InstanceType::Inf12xlarge
    | InstanceType::Inf16xlarge
    | InstanceType::Inf1Xlarge
    | InstanceType::Inf224xlarge
    | InstanceType::Inf248xlarge
    | InstanceType::Inf28xlarge
    | InstanceType::Inf2Xlarge
    | InstanceType::Is4gen2xlarge
    | InstanceType::Is4gen4xlarge
    | InstanceType::Is4gen8xlarge
    | InstanceType::Is4genLarge
    | InstanceType::Is4genMedium
    | InstanceType::Is4genXlarge
    | InstanceType::M1Large
    | InstanceType::M1Medium
    | InstanceType::M1Small
    | InstanceType::M1Xlarge
    | InstanceType::M22xlarge
    | InstanceType::M24xlarge
    | InstanceType::M2Xlarge
    | InstanceType::M32xlarge
    | InstanceType::M3Large
    | InstanceType::M3Medium
    | InstanceType::M3Xlarge
    | InstanceType::M410xlarge
    | InstanceType::M416xlarge
    | InstanceType::M42xlarge
    | InstanceType::M44xlarge
    | InstanceType::M4Large
    | InstanceType::M4Xlarge
    | InstanceType::M512xlarge
    | InstanceType::M516xlarge
    | InstanceType::M524xlarge
    | InstanceType::M52xlarge
    | InstanceType::M54xlarge
    | InstanceType::M58xlarge
    | InstanceType::M5Large
    | InstanceType::M5Metal
    | InstanceType::M5Xlarge
    | InstanceType::M5a12xlarge
    | InstanceType::M5a16xlarge
    | InstanceType::M5a24xlarge
    | InstanceType::M5a2xlarge
    | InstanceType::M5a4xlarge
    | InstanceType::M5a8xlarge
    | InstanceType::M5aLarge
    | InstanceType::M5aXlarge
    | InstanceType::M5ad12xlarge
    | InstanceType::M5ad16xlarge
    | InstanceType::M5ad24xlarge
    | InstanceType::M5ad2xlarge
    | InstanceType::M5ad4xlarge
    | InstanceType::M5ad8xlarge
    | InstanceType::M5adLarge
    | InstanceType::M5adXlarge
    | InstanceType::M5d12xlarge
    | InstanceType::M5d16xlarge
    | InstanceType::M5d24xlarge
    | InstanceType::M5d2xlarge
    | InstanceType::M5d4xlarge
    | InstanceType::M5d8xlarge
    | InstanceType::M5dLarge
    | InstanceType::M5dMetal
    | InstanceType::M5dXlarge
    | InstanceType::M5dn12xlarge
    | InstanceType::M5dn16xlarge
    | InstanceType::M5dn24xlarge
    | InstanceType::M5dn2xlarge
    | InstanceType::M5dn4xlarge
    | InstanceType::M5dn8xlarge
    | InstanceType::M5dnLarge
    | InstanceType::M5dnMetal
    | InstanceType::M5dnXlarge
    | InstanceType::M5n12xlarge
    | InstanceType::M5n16xlarge
    | InstanceType::M5n24xlarge
    | InstanceType::M5n2xlarge
    | InstanceType::M5n4xlarge
    | InstanceType::M5n8xlarge
    | InstanceType::M5nLarge
    | InstanceType::M5nMetal
    | InstanceType::M5nXlarge
    | InstanceType::M5zn12xlarge
    | InstanceType::M5zn2xlarge
    | InstanceType::M5zn3xlarge
    | InstanceType::M5zn6xlarge
    | InstanceType::M5znLarge
    | InstanceType::M5znMetal
    | InstanceType::M5znXlarge
    | InstanceType::M6a12xlarge
    | InstanceType::M6a16xlarge
    | InstanceType::M6a24xlarge
    | InstanceType::M6a2xlarge
    | InstanceType::M6a32xlarge
    | InstanceType::M6a48xlarge
    | InstanceType::M6a4xlarge
    | InstanceType::M6a8xlarge
    | InstanceType::M6aLarge
    | InstanceType::M6aMetal
    | InstanceType::M6aXlarge
    | InstanceType::M6g12xlarge
    | InstanceType::M6g16xlarge
    | InstanceType::M6g2xlarge
    | InstanceType::M6g4xlarge
    | InstanceType::M6g8xlarge
    | InstanceType::M6gLarge
    | InstanceType::M6gMedium
    | InstanceType::M6gMetal
    | InstanceType::M6gXlarge
    | InstanceType::M6gd12xlarge
    | InstanceType::M6gd16xlarge
    | InstanceType::M6gd2xlarge
    | InstanceType::M6gd4xlarge
    | InstanceType::M6gd8xlarge
    | InstanceType::M6gdLarge
    | InstanceType::M6gdMedium
    | InstanceType::M6gdMetal
    | InstanceType::M6gdXlarge
    | InstanceType::M6i12xlarge
    | InstanceType::M6i16xlarge
    | InstanceType::M6i24xlarge
    | InstanceType::M6i2xlarge
    | InstanceType::M6i32xlarge
    | InstanceType::M6i4xlarge
    | InstanceType::M6i8xlarge
    | InstanceType::M6iLarge
    | InstanceType::M6iMetal
    | InstanceType::M6iXlarge
    | InstanceType::M6id12xlarge
    | InstanceType::M6id16xlarge
    | InstanceType::M6id24xlarge
    | InstanceType::M6id2xlarge
    | InstanceType::M6id32xlarge
    | InstanceType::M6id4xlarge
    | InstanceType::M6id8xlarge
    | InstanceType::M6idLarge
    | InstanceType::M6idMetal
    | InstanceType::M6idXlarge
    | InstanceType::M6idn12xlarge
    | InstanceType::M6idn16xlarge
    | InstanceType::M6idn24xlarge
    | InstanceType::M6idn2xlarge
    | InstanceType::M6idn32xlarge
    | InstanceType::M6idn4xlarge
    | InstanceType::M6idn8xlarge
    | InstanceType::M6idnLarge
    | InstanceType::M6idnMetal
    | InstanceType::M6idnXlarge
    | InstanceType::M6in12xlarge
    | InstanceType::M6in16xlarge
    | InstanceType::M6in24xlarge
    | InstanceType::M6in2xlarge
    | InstanceType::M6in32xlarge
    | InstanceType::M6in4xlarge
    | InstanceType::M6in8xlarge
    | InstanceType::M6inLarge
    | InstanceType::M6inMetal
    | InstanceType::M6inXlarge
    | InstanceType::M7a12xlarge
    | InstanceType::M7a16xlarge
    | InstanceType::M7a24xlarge
    | InstanceType::M7a2xlarge
    | InstanceType::M7a32xlarge
    | InstanceType::M7a48xlarge
    | InstanceType::M7a4xlarge
    | InstanceType::M7a8xlarge
    | InstanceType::M7aLarge
    | InstanceType::M7aMedium
    | InstanceType::M7aMetal48xl
    | InstanceType::M7aXlarge
    | InstanceType::M7g12xlarge
    | InstanceType::M7g16xlarge
    | InstanceType::M7g2xlarge
    | InstanceType::M7g4xlarge
    | InstanceType::M7g8xlarge
    | InstanceType::M7gLarge
    | InstanceType::M7gMedium
    | InstanceType::M7gMetal
    | InstanceType::M7gXlarge
    | InstanceType::M7gd12xlarge
    | InstanceType::M7gd16xlarge
    | InstanceType::M7gd2xlarge
    | InstanceType::M7gd4xlarge
    | InstanceType::M7gd8xlarge
    | InstanceType::M7gdLarge
    | InstanceType::M7gdMedium
    | InstanceType::M7gdXlarge
    | InstanceType::M7iFlex2xlarge
    | InstanceType::M7iFlex4xlarge
    | InstanceType::M7iFlex8xlarge
    | InstanceType::M7iFlexLarge
    | InstanceType::M7iFlexXlarge
    | InstanceType::M7i12xlarge
    | InstanceType::M7i16xlarge
    | InstanceType::M7i24xlarge
    | InstanceType::M7i2xlarge
    | InstanceType::M7i48xlarge
    | InstanceType::M7i4xlarge
    | InstanceType::M7i8xlarge
    | InstanceType::M7iLarge
    | InstanceType::M7iMetal24xl
    | InstanceType::M7iMetal48xl
    | InstanceType::M7iXlarge
    | InstanceType::Mac1Metal
    | InstanceType::Mac2M2Metal
    | InstanceType::Mac2M2proMetal
    | InstanceType::Mac2Metal
    | InstanceType::P216xlarge
    | InstanceType::P28xlarge
    | InstanceType::P2Xlarge
    | InstanceType::P316xlarge
    | InstanceType::P32xlarge
    | InstanceType::P38xlarge
    | InstanceType::P3dn24xlarge
    | InstanceType::P4d24xlarge
    | InstanceType::P4de24xlarge
    | InstanceType::P548xlarge
    | InstanceType::R32xlarge
    | InstanceType::R34xlarge
    | InstanceType::R38xlarge
    | InstanceType::R3Large
    | InstanceType::R3Xlarge
    | InstanceType::R416xlarge
    | InstanceType::R42xlarge
    | InstanceType::R44xlarge
    | InstanceType::R48xlarge
    | InstanceType::R4Large
    | InstanceType::R4Xlarge
    | InstanceType::R512xlarge
    | InstanceType::R516xlarge
    | InstanceType::R524xlarge
    | InstanceType::R52xlarge
    | InstanceType::R54xlarge
    | InstanceType::R58xlarge
    | InstanceType::R5Large
    | InstanceType::R5Metal
    | InstanceType::R5Xlarge
    | InstanceType::R5a12xlarge
    | InstanceType::R5a16xlarge
    | InstanceType::R5a24xlarge
    | InstanceType::R5a2xlarge
    | InstanceType::R5a4xlarge
    | InstanceType::R5a8xlarge
    | InstanceType::R5aLarge
    | InstanceType::R5aXlarge
    | InstanceType::R5ad12xlarge
    | InstanceType::R5ad16xlarge
    | InstanceType::R5ad24xlarge
    | InstanceType::R5ad2xlarge
    | InstanceType::R5ad4xlarge
    | InstanceType::R5ad8xlarge
    | InstanceType::R5adLarge
    | InstanceType::R5adXlarge
    | InstanceType::R5b12xlarge
    | InstanceType::R5b16xlarge
    | InstanceType::R5b24xlarge
    | InstanceType::R5b2xlarge
    | InstanceType::R5b4xlarge
    | InstanceType::R5b8xlarge
    | InstanceType::R5bLarge
    | InstanceType::R5bMetal
    | InstanceType::R5bXlarge
    | InstanceType::R5d12xlarge
    | InstanceType::R5d16xlarge
    | InstanceType::R5d24xlarge
    | InstanceType::R5d2xlarge
    | InstanceType::R5d4xlarge
    | InstanceType::R5d8xlarge
    | InstanceType::R5dLarge
    | InstanceType::R5dMetal
    | InstanceType::R5dXlarge
    | InstanceType::R5dn12xlarge
    | InstanceType::R5dn16xlarge
    | InstanceType::R5dn24xlarge
    | InstanceType::R5dn2xlarge
    | InstanceType::R5dn4xlarge
    | InstanceType::R5dn8xlarge
    | InstanceType::R5dnLarge
    | InstanceType::R5dnMetal
    | InstanceType::R5dnXlarge
    | InstanceType::R5n12xlarge
    | InstanceType::R5n16xlarge
    | InstanceType::R5n24xlarge
    | InstanceType::R5n2xlarge
    | InstanceType::R5n4xlarge
    | InstanceType::R5n8xlarge
    | InstanceType::R5nLarge
    | InstanceType::R5nMetal
    | InstanceType::R5nXlarge
    | InstanceType::R6a12xlarge
    | InstanceType::R6a16xlarge
    | InstanceType::R6a24xlarge
    | InstanceType::R6a2xlarge
    | InstanceType::R6a32xlarge
    | InstanceType::R6a48xlarge
    | InstanceType::R6a4xlarge
    | InstanceType::R6a8xlarge
    | InstanceType::R6aLarge
    | InstanceType::R6aMetal
    | InstanceType::R6aXlarge
    | InstanceType::R6g12xlarge
    | InstanceType::R6g16xlarge
    | InstanceType::R6g2xlarge
    | InstanceType::R6g4xlarge
    | InstanceType::R6g8xlarge
    | InstanceType::R6gLarge
    | InstanceType::R6gMedium
    | InstanceType::R6gMetal
    | InstanceType::R6gXlarge
    | InstanceType::R6gd12xlarge
    | InstanceType::R6gd16xlarge
    | InstanceType::R6gd2xlarge
    | InstanceType::R6gd4xlarge
    | InstanceType::R6gd8xlarge
    | InstanceType::R6gdLarge
    | InstanceType::R6gdMedium
    | InstanceType::R6gdMetal
    | InstanceType::R6gdXlarge
    | InstanceType::R6i12xlarge
    | InstanceType::R6i16xlarge
    | InstanceType::R6i24xlarge
    | InstanceType::R6i2xlarge
    | InstanceType::R6i32xlarge
    | InstanceType::R6i4xlarge
    | InstanceType::R6i8xlarge
    | InstanceType::R6iLarge
    | InstanceType::R6iMetal
    | InstanceType::R6iXlarge
    | InstanceType::R6id12xlarge
    | InstanceType::R6id16xlarge
    | InstanceType::R6id24xlarge
    | InstanceType::R6id2xlarge
    | InstanceType::R6id32xlarge
    | InstanceType::R6id4xlarge
    | InstanceType::R6id8xlarge
    | InstanceType::R6idLarge
    | InstanceType::R6idMetal
    | InstanceType::R6idXlarge
    | InstanceType::R6idn12xlarge
    | InstanceType::R6idn16xlarge
    | InstanceType::R6idn24xlarge
    | InstanceType::R6idn2xlarge
    | InstanceType::R6idn32xlarge
    | InstanceType::R6idn4xlarge
    | InstanceType::R6idn8xlarge
    | InstanceType::R6idnLarge
    | InstanceType::R6idnMetal
    | InstanceType::R6idnXlarge
    | InstanceType::R6in12xlarge
    | InstanceType::R6in16xlarge
    | InstanceType::R6in24xlarge
    | InstanceType::R6in2xlarge
    | InstanceType::R6in32xlarge
    | InstanceType::R6in4xlarge
    | InstanceType::R6in8xlarge
    | InstanceType::R6inLarge
    | InstanceType::R6inMetal
    | InstanceType::R6inXlarge
    | InstanceType::R7a12xlarge
    | InstanceType::R7a16xlarge
    | InstanceType::R7a24xlarge
    | InstanceType::R7a2xlarge
    | InstanceType::R7a32xlarge
    | InstanceType::R7a48xlarge
    | InstanceType::R7a4xlarge
    | InstanceType::R7a8xlarge
    | InstanceType::R7aLarge
    | InstanceType::R7aMedium
    | InstanceType::R7aMetal48xl
    | InstanceType::R7aXlarge
    | InstanceType::R7g12xlarge
    | InstanceType::R7g16xlarge
    | InstanceType::R7g2xlarge
    | InstanceType::R7g4xlarge
    | InstanceType::R7g8xlarge
    | InstanceType::R7gLarge
    | InstanceType::R7gMedium
    | InstanceType::R7gMetal
    | InstanceType::R7gXlarge
    | InstanceType::R7gd12xlarge
    | InstanceType::R7gd16xlarge
    | InstanceType::R7gd2xlarge
    | InstanceType::R7gd4xlarge
    | InstanceType::R7gd8xlarge
    | InstanceType::R7gdLarge
    | InstanceType::R7gdMedium
    | InstanceType::R7gdXlarge
    | InstanceType::R7i12xlarge
    | InstanceType::R7i16xlarge
    | InstanceType::R7i24xlarge
    | InstanceType::R7i2xlarge
    | InstanceType::R7i48xlarge
    | InstanceType::R7i4xlarge
    | InstanceType::R7i8xlarge
    | InstanceType::R7iLarge
    | InstanceType::R7iMetal24xl
    | InstanceType::R7iMetal48xl
    | InstanceType::R7iXlarge
    | InstanceType::R7iz12xlarge
    | InstanceType::R7iz16xlarge
    | InstanceType::R7iz2xlarge
    | InstanceType::R7iz32xlarge
    | InstanceType::R7iz4xlarge
    | InstanceType::R7iz8xlarge
    | InstanceType::R7izLarge
    | InstanceType::R7izXlarge
    | InstanceType::T1Micro
    | InstanceType::T22xlarge
    | InstanceType::T2Large
    | InstanceType::T2Medium
    | InstanceType::T2Micro
    | InstanceType::T2Nano
    | InstanceType::T2Small
    | InstanceType::T2Xlarge
    | InstanceType::T32xlarge
    | InstanceType::T3Large
    | InstanceType::T3Medium
    | InstanceType::T3Micro
    | InstanceType::T3Nano
    | InstanceType::T3Small
    | InstanceType::T3Xlarge
    | InstanceType::T3a2xlarge
    | InstanceType::T3aLarge
    | InstanceType::T3aMedium
    | InstanceType::T3aMicro
    | InstanceType::T3aNano
    | InstanceType::T3aSmall
    | InstanceType::T3aXlarge
    | InstanceType::T4g2xlarge
    | InstanceType::T4gLarge
    | InstanceType::T4gMedium
    | InstanceType::T4gMicro
    | InstanceType::T4gNano
    | InstanceType::T4gSmall
    | InstanceType::T4gXlarge
    | InstanceType::Trn12xlarge
    | InstanceType::Trn132xlarge
    | InstanceType::Trn1n32xlarge
    | InstanceType::U12tb1112xlarge
    | InstanceType::U12tb1Metal
    | InstanceType::U18tb1112xlarge
    | InstanceType::U18tb1Metal
    | InstanceType::U24tb1112xlarge
    | InstanceType::U24tb1Metal
    | InstanceType::U3tb156xlarge
    | InstanceType::U6tb1112xlarge
    | InstanceType::U6tb156xlarge
    | InstanceType::U6tb1Metal
    | InstanceType::U9tb1112xlarge
    | InstanceType::U9tb1Metal
    | InstanceType::Vt124xlarge
    | InstanceType::Vt13xlarge
    | InstanceType::Vt16xlarge
    | InstanceType::X116xlarge
    | InstanceType::X132xlarge
    | InstanceType::X1e16xlarge
    | InstanceType::X1e2xlarge
    | InstanceType::X1e32xlarge
    | InstanceType::X1e4xlarge
    | InstanceType::X1e8xlarge
    | InstanceType::X1eXlarge
    | InstanceType::X2gd12xlarge
    | InstanceType::X2gd16xlarge
    | InstanceType::X2gd2xlarge
    | InstanceType::X2gd4xlarge
    | InstanceType::X2gd8xlarge
    | InstanceType::X2gdLarge
    | InstanceType::X2gdMedium
    | InstanceType::X2gdMetal
    | InstanceType::X2gdXlarge
    | InstanceType::X2idn16xlarge
    | InstanceType::X2idn24xlarge
    | InstanceType::X2idn32xlarge
    | InstanceType::X2idnMetal
    | InstanceType::X2iedn16xlarge
    | InstanceType::X2iedn24xlarge
    | InstanceType::X2iedn2xlarge
    | InstanceType::X2iedn32xlarge
    | InstanceType::X2iedn4xlarge
    | InstanceType::X2iedn8xlarge
    | InstanceType::X2iednMetal
    | InstanceType::X2iednXlarge
    | InstanceType::X2iezn12xlarge
    | InstanceType::X2iezn2xlarge
    | InstanceType::X2iezn4xlarge
    | InstanceType::X2iezn6xlarge
    | InstanceType::X2iezn8xlarge
    | InstanceType::X2ieznMetal
    | InstanceType::Z1d12xlarge
    | InstanceType::Z1d2xlarge
    | InstanceType::Z1d3xlarge
    | InstanceType::Z1d6xlarge
    | InstanceType::Z1dLarge
    | InstanceType::Z1dMetal
    | InstanceType::Z1dXlarge
    | InstanceType::C7gdMetal
    | InstanceType::C7gnMetal
    | InstanceType::C7iFlex2xlarge
    | InstanceType::C7iFlex4xlarge
    | InstanceType::C7iFlex8xlarge
    | InstanceType::C7iFlexLarge
    | InstanceType::C7iFlexXlarge
    | InstanceType::C8g12xlarge
    | InstanceType::C8g16xlarge
    | InstanceType::C8g24xlarge
    | InstanceType::C8g2xlarge
    | InstanceType::C8g48xlarge
    | InstanceType::C8g4xlarge
    | InstanceType::C8g8xlarge
    | InstanceType::C8gLarge
    | InstanceType::C8gMedium
    | InstanceType::C8gMetal24xl
    | InstanceType::C8gMetal48xl
    | InstanceType::C8gXlarge
    | InstanceType::G612xlarge
    | InstanceType::G616xlarge
    | InstanceType::G624xlarge
    | InstanceType::G62xlarge
    | InstanceType::G648xlarge
    | InstanceType::G64xlarge
    | InstanceType::G68xlarge
    | InstanceType::G6Xlarge
    | InstanceType::G6e12xlarge
    | InstanceType::G6e16xlarge
    | InstanceType::G6e24xlarge
    | InstanceType::G6e2xlarge
    | InstanceType::G6e48xlarge
    | InstanceType::G6e4xlarge
    | InstanceType::G6e8xlarge
    | InstanceType::G6eXlarge
    | InstanceType::Gr64xlarge
    | InstanceType::Gr68xlarge
    | InstanceType::M7gdMetal
    | InstanceType::M8g12xlarge
    | InstanceType::M8g16xlarge
    | InstanceType::M8g24xlarge
    | InstanceType::M8g2xlarge
    | InstanceType::M8g48xlarge
    | InstanceType::M8g4xlarge
    | InstanceType::M8g8xlarge
    | InstanceType::M8gLarge
    | InstanceType::M8gMedium
    | InstanceType::M8gMetal24xl
    | InstanceType::M8gMetal48xl
    | InstanceType::M8gXlarge
    | InstanceType::Mac2M1ultraMetal
    | InstanceType::R7gdMetal
    | InstanceType::R7izMetal16xl
    | InstanceType::R7izMetal32xl
    | InstanceType::R8g12xlarge
    | InstanceType::R8g16xlarge
    | InstanceType::R8g24xlarge
    | InstanceType::R8g2xlarge
    | InstanceType::R8g48xlarge
    | InstanceType::R8g4xlarge
    | InstanceType::R8g8xlarge
    | InstanceType::R8gLarge
    | InstanceType::R8gMedium
    | InstanceType::R8gMetal24xl
    | InstanceType::R8gMetal48xl
    | InstanceType::R8gXlarge
    | InstanceType::U7i12tb224xlarge
    | InstanceType::U7ib12tb224xlarge
    | InstanceType::U7in16tb224xlarge
    | InstanceType::U7in24tb224xlarge
    | InstanceType::U7in32tb224xlarge
    | InstanceType::X8g12xlarge
    | InstanceType::X8g16xlarge
    | InstanceType::X8g24xlarge
    | InstanceType::X8g2xlarge
    | InstanceType::X8g48xlarge
    | InstanceType::X8g4xlarge
    | InstanceType::X8g8xlarge
    | InstanceType::X8gLarge
    | InstanceType::X8gMedium
    | InstanceType::X8gMetal24xl
    | InstanceType::X8gMetal48xl
    | InstanceType::X8gXlarge => Ok(instance_type),
    other => Err(anyhow!("unknown InstanceType: {other:?}")),
  }
}
