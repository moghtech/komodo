use bollard::query_parameters::ListVolumesOptions;
use komodo_client::entities::docker::{
  PortBinding, container::ContainerListItem, volume::*,
};

use crate::docker::DockerClient;

impl DockerClient {
  pub async fn list_volumes(
    &self,
    containers: &[ContainerListItem],
  ) -> anyhow::Result<Vec<VolumeListItem>> {
    let mut volumes = self
      .docker
      .list_volumes(Option::<ListVolumesOptions>::None)
      .await?
      .volumes
      .unwrap_or_default()
      .into_iter()
      .map(|volume| {
        let scope = volume
          .scope
          .map(|scope| match scope {
            bollard::secret::VolumeScopeEnum::EMPTY => {
              VolumeScopeEnum::Empty
            }
            bollard::secret::VolumeScopeEnum::LOCAL => {
              VolumeScopeEnum::Local
            }
            bollard::secret::VolumeScopeEnum::GLOBAL => {
              VolumeScopeEnum::Global
            }
          })
          .unwrap_or(VolumeScopeEnum::Empty);
        let in_use = containers.iter().any(|container| {
          container.volumes.iter().any(|name| &volume.name == name)
        });
        VolumeListItem {
          name: volume.name,
          driver: volume.driver,
          mountpoint: volume.mountpoint,
          created: volume.created_at,
          size: volume.usage_data.map(|data| data.size),
          scope,
          in_use,
        }
      })
      .collect::<Vec<_>>();

    volumes.sort_by(|a, b| {
      a.in_use.cmp(&b.in_use).then_with(|| a.name.cmp(&b.name))
    });

    Ok(volumes)
  }

  pub async fn inspect_volume(
    &self,
    volume_name: &str,
  ) -> anyhow::Result<Volume> {
    let volume = self.docker.inspect_volume(volume_name).await?;
    Ok(Volume {
      name: volume.name,
      driver: volume.driver,
      mountpoint: volume.mountpoint,
      created_at: volume.created_at,
      status: volume.status.unwrap_or_default().into_keys().map(|k| (k, Default::default())).collect(),
      labels: volume.labels,
      scope: volume
        .scope
        .map(|scope| match scope {
          bollard::secret::VolumeScopeEnum::EMPTY => {
            VolumeScopeEnum::Empty
          }
          bollard::secret::VolumeScopeEnum::LOCAL => {
            VolumeScopeEnum::Local
          }
          bollard::secret::VolumeScopeEnum::GLOBAL => {
            VolumeScopeEnum::Global
          }
        })
        .unwrap_or_default(),
      cluster_volume: volume.cluster_volume.map(|volume| {
        ClusterVolume {
          id: volume.id,
          version: volume.version.map(super::convert_object_version),
          created_at: volume.created_at,
          updated_at: volume.updated_at,
          spec: volume.spec.map(|spec| ClusterVolumeSpec {
            group: spec.group,
            access_mode: spec.access_mode.map(|mode| {
              ClusterVolumeSpecAccessMode {
                scope: mode.scope.map(|scope| match scope {
                  bollard::secret::ClusterVolumeSpecAccessModeScopeEnum::EMPTY => ClusterVolumeSpecAccessModeScopeEnum::Empty,
                  bollard::secret::ClusterVolumeSpecAccessModeScopeEnum::SINGLE => ClusterVolumeSpecAccessModeScopeEnum::Single,
                  bollard::secret::ClusterVolumeSpecAccessModeScopeEnum::MULTI => ClusterVolumeSpecAccessModeScopeEnum::Multi,
                }).unwrap_or_default(),
                sharing: mode.sharing.map(|sharing| match sharing {
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::EMPTY => ClusterVolumeSpecAccessModeSharingEnum::Empty,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::NONE => ClusterVolumeSpecAccessModeSharingEnum::None,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::READONLY => ClusterVolumeSpecAccessModeSharingEnum::Readonly,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::ONEWRITER => ClusterVolumeSpecAccessModeSharingEnum::Onewriter,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::ALL => ClusterVolumeSpecAccessModeSharingEnum::All,
                }).unwrap_or_default(),
                secrets: mode.secrets.unwrap_or_default().into_iter().map(|secret| ClusterVolumeSpecAccessModeSecrets {
                  key: secret.key,
                  secret: secret.secret,
                }).collect(),
                accessibility_requirements: mode
                  .accessibility_requirements.map(|req| ClusterVolumeSpecAccessModeAccessibilityRequirements {
                    requisite: req.requisite.unwrap_or_default().into_iter().map(|map| map.into_iter().map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|p| PortBinding { host_ip: p.host_ip, host_port: p.host_port }).collect())).collect()).collect(),
                    preferred: req.preferred.unwrap_or_default().into_iter().map(|map| map.into_iter().map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|p| PortBinding { host_ip: p.host_ip, host_port: p.host_port }).collect())).collect()).collect(),
                }),
                capacity_range: mode.capacity_range.map(|range| ClusterVolumeSpecAccessModeCapacityRange {
                  required_bytes: range.required_bytes,
                  limit_bytes: range.limit_bytes,
                }),
                availability: mode.availability.map(|availability| match availability {
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::EMPTY => ClusterVolumeSpecAccessModeAvailabilityEnum::Empty,
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::ACTIVE => ClusterVolumeSpecAccessModeAvailabilityEnum::Active,
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::PAUSE => ClusterVolumeSpecAccessModeAvailabilityEnum::Pause,
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::DRAIN => ClusterVolumeSpecAccessModeAvailabilityEnum::Drain,
                }).unwrap_or_default(),
              }
            }),
          }),
          info: volume.info.map(|info| ClusterVolumeInfo {
            capacity_bytes: info.capacity_bytes,
            volume_context: info.volume_context.unwrap_or_default(),
            volume_id: info.volume_id,
            accessible_topology: info.accessible_topology.unwrap_or_default().into_iter().map(|map| map.into_iter().map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|p| PortBinding { host_ip: p.host_ip, host_port: p.host_port }).collect())).collect()).collect(),
          }),
          publish_status: volume
            .publish_status
            .unwrap_or_default()
            .into_iter()
            .map(|status| ClusterVolumePublishStatus {
              node_id: status.node_id,
              state: status.state.map(|state| match state {
                bollard::secret::ClusterVolumePublishStatusStateEnum::EMPTY => ClusterVolumePublishStatusStateEnum::Empty,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PENDING_PUBLISH => ClusterVolumePublishStatusStateEnum::PendingPublish,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PUBLISHED => ClusterVolumePublishStatusStateEnum::Published,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PENDING_NODE_UNPUBLISH => ClusterVolumePublishStatusStateEnum::PendingNodeUnpublish,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PENDING_CONTROLLER_UNPUBLISH => ClusterVolumePublishStatusStateEnum::PendingControllerUnpublish,
              }).unwrap_or_default(),
              publish_context: status.publish_context.unwrap_or_default(),
            })
            .collect(),
        }
      }),
      options: volume.options,
      usage_data: volume.usage_data.map(|data| VolumeUsageData {
        size: data.size,
        ref_count: data.ref_count,
      }),
    })
  }
}
