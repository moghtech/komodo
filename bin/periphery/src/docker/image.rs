use anyhow::Context;
use bollard::query_parameters::ListImagesOptions;
use command::run_komodo_standard_command;
use komodo_client::entities::docker::{
  GraphDriverData, HealthConfig, container::ContainerListItem,
  image::*,
};
use serde::Deserialize;

use super::DockerClient;

impl DockerClient {
  pub async fn list_images(
    &self,
    containers: &[ContainerListItem],
  ) -> anyhow::Result<Vec<ImageListItem>> {
    let mut images = self
      .docker
      .list_images(Option::<ListImagesOptions>::None)
      .await?
      .into_iter()
      .map(|image| {
        let in_use = containers.iter().any(|container| {
          container
            .image_id
            .as_ref()
            .map(|id| id == &image.id)
            .unwrap_or_default()
        });
        let name = image
          .repo_tags
          .first()
          .cloned()
          .unwrap_or_else(|| image.id.clone());
        ImageListItem {
          id: image.id,
          parent_id: image.parent_id,
          name,
          tags: image.repo_tags,
          digests: image.repo_digests,
          created: image.created,
          size: image.size,
          in_use,
        }
      })
      .collect::<Vec<_>>();

    images.sort_by(|a, b| {
      a.in_use.cmp(&b.in_use).then_with(|| a.name.cmp(&b.name))
    });

    Ok(images)
  }

  pub async fn inspect_image(
    &self,
    image_name: &str,
  ) -> anyhow::Result<Image> {
    let image = self.docker.inspect_image(image_name).await?;
    Ok(Image {
      id: image.id,
      descriptor: image.descriptor.map(convert_oci_descriptor),
      manifests: image.manifests.map(|manifests| {
        manifests
          .into_iter()
          .map(|manifest| ImageManifestSummary {
            id: manifest.id,
            descriptor: convert_oci_descriptor(manifest.descriptor),
            available: manifest.available,
            size: ImageManifestSummarySize {
              total: manifest.size.total,
              content: manifest.size.content,
            },
            kind: manifest.kind.map(|kind| match kind {
                bollard::config::ImageManifestSummaryKindEnum::EMPTY => ImageManifestSummaryKindEnum::Empty,
                bollard::config::ImageManifestSummaryKindEnum::IMAGE => ImageManifestSummaryKindEnum::Image,
                bollard::config::ImageManifestSummaryKindEnum::ATTESTATION => ImageManifestSummaryKindEnum::Attestation,
                bollard::config::ImageManifestSummaryKindEnum::UNKNOWN => ImageManifestSummaryKindEnum::Unknown,
            }),
            image_data: manifest.image_data.map(|data| {
              ImageManifestSummaryImageData {
                platform: convert_oci_platform(data.platform),
                containers: data.containers,
                size: ImageManifestSummaryImageDataSize { unpacked: data.size.unpacked }
              }
            }),
            attestation_data: manifest.attestation_data.map(|data| ImageManifestSummaryAttestationData { _for: data._for }),
          })
          .collect()
      }),
      repo_tags: image.repo_tags,
      repo_digests: image.repo_digests,
      comment: image.comment,
      created: image.created,
      author: image.author,
      architecture: image.architecture,
      variant: image.variant,
      os: image.os,
      os_version: image.os_version,
      size: image.size,
      graph_driver: image.graph_driver.map(|driver| {
        GraphDriverData {
          name: driver.name,
          data: driver.data,
        }
      }),
      root_fs: image.root_fs.map(|fs| ImageInspectRootFs {
        typ: fs.typ,
        layers: fs.layers.unwrap_or_default(),
      }),
      metadata: image.metadata.map(|metadata| ImageInspectMetadata {
        last_tag_time: metadata.last_tag_time,
      }),
      config: image.config.map(|config| ImageConfig {
        user: config.user,
        exposed_ports: config.exposed_ports,
        env: config.env,
        cmd: config.cmd,
        healthcheck: config.healthcheck.map(|health| HealthConfig {
          test: health.test.unwrap_or_default(),
          interval: health.interval,
          timeout: health.timeout,
          retries: health.retries,
          start_period: health.start_period,
          start_interval: health.start_interval,
        }),
        args_escaped: config.args_escaped,
        volumes: config.volumes,
        working_dir: config.working_dir,
        entrypoint: config.entrypoint,
        on_build: config.on_build,
        labels: config.labels,
        stop_signal: config.stop_signal,
        shell: config.shell,
      }),
    })
  }

  pub async fn image_history(
    &self,
    image_name: &str,
  ) -> anyhow::Result<Vec<ImageHistoryResponseItem>> {
    let res = self
      .docker
      .image_history(image_name)
      .await?
      .into_iter()
      .map(|image| ImageHistoryResponseItem {
        id: image.id,
        created: image.created,
        created_by: image.created_by,
        tags: image.tags,
        size: image.size,
        comment: image.comment,
      })
      .collect();
    Ok(res)
  }
}

fn convert_oci_descriptor(
  descriptor: bollard::config::OciDescriptor,
) -> OciDescriptor {
  OciDescriptor {
    media_type: descriptor.media_type,
    digest: descriptor.digest,
    size: descriptor.size,
    urls: descriptor.urls,
    annotations: descriptor.annotations,
    data: descriptor.data,
    platform: descriptor.platform.map(convert_oci_platform),
    artifact_type: descriptor.artifact_type,
  }
}

fn convert_oci_platform(
  platform: bollard::config::OciPlatform,
) -> OciPlatform {
  OciPlatform {
    architecture: platform.architecture,
    os: platform.os,
    os_version: platform.os_version,
    os_features: platform.os_features,
    variant: platform.variant,
  }
}

/// Private images will require `docker login`
/// for this to work.
pub async fn get_image_digest_from_registry(
  image: &str,
) -> anyhow::Result<String> {
  let command = String::from(
    r#"docker buildx imagetools inspect --format "{{json .Manifest}}" "#,
  ) + image;
  let log = run_komodo_standard_command("", None, command).await;
  if !log.success {
    return Err(anyhow::Error::msg(log.combined()));
  }
  #[derive(Deserialize)]
  struct ImageManifest {
    digest: String,
  }
  let ImageManifest { digest } = serde_json::from_str(&log.stdout)
    .context("Failed to parse image manifest from 'docker buildx imagetools inspect' output")?;
  Ok(digest)
}
