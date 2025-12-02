use bollard::query_parameters::ListImagesOptions;
use komodo_client::entities::docker::{
  ContainerConfig, GraphDriverData, HealthConfig,
  container::ContainerListItem, image::*,
};

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
        ImageListItem {
          name: image
            .repo_tags
            .into_iter()
            .next()
            .unwrap_or_else(|| image.id.clone()),
          id: image.id,
          parent_id: image.parent_id,
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
      repo_tags: image.repo_tags.unwrap_or_default(),
      repo_digests: image.repo_digests.unwrap_or_default(),
      parent: image.parent,
      comment: image.comment,
      created: image.created,
      docker_version: image.docker_version,
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
      config: image.config.map(|config| ContainerConfig {
        hostname: config.hostname,
        domainname: config.domainname,
        user: config.user,
        attach_stdin: config.attach_stdin,
        attach_stdout: config.attach_stdout,
        attach_stderr: config.attach_stderr,
        exposed_ports: config
          .exposed_ports
          .unwrap_or_default()
          .into_keys()
          .map(|k| (k, Default::default()))
          .collect(),
        tty: config.tty,
        open_stdin: config.open_stdin,
        stdin_once: config.stdin_once,
        env: config.env.unwrap_or_default(),
        cmd: config.cmd.unwrap_or_default(),
        healthcheck: config.healthcheck.map(|health| HealthConfig {
          test: health.test.unwrap_or_default(),
          interval: health.interval,
          timeout: health.timeout,
          retries: health.retries,
          start_period: health.start_period,
          start_interval: health.start_interval,
        }),
        args_escaped: config.args_escaped,
        image: config.image,
        volumes: config
          .volumes
          .unwrap_or_default()
          .into_keys()
          .map(|k| (k, Default::default()))
          .collect(),
        working_dir: config.working_dir,
        entrypoint: config.entrypoint.unwrap_or_default(),
        network_disabled: config.network_disabled,
        mac_address: config.mac_address,
        on_build: config.on_build.unwrap_or_default(),
        labels: config.labels.unwrap_or_default(),
        stop_signal: config.stop_signal,
        stop_timeout: config.stop_timeout,
        shell: config.shell.unwrap_or_default(),
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
