use std::{
  fs,
  path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use formatting::{Color, bold, colored, format_serror, muted};
use komodo_client::entities::{
  sync::SyncFileContents,
  toml::{ResourceToml, ResourcesToml},
  update::Log,
};

pub fn read_resources(
  root_path: &Path,
  resource_path: &[String],
  match_tags: &[String],
  logs: &mut Vec<Log>,
  files: &mut Vec<SyncFileContents>,
  file_errors: &mut Vec<SyncFileContents>,
) -> anyhow::Result<ResourcesToml> {
  let mut resources = ResourcesToml::default();

  for resource_path in resource_path {
    let resource_path = resource_path
      .parse::<PathBuf>()
      .context("Invalid resource path")?;
    let full_path = root_path
      .join(&resource_path)
      .components()
      .collect::<PathBuf>();

    let mut log = format!(
      "{}: reading resources from {full_path:?}",
      muted("INFO")
    );

    if full_path.is_file() {
      if let Err(e) = read_resource_file(
        root_path,
        None,
        &resource_path,
        match_tags,
        &mut resources,
        &mut log,
        files,
      )
      .with_context(|| {
        format!("failed to read resources from {full_path:?}")
      }) {
        file_errors.push(SyncFileContents {
          resource_path: String::new(),
          path: resource_path.display().to_string(),
          contents: format_serror(&e.into()),
        });
        logs.push(Log::error("Read remote resources", log));
      } else {
        logs.push(Log::simple("Read remote resources", log));
      };
    } else if full_path.is_dir() {
      if let Err(e) = read_resources_directory(
        root_path,
        &resource_path,
        &PathBuf::new(),
        match_tags,
        &mut resources,
        &mut log,
        files,
        file_errors,
      )
      .with_context(|| {
        format!("Failed to read resources from {full_path:?}")
      }) {
        file_errors.push(SyncFileContents {
          resource_path: String::new(),
          path: resource_path.display().to_string(),
          contents: format_serror(&e.into()),
        });
        logs.push(Log::error("Read remote resources", log));
      } else {
        logs.push(Log::simple("Read remote resources", log));
      };
    } else if !full_path.exists() {
      file_errors.push(SyncFileContents {
        resource_path: String::new(),
        path: resource_path.display().to_string(),
        contents: format_serror(
          &anyhow!("Initialize the file to proceed.")
            .context(format!("Path {full_path:?} does not exist."))
            .into(),
        ),
      });
      log.push_str(&format!(
        "{}: Resoure path {} does not exist.",
        colored("ERROR", Color::Red),
        bold(resource_path.display())
      ));
      logs.push(Log::error("Read remote resources", log));
    } else {
      log.push_str(&format!(
        "{}: Resoure path {} exists, but is neither a file nor a directory.",
        colored("WARN", Color::Red),
        bold(resource_path.display())
      ));
      logs.push(Log::error("Read remote resources", log));
    }
  }

  Ok(resources)
}

/// Use when incoming resource path is a file.
fn read_resource_file(
  root_path: &Path,
  // relative to root path.
  resource_path: Option<&Path>,
  // relative to resource path if provided, or root path.
  file_path: &Path,
  match_tags: &[String],
  resources: &mut ResourcesToml,
  log: &mut String,
  files: &mut Vec<SyncFileContents>,
) -> anyhow::Result<()> {
  let full_path = if let Some(resource_path) = resource_path {
    root_path.join(resource_path).join(file_path)
  } else {
    root_path.join(file_path)
  };
  if !full_path
    .extension()
    .map(|ext| ext == "toml")
    .unwrap_or_default()
  {
    return Ok(());
  }
  let contents = std::fs::read_to_string(&full_path)
    .context("failed to read file contents")?;

  files.push(SyncFileContents {
    resource_path: resource_path
      .map(|path| path.display().to_string())
      .unwrap_or_default(),
    path: file_path.display().to_string(),
    contents: contents.clone(),
  });
  let more = super::deserialize_resources_toml(&contents)
    .context("failed to parse resource file contents")?;
  log.push('\n');
  let path_for_view =
    if let Some(resource_path) = resource_path.as_ref() {
      resource_path.join(file_path)
    } else {
      file_path.to_path_buf()
    };
  log.push_str(&format!(
    "{}: {} from {}",
    muted("INFO"),
    colored("adding resources", Color::Green),
    colored(path_for_view.display(), Color::Blue)
  ));

  extend_resources(resources, more, match_tags);

  Ok(())
}

/// Reads down into directories.
#[allow(clippy::too_many_arguments)]
fn read_resources_directory(
  root_path: &Path,
  // relative to root path.
  resource_path: &Path,
  // relative to resource path. start as empty path
  curr_path: &Path,
  match_tags: &[String],
  resources: &mut ResourcesToml,
  log: &mut String,
  files: &mut Vec<SyncFileContents>,
  file_errors: &mut Vec<SyncFileContents>,
) -> anyhow::Result<()> {
  let full_resource_path = root_path.join(resource_path);
  let full_path = full_resource_path.join(curr_path);
  let directory = fs::read_dir(&full_path).with_context(|| {
    format!("Failed to read directory contents at {full_path:?}")
  })?;
  for entry in directory.into_iter().flatten() {
    let path = entry.path();
    let curr_path =
      path.strip_prefix(&full_resource_path).unwrap_or(&path);
    if path.is_file() {
      if let Err(e) = read_resource_file(
        root_path,
        Some(resource_path),
        curr_path,
        match_tags,
        resources,
        log,
        files,
      )
      .with_context(|| {
        format!("failed to read resources from {full_path:?}")
      }) {
        file_errors.push(SyncFileContents {
          resource_path: String::new(),
          path: resource_path.display().to_string(),
          contents: format_serror(&e.into()),
        });
      };
    } else if path.is_dir() {
      if let Err(e) = read_resources_directory(
        root_path,
        resource_path,
        curr_path,
        match_tags,
        resources,
        log,
        files,
        file_errors,
      )
      .with_context(|| {
        format!("failed to read resources from {path:?}")
      }) {
        file_errors.push(SyncFileContents {
          resource_path: resource_path.display().to_string(),
          path: curr_path.display().to_string(),
          contents: format_serror(&e.into()),
        });
        log.push('\n');
        log.push_str(&format!(
          "{}: {} from {}",
          colored("ERROR", Color::Red),
          colored("adding resources", Color::Green),
          colored(path.display(), Color::Blue)
        ));
      }
    }
  }
  Ok(())
}

pub fn extend_resources(
  resources: &mut ResourcesToml,
  more: ResourcesToml,
  match_tags: &[String],
) {
  resources
    .servers
    .extend(filter_by_tag(more.servers, match_tags));
  resources
    .stacks
    .extend(filter_by_tag(more.stacks, match_tags));
  resources
    .deployments
    .extend(filter_by_tag(more.deployments, match_tags));
  resources
    .builds
    .extend(filter_by_tag(more.builds, match_tags));
  resources
    .repos
    .extend(filter_by_tag(more.repos, match_tags));
  resources
    .procedures
    .extend(filter_by_tag(more.procedures, match_tags));
  resources
    .actions
    .extend(filter_by_tag(more.actions, match_tags));
  resources
    .alerters
    .extend(filter_by_tag(more.alerters, match_tags));
  resources
    .builders
    .extend(filter_by_tag(more.builders, match_tags));
  resources
    .resource_syncs
    .extend(filter_by_tag(more.resource_syncs, match_tags));
  resources.user_groups.extend(more.user_groups);
  resources.variables.extend(more.variables);
}

fn filter_by_tag<T: Default>(
  resources: Vec<ResourceToml<T>>,
  match_tags: &[String],
) -> Vec<ResourceToml<T>> {
  resources
    .into_iter()
    .filter(|resource| {
      match_tags.iter().all(|tag| resource.tags.contains(tag))
    })
    .collect()
}
