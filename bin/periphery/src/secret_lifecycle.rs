use std::{
  collections::{BTreeMap, BTreeSet, HashMap},
  path::{Path, PathBuf},
  sync::{LazyLock, Mutex as StdMutex},
  time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};
use bollard::{Docker, query_parameters::ListContainersOptions};
use komodo_client::entities::{
  EnvironmentVar, to_path_compatible_name,
};
use periphery_client::api::compose::{
  ExternalFileSecretBinding, ExternalServiceEnvBinding,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::periphery_config;

const MANIFEST_SCHEMA_VERSION: u32 = 1;
const REDACTED_MARKER: &str = "<REDACTED>";

static PROJECT_LOCKS: LazyLock<
  StdMutex<HashMap<String, Arc<Mutex<()>>>>,
> = LazyLock::new(|| StdMutex::new(HashMap::new()));

fn project_lock(project: &str) -> Arc<Mutex<()>> {
  let mut guard =
    PROJECT_LOCKS.lock().unwrap_or_else(|e| e.into_inner());
  guard
    .entry(project.to_string())
    .or_insert_with(|| Arc::new(Mutex::new(())))
    .clone()
}

pub fn secret_runtime_root() -> PathBuf {
  periphery_config()
    .root_directory
    .join("secret-runtime")
    .components()
    .collect()
}

pub fn stack_secret_runtime_root() -> PathBuf {
  secret_runtime_root().join("stacks")
}

pub fn build_secret_runtime_root() -> PathBuf {
  secret_runtime_root().join("builds")
}

fn stack_project_runtime_dir(
  project_name: &str,
) -> anyhow::Result<PathBuf> {
  let safe = to_path_compatible_name(project_name);
  if safe.is_empty()
    || safe == "."
    || safe == ".."
    || safe.contains('/')
    || safe.contains('\\')
  {
    bail!("Invalid Compose project name '{project_name}'");
  }
  Ok(stack_secret_runtime_root().join(safe))
}

fn validate_identifier(
  value: &str,
  kind: &str,
) -> anyhow::Result<()> {
  let trimmed = value.trim();
  if trimmed.is_empty()
    || trimmed.len() > 128
    || trimmed != value
    || value == "."
    || value == ".."
    || !value.chars().all(|c| {
      c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
    })
  {
    bail!("Invalid {kind} '{value}': must match [A-Za-z0-9._-]+");
  }
  Ok(())
}

fn validate_target_path(target_path: &str) -> anyhow::Result<()> {
  let trimmed = target_path.trim();
  if trimmed.is_empty() {
    return Ok(());
  }
  if trimmed != target_path
    || trimmed.bytes().any(|b| b < 0x20 || b == 0x7f)
    || trimmed.contains('\\')
  {
    bail!("Invalid Compose secret target_path '{target_path}'");
  }
  for seg in trimmed.split('/') {
    if seg == ".." {
      bail!(
        "Compose secret target_path '{target_path}' cannot contain '..'"
      );
    }
  }
  Ok(())
}

async fn create_dir_0700(path: &Path) -> anyhow::Result<()> {
  if let Ok(meta) = tokio::fs::symlink_metadata(path).await
    && meta.file_type().is_symlink()
  {
    bail!("Refusing to use symlinked runtime directory {path:?}");
  }
  tokio::fs::create_dir_all(path).await.with_context(|| {
    format!("Failed to create directory {path:?}")
  })?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::set_permissions(
      path,
      std::fs::Permissions::from_mode(0o700),
    )
    .await
    .with_context(|| {
      format!("Failed to set 0700 permissions on {path:?}")
    })?;
  }
  Ok(())
}

/// Recursively relax read-only (`0400`) file permissions inside `dir` before removing it.
pub async fn remove_secret_dir_all(dir: &Path) -> anyhow::Result<()> {
  if tokio::fs::symlink_metadata(dir).await.is_err() {
    return Ok(());
  }
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
      if let Ok(meta) = tokio::fs::symlink_metadata(&cur).await {
        if meta.file_type().is_symlink() {
          continue;
        }
        if meta.is_dir() {
          let _ = tokio::fs::set_permissions(
            &cur,
            std::fs::Permissions::from_mode(0o700),
          )
          .await;
          if let Ok(mut rd) = tokio::fs::read_dir(&cur).await {
            while let Ok(Some(entry)) = rd.next_entry().await {
              stack.push(entry.path());
            }
          }
        } else {
          let _ = tokio::fs::set_permissions(
            &cur,
            std::fs::Permissions::from_mode(0o600),
          )
          .await;
        }
      }
    }
  }
  if tokio::fs::symlink_metadata(dir).await.is_ok() {
    tokio::fs::remove_dir_all(dir)
      .await
      .with_context(|| format!("Failed to remove {dir:?}"))?;
  }
  Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackSecretRuntimeManifest {
  #[serde(default = "default_manifest_version")]
  pub schema_version: u32,
  #[serde(default)]
  pub project_name: String,
  #[serde(default)]
  pub active_generation: Option<String>,
  #[serde(default)]
  pub generations: BTreeMap<String, GenerationRecord>,
}

fn default_manifest_version() -> u32 {
  MANIFEST_SCHEMA_VERSION
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationRecord {
  pub created_at_ms: i64,
  #[serde(default)]
  pub secret_names: Vec<String>,
  #[serde(default)]
  pub service_env_keys: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct PreparedStackSecretGeneration {
  pub project_name: String,
  pub generation_id: String,
  pub generation_dir: PathBuf,
  pub override_compose_path: PathBuf,
  pub record: GenerationRecord,
}

#[derive(Serialize)]
struct ComposeOverrideDoc {
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  services: BTreeMap<String, ComposeOverrideService>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  secrets: BTreeMap<String, ComposeOverrideTopSecret>,
}

#[derive(Default, Serialize)]
struct ComposeOverrideService {
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  environment: BTreeMap<String, String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  secrets: Vec<ComposeOverrideServiceSecretRef>,
}

#[derive(Serialize)]
struct ComposeOverrideServiceSecretRef {
  source: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  target: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  uid: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  gid: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  mode: Option<u32>,
}

#[derive(Serialize)]
struct ComposeOverrideTopSecret {
  file: String,
}

fn now_epoch_ms() -> i64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_millis() as i64)
    .unwrap_or(0)
}

/// Materialize an immutable generation directory containing `0400` file secrets and
/// `0600` `override.compose.yaml` when service-scoped envs or file secrets are present.
pub async fn prepare_stack_secret_generation_at_root(
  stacks_root: &Path,
  project_name: &str,
  service_env_bindings: &[ExternalServiceEnvBinding],
  file_secret_bindings: &[ExternalFileSecretBinding],
) -> anyhow::Result<Option<PreparedStackSecretGeneration>> {
  if service_env_bindings.is_empty()
    && file_secret_bindings.is_empty()
  {
    return Ok(None);
  }

  let safe_proj = to_path_compatible_name(project_name);
  if safe_proj.is_empty() || safe_proj == "." || safe_proj == ".." {
    bail!("Invalid Compose project name '{project_name}'");
  }

  create_dir_0700(stacks_root).await?;
  let proj_dir = stacks_root.join(&safe_proj);
  create_dir_0700(&proj_dir).await?;
  let gens_dir = proj_dir.join("generations");
  create_dir_0700(&gens_dir).await?;

  let ts_ms = now_epoch_ms();
  let short = &Uuid::new_v4().simple().to_string()[..12];
  let generation_id = format!("gen-{ts_ms}-{short}");
  let generation_dir = gens_dir.join(&generation_id);
  create_dir_0700(&generation_dir).await?;

  let build_res: anyhow::Result<PreparedStackSecretGeneration> =
    async {
      let mut services_map: BTreeMap<String, ComposeOverrideService> =
        BTreeMap::new();
      let mut top_secrets: BTreeMap<
        String,
        ComposeOverrideTopSecret,
      > = BTreeMap::new();
      let mut service_env_keys: BTreeMap<String, Vec<String>> =
        BTreeMap::new();
      let mut secret_names = Vec::new();

      for binding in service_env_bindings {
        environment::validate_env_var_key(&binding.variable)?;
        if binding.value.contains('\0') {
          bail!(
            "External service environment variable '{}' cannot contain NUL bytes",
            binding.variable
          );
        }
        if binding.services.is_empty() {
          bail!(
            "ServiceEnv binding for '{}' must specify at least one target Compose service",
            binding.variable
          );
        }
        // Escape `$` as `$$` so Docker Compose YAML interpolation preserves literal `$` / `${...}`.
        let yaml_literal_value = binding.value.replace('$', "$$");
        for svc in &binding.services {
          validate_identifier(svc, "Compose service name")?;
          services_map
            .entry(svc.clone())
            .or_default()
            .environment
            .insert(
              binding.variable.clone(),
              yaml_literal_value.clone(),
            );
          let keys_vec =
            service_env_keys.entry(svc.clone()).or_default();
          if !keys_vec.contains(&binding.variable) {
            keys_vec.push(binding.variable.clone());
          }
        }
      }

      if !file_secret_bindings.is_empty() {
        let files_dir = generation_dir.join("files");
        create_dir_0700(&files_dir).await?;

        for binding in file_secret_bindings {
          validate_identifier(
            &binding.secret_name,
            "Compose file secret name",
          )?;
          if binding.services.is_empty() {
            bail!(
              "ComposeFileSecret '{}' must specify at least one target Compose service",
              binding.secret_name
            );
          }
          validate_target_path(&binding.target_path)?;

          let host_mode = binding
            .mode
            .map(|m| m & 0o777)
            .filter(|m| *m != 0)
            .unwrap_or(0o400);
          let secret_file_path =
            files_dir.join(&binding.secret_name);
          environment::write_atomic_mode(
            &secret_file_path,
            &binding.bytes,
            host_mode,
          )
          .await
          .with_context(|| {
            format!(
              "Failed to write runtime Compose secret file '{}'",
              binding.secret_name
            )
          })?;

          let canonical_file =
            secret_file_path.canonicalize().unwrap_or(secret_file_path);

          top_secrets.insert(
            binding.secret_name.clone(),
            ComposeOverrideTopSecret {
              file: canonical_file.to_string_lossy().into_owned(),
            },
          );
          secret_names.push(binding.secret_name.clone());

          let target = (!binding.target_path.trim().is_empty())
            .then(|| binding.target_path.trim().to_string());
          let uid = (!binding.uid.trim().is_empty())
            .then(|| binding.uid.trim().to_string());
          let gid = (!binding.gid.trim().is_empty())
            .then(|| binding.gid.trim().to_string());

          for svc in &binding.services {
            validate_identifier(svc, "Compose service name")?;
            services_map
              .entry(svc.clone())
              .or_default()
              .secrets
              .push(ComposeOverrideServiceSecretRef {
                source: binding.secret_name.clone(),
                target: target.clone(),
                uid: uid.clone(),
                gid: gid.clone(),
                mode: binding.mode,
              });
          }
        }
      }

      let doc = ComposeOverrideDoc {
        services: services_map,
        secrets: top_secrets,
      };
      let yaml = serde_yaml_ng::to_string(&doc)
        .context("Failed to serialize override.compose.yaml")?;
      let override_compose_path =
        generation_dir.join("override.compose.yaml");
      environment::write_atomic_mode(
        &override_compose_path,
        yaml.as_bytes(),
        0o600,
      )
      .await
      .context("Failed to write override.compose.yaml")?;

      let canonical_override = override_compose_path
        .canonicalize()
        .unwrap_or(override_compose_path);

      Ok(PreparedStackSecretGeneration {
        project_name: project_name.to_string(),
        generation_id,
        generation_dir,
        override_compose_path: canonical_override,
        record: GenerationRecord {
          created_at_ms: ts_ms,
          secret_names,
          service_env_keys,
        },
      })
    }
    .await;

  if build_res.is_err() {
    let _ = remove_secret_dir_all(
      &gens_dir.join(format!("gen-{ts_ms}-{short}")),
    )
    .await;
  }

  build_res.map(Some)
}

pub async fn prepare_stack_secret_generation(
  project_name: &str,
  service_env_bindings: &[ExternalServiceEnvBinding],
  file_secret_bindings: &[ExternalFileSecretBinding],
) -> anyhow::Result<Option<PreparedStackSecretGeneration>> {
  let root = stack_secret_runtime_root();
  if let Ok(parent) = secret_runtime_root().canonicalize() {
    let _ = create_dir_0700(&parent).await;
  } else {
    create_dir_0700(&secret_runtime_root()).await?;
  }
  prepare_stack_secret_generation_at_root(
    &root,
    project_name,
    service_env_bindings,
    file_secret_bindings,
  )
  .await
}

async fn load_manifest(
  proj_dir: &Path,
  project_name: &str,
) -> StackSecretRuntimeManifest {
  let path = proj_dir.join("manifest.json");
  if let Ok(raw) = tokio::fs::read_to_string(&path).await
    && let Ok(parsed) =
      serde_json::from_str::<StackSecretRuntimeManifest>(&raw)
  {
    return parsed;
  }
  StackSecretRuntimeManifest {
    schema_version: MANIFEST_SCHEMA_VERSION,
    project_name: project_name.to_string(),
    active_generation: None,
    generations: BTreeMap::new(),
  }
}

async fn save_manifest(
  proj_dir: &Path,
  manifest: &StackSecretRuntimeManifest,
) -> anyhow::Result<()> {
  let bytes = serde_json::to_vec_pretty(manifest)
    .context("Failed to serialize runtime secret manifest")?;
  environment::write_atomic_mode(
    &proj_dir.join("manifest.json"),
    &bytes,
    0o600,
  )
  .await
}

/// Query Docker daemon for all container bind-mount source paths (including stopped containers
/// that may be started/restarted later!). Fails closed by returning `Err` if Docker is unreachable.
async fn list_all_container_mount_sources()
-> anyhow::Result<Vec<PathBuf>> {
  let docker = Docker::connect_with_defaults()
    .context("Failed to connect to Docker API for secret mount GC")?;
  let containers = docker
    .list_containers(Some(ListContainersOptions {
      all: true,
      ..Default::default()
    }))
    .await
    .context(
      "Failed to list Docker containers for secret mount GC",
    )?;

  let mut sources = Vec::new();
  for c in containers {
    if let Some(mounts) = c.mounts {
      for m in mounts {
        if let Some(src) = m.source
          && !src.is_empty()
        {
          let pb = PathBuf::from(&src);
          let canon = pb.canonicalize().unwrap_or(pb);
          sources.push(canon);
        }
      }
    }
  }
  Ok(sources)
}

fn is_dir_mounted(dir: &Path, mount_sources: &[PathBuf]) -> bool {
  let canon_dir =
    dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
  mount_sources
    .iter()
    .any(|src| src.starts_with(&canon_dir) || src.starts_with(dir))
}

/// Promote `prepared` (or clear `active_generation` when `None`) in `manifest.json`
/// and garbage-collect older unmounted generations.
pub async fn commit_stack_secret_generation_at_root(
  stacks_root: &Path,
  project_name: &str,
  prepared: Option<&PreparedStackSecretGeneration>,
  mount_sources_override: Option<anyhow::Result<Vec<PathBuf>>>,
) -> anyhow::Result<Option<String>> {
  let lock = project_lock(project_name);
  let _guard = lock.lock().await;

  let safe_proj = to_path_compatible_name(project_name);
  let proj_dir = stacks_root.join(&safe_proj);

  if prepared.is_none()
    && tokio::fs::symlink_metadata(&proj_dir).await.is_err()
  {
    return Ok(None);
  }

  create_dir_0700(&proj_dir).await?;
  let mut manifest = load_manifest(&proj_dir, project_name).await;
  manifest.project_name = project_name.to_string();

  let new_active = if let Some(prep) = prepared {
    manifest
      .generations
      .insert(prep.generation_id.clone(), prep.record.clone());
    manifest.active_generation = Some(prep.generation_id.clone());
    Some(prep.generation_id.clone())
  } else {
    manifest.active_generation = None;
    None
  };

  save_manifest(&proj_dir, &manifest).await?;

  // GC older unmounted generations (fail-closed if mount inspection fails!)
  let mounts_res = match mount_sources_override {
    Some(r) => r,
    None => list_all_container_mount_sources().await,
  };

  if let Ok(mount_sources) = mounts_res {
    let gens_dir = proj_dir.join("generations");
    let mut removed_ids = Vec::new();
    for gen_id in manifest.generations.keys() {
      if manifest.active_generation.as_deref()
        == Some(gen_id.as_str())
      {
        continue;
      }
      let gdir = gens_dir.join(gen_id);
      if !is_dir_mounted(&gdir, &mount_sources) {
        if remove_secret_dir_all(&gdir).await.is_ok() {
          removed_ids.push(gen_id.clone());
        }
      }
    }
    for id in removed_ids {
      manifest.generations.remove(&id);
    }

    // Also sweep any untracked directories inside `generations/` that aren't active or mounted
    if let Ok(mut rd) = tokio::fs::read_dir(&gens_dir).await {
      while let Ok(Some(entry)) = rd.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        if manifest.active_generation.as_deref()
          == Some(name.as_str())
          || manifest.generations.contains_key(&name)
        {
          continue;
        }
        let path = entry.path();
        if !is_dir_mounted(&path, &mount_sources) {
          let _ = remove_secret_dir_all(&path).await;
        }
      }
    }

    if manifest.active_generation.is_none()
      && manifest.generations.is_empty()
    {
      let _ = remove_secret_dir_all(&proj_dir).await;
    } else {
      save_manifest(&proj_dir, &manifest).await?;
    }
  } else if let Err(e) = mounts_res {
    warn!(
      "Skipping runtime secret generation GC for stack '{project_name}' (unable to inspect Docker mounts, failing closed): {e:#}"
    );
  }

  Ok(new_active)
}

pub async fn commit_stack_secret_generation(
  project_name: &str,
  prepared: Option<&PreparedStackSecretGeneration>,
) -> anyhow::Result<Option<String>> {
  commit_stack_secret_generation_at_root(
    &stack_secret_runtime_root(),
    project_name,
    prepared,
    None,
  )
  .await
}

/// Clean up an unpromoted generation directory (e.g., after `ComposePull`, `ComposeRun`,
/// or a failed `ComposeUp`), preserving `manifest.json` and any previously active generation.
pub async fn abort_prepared_stack_secret_generation(
  prepared: Option<PreparedStackSecretGeneration>,
) {
  let Some(prep) = prepared else {
    return;
  };
  let lock = project_lock(&prep.project_name);
  let _guard = lock.lock().await;

  if let Ok(mounts) = list_all_container_mount_sources().await {
    if !is_dir_mounted(&prep.generation_dir, &mounts) {
      let _ = remove_secret_dir_all(&prep.generation_dir).await;
    }
  }
}

/// Purge all runtime secret files and `manifest.json` for `project_name` after `docker compose down` / stack deletion.
pub async fn purge_stack_secret_runtime(
  project_name: &str,
) -> anyhow::Result<()> {
  let lock = project_lock(project_name);
  let _guard = lock.lock().await;
  let proj_dir = stack_project_runtime_dir(project_name)?;
  if tokio::fs::symlink_metadata(&proj_dir).await.is_err() {
    return Ok(());
  }
  if let Ok(mounts) = list_all_container_mount_sources().await
    && is_dir_mounted(&proj_dir, &mounts)
  {
    warn!(
      "Stack '{project_name}' still has containers mounting runtime secrets; clearing active_generation and retaining mounted generation"
    );
    drop(_guard);
    let _ = commit_stack_secret_generation(project_name, None).await;
    return Ok(());
  }
  remove_secret_dir_all(&proj_dir).await
}

/// Startup reconciliation: clean up leftover build temp dirs, `.komodo-tmp-*` files,
/// and unmounted non-active stack secret generations.
pub async fn reconcile_runtime_secrets_on_startup() {
  let builds_root = build_secret_runtime_root();
  if tokio::fs::symlink_metadata(&builds_root).await.is_ok() {
    if let Err(e) = remove_secret_dir_all(&builds_root).await {
      warn!(
        "Failed to clean up leftover build secret runtime directory on startup | {e:#}"
      );
    }
  }

  let stacks_root = stack_secret_runtime_root();
  let Ok(mut rd) = tokio::fs::read_dir(&stacks_root).await else {
    return;
  };
  let Ok(mount_sources) = list_all_container_mount_sources().await
  else {
    return;
  };

  while let Ok(Some(entry)) = rd.next_entry().await {
    let proj_dir = entry.path();
    let proj_name = entry.file_name().to_string_lossy().into_owned();
    if proj_name.starts_with(".komodo-tmp-") {
      let _ = tokio::fs::remove_file(&proj_dir).await;
      continue;
    }
    let mut manifest = load_manifest(&proj_dir, &proj_name).await;
    let gens_dir = proj_dir.join("generations");
    let mut changed = false;

    let known_ids: Vec<String> =
      manifest.generations.keys().cloned().collect();
    for gid in known_ids {
      if manifest.active_generation.as_deref() == Some(gid.as_str()) {
        continue;
      }
      let gdir = gens_dir.join(&gid);
      if !is_dir_mounted(&gdir, &mount_sources)
        && remove_secret_dir_all(&gdir).await.is_ok()
      {
        manifest.generations.remove(&gid);
        changed = true;
      }
    }

    if let Ok(mut grd) = tokio::fs::read_dir(&gens_dir).await {
      while let Ok(Some(gentry)) = grd.next_entry().await {
        let gname = gentry.file_name().to_string_lossy().into_owned();
        if manifest.active_generation.as_deref()
          == Some(gname.as_str())
          || manifest.generations.contains_key(&gname)
        {
          continue;
        }
        let gpath = gentry.path();
        if !is_dir_mounted(&gpath, &mount_sources) {
          let _ = remove_secret_dir_all(&gpath).await;
        }
      }
    }

    if manifest.active_generation.is_none()
      && manifest.generations.is_empty()
    {
      let _ = remove_secret_dir_all(&proj_dir).await;
    } else if changed {
      let _ = save_manifest(&proj_dir, &manifest).await;
    }
  }
}

/// Structurally redact `services.<svc>.environment` keys and internal `/secret-runtime/stacks/`
/// host paths in `docker compose config` YAML output on Periphery before returning `DeployStackResponse`.
pub fn redact_compose_merged_config_yaml(
  raw_yaml: &str,
  secret_env_keys: &[String],
) -> String {
  if raw_yaml.trim().is_empty() {
    return raw_yaml.to_string();
  }
  let key_set: BTreeSet<&str> =
    secret_env_keys.iter().map(String::as_str).collect();

  let Ok(mut doc) =
    serde_yaml_ng::from_str::<serde_yaml_ng::Value>(raw_yaml)
  else {
    return raw_yaml.to_string();
  };

  if let serde_yaml_ng::Value::Mapping(root) = &mut doc {
    if !key_set.is_empty()
      && let Some(serde_yaml_ng::Value::Mapping(services)) =
        root.get_mut(serde_yaml_ng::Value::String("services".into()))
    {
      for (_, svc_val) in services.iter_mut() {
        let serde_yaml_ng::Value::Mapping(svc_map) = svc_val else {
          continue;
        };
        let Some(env_val) = svc_map.get_mut(
          serde_yaml_ng::Value::String("environment".into()),
        ) else {
          continue;
        };
        match env_val {
          serde_yaml_ng::Value::Mapping(env_map) => {
            for (k, v) in env_map.iter_mut() {
              if let Some(k_str) = k.as_str()
                && key_set.contains(k_str)
              {
                *v = serde_yaml_ng::Value::String(
                  REDACTED_MARKER.to_string(),
                );
              }
            }
          }
          serde_yaml_ng::Value::Sequence(env_seq) => {
            for item in env_seq.iter_mut() {
              if let Some(s) = item.as_str()
                && let Some((k_str, _)) = s.split_once('=')
                && key_set.contains(k_str.trim())
              {
                *item = serde_yaml_ng::Value::String(format!(
                  "{}={REDACTED_MARKER}",
                  k_str.trim()
                ));
              }
            }
          }
          _ => {}
        }
      }
    }

    if let Some(serde_yaml_ng::Value::Mapping(secrets_map)) =
      root.get_mut(serde_yaml_ng::Value::String("secrets".into()))
    {
      for (_, sec_val) in secrets_map.iter_mut() {
        if let serde_yaml_ng::Value::Mapping(sec_obj) = sec_val
          && let Some(file_val) = sec_obj
            .get_mut(serde_yaml_ng::Value::String("file".into()))
          && let Some(file_str) = file_val.as_str()
          && file_str.contains("/secret-runtime/stacks/")
        {
          *file_val =
            serde_yaml_ng::Value::String(REDACTED_MARKER.to_string());
        }
      }
    }
  }

  serde_yaml_ng::to_string(&doc)
    .unwrap_or_else(|_| raw_yaml.to_string())
}

fn collect_secret_values_from_env_slice(
  env_entries: &[String],
  key_set: &BTreeSet<&str>,
  values: &mut BTreeSet<String>,
) {
  for entry in env_entries {
    if let Some((k, v)) = entry.split_once('=')
      && key_set.contains(k)
      && v.len() >= 4
    {
      values.insert(v.to_string());
    }
  }
}

fn apply_value_redactions_to_log(
  values: &BTreeSet<String>,
  log: &mut komodo_client::entities::update::Log,
) {
  if values.is_empty() {
    return;
  }
  let mut sorted: Vec<&str> =
    values.iter().map(String::as_str).collect();
  sorted.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
  for needle in sorted {
    if !log.stdout.is_empty() && log.stdout.contains(needle) {
      log.stdout = log.stdout.replace(needle, REDACTED_MARKER);
    }
    if !log.stderr.is_empty() && log.stderr.contains(needle) {
      log.stderr = log.stderr.replace(needle, REDACTED_MARKER);
    }
  }
}

/// Scrub runtime secret environment variable values from Docker Compose logs on Periphery
/// before returning `Log` to Core (preserving Zero Plaintext Persistence in Core/MongoDB).
pub async fn redact_compose_project_log(
  project: &str,
  redact_env_keys: &[String],
  log: &mut komodo_client::entities::update::Log,
) {
  if redact_env_keys.is_empty()
    || (log.stdout.is_empty() && log.stderr.is_empty())
  {
    return;
  }
  let key_set: BTreeSet<&str> =
    redact_env_keys.iter().map(String::as_str).collect();
  let Ok(docker) = Docker::connect_with_defaults() else {
    return;
  };
  let Ok(containers) = docker
    .list_containers(Some(ListContainersOptions {
      all: true,
      ..Default::default()
    }))
    .await
  else {
    return;
  };

  let mut secret_values = BTreeSet::new();
  for c in containers {
    let is_project = c
      .labels
      .as_ref()
      .and_then(|l| l.get("com.docker.compose.project"))
      .is_some_and(|p| p == project);
    if !is_project {
      continue;
    }
    let Some(cid) = c.id.as_deref() else {
      continue;
    };
    if let Ok(inspected) = docker.inspect_container(cid, None).await
      && let Some(cfg) = inspected.config
      && let Some(env) = cfg.env
    {
      collect_secret_values_from_env_slice(
        &env,
        &key_set,
        &mut secret_values,
      );
    }
  }

  apply_value_redactions_to_log(&secret_values, log);
}

/// Scrub runtime secret environment variable values from a single Docker container's logs on Periphery.
pub async fn redact_single_container_log(
  container_name: &str,
  redact_env_keys: &[String],
  log: &mut komodo_client::entities::update::Log,
) {
  if redact_env_keys.is_empty()
    || (log.stdout.is_empty() && log.stderr.is_empty())
  {
    return;
  }
  let key_set: BTreeSet<&str> =
    redact_env_keys.iter().map(String::as_str).collect();
  let Ok(docker) = Docker::connect_with_defaults() else {
    return;
  };
  let mut secret_values = BTreeSet::new();
  if let Ok(inspected) =
    docker.inspect_container(container_name, None).await
    && let Some(cfg) = inspected.config
    && let Some(env) = cfg.env
  {
    collect_secret_values_from_env_slice(
      &env,
      &key_set,
      &mut secret_values,
    );
  }
  apply_value_redactions_to_log(&secret_values, log);
}

/// Isolated `0700` BuildKit `--secret` directory outside `build_dir` / Git working tree.
/// Automatically deletes all temporary secret files on explicit `cleanup()` AND on `Drop`.
pub struct ScopedBuildSecretDir {
  dir: PathBuf,
  cleaned: bool,
}

impl ScopedBuildSecretDir {
  pub async fn create_at_root(
    builds_root: &Path,
    run_id: &str,
  ) -> anyhow::Result<Self> {
    let safe_id = to_path_compatible_name(run_id);
    let unique = format!(
      "{}-{}",
      if safe_id.is_empty() {
        "build"
      } else {
        &safe_id
      },
      &Uuid::new_v4().simple().to_string()[..12]
    );
    create_dir_0700(builds_root).await?;
    let dir = builds_root.join(unique);
    create_dir_0700(&dir).await?;
    Ok(Self {
      dir,
      cleaned: false,
    })
  }

  pub async fn create(run_id: &str) -> anyhow::Result<Self> {
    create_dir_0700(&secret_runtime_root()).await?;
    Self::create_at_root(&build_secret_runtime_root(), run_id).await
  }

  #[cfg(test)]
  pub fn path(&self) -> &Path {
    &self.dir
  }

  /// Write all standard `secret_args` and `external_secret_args` (`0600` mode) into the isolated
  /// directory and return formatted ` --secret id=...,src=...` flags.
  pub async fn write_buildkit_secret_args(
    &self,
    standard_secret_args: &[EnvironmentVar],
    external_secret_args: &[periphery_client::api::ExternalEnvVar],
  ) -> anyhow::Result<String> {
    use std::fmt::Write;
    let external_ids: BTreeSet<&str> = external_secret_args
      .iter()
      .map(|s| s.variable.as_str())
      .collect();

    let mut combined: Vec<(&str, &str)> = Vec::new();
    for s in standard_secret_args {
      if !external_ids.contains(s.variable.as_str()) {
        combined.push((s.variable.as_str(), s.value.as_str()));
      }
    }
    for ext in external_secret_args {
      combined.push((ext.variable.as_str(), ext.value.as_str()));
    }

    let mut args_str = String::new();
    for (secret_id, secret_val) in combined {
      validate_identifier(secret_id, "BuildKit secret id")?;
      let file_path = self.dir.join(secret_id);
      environment::write_atomic_mode(
        &file_path,
        secret_val.as_bytes(),
        0o600,
      )
      .await
      .with_context(|| {
        format!(
          "Failed to write temporary BuildKit secret '{secret_id}'"
        )
      })?;
      let escaped_src =
        shell_escape::unix::escape(file_path.to_string_lossy());
      write!(
        &mut args_str,
        " --secret id={secret_id},src={escaped_src}"
      )
      .with_context(|| {
        format!("Failed to format --secret flag for '{secret_id}'")
      })?;
    }

    Ok(args_str)
  }

  pub async fn cleanup(&mut self) {
    if !self.cleaned {
      let _ = remove_secret_dir_all(&self.dir).await;
      self.cleaned = true;
    }
  }
}

impl Drop for ScopedBuildSecretDir {
  fn drop(&mut self) {
    if !self.cleaned {
      let _ = std::fs::remove_dir_all(&self.dir);
      self.cleaned = true;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;

  fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
      "komodo-seclife-{label}-{}-{}",
      std::process::id(),
      now_epoch_ms()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[tokio::test]
  async fn generation_lifecycle_promotes_rolls_back_and_enforces_0400_permissions()
   {
    let root = temp_dir("stacks");

    // 1. Prepare generation 1 with both ServiceEnv (containing $) and ComposeFileSecret
    let gen1 = prepare_stack_secret_generation_at_root(
      &root,
      "demo-stack",
      &[ExternalServiceEnvBinding {
        variable: String::from("API_SECRET"),
        value: String::from("s3cr#t_$DOLLAR_${HOME}"),
        services: vec![String::from("api")],
      }],
      &[ExternalFileSecretBinding {
        secret_name: String::from("tls_cert"),
        bytes: b"-----BEGIN CERT-----\nBODY\n-----END CERT-----\n"
          .to_vec(),
        services: vec![String::from("api")],
        target_path: String::from("/run/secrets/tls_cert.pem"),
        uid: String::new(),
        gid: String::new(),
        mode: Some(0o400),
      }],
    )
    .await
    .unwrap()
    .expect("should prepare gen1");

    let secret_file = gen1.generation_dir.join("files/tls_cert");
    let secret_mode = std::fs::metadata(&secret_file)
      .unwrap()
      .permissions()
      .mode()
      & 0o777;
    assert_eq!(secret_mode, 0o400);

    let override_yaml =
      std::fs::read_to_string(&gen1.override_compose_path).unwrap();
    assert!(override_yaml.contains("s3cr#t_$$DOLLAR_$${HOME}"));
    assert!(override_yaml.contains("/run/secrets/tls_cert.pem"));

    // Promote gen1
    let active1 = commit_stack_secret_generation_at_root(
      &root,
      "demo-stack",
      Some(&gen1),
      Some(Ok(vec![])),
    )
    .await
    .unwrap();
    assert_eq!(active1.as_deref(), Some(gen1.generation_id.as_str()));

    // 2. Prepare generation 2 and promote with empty mounts -> gen1 should be GC'd!
    let gen2 = prepare_stack_secret_generation_at_root(
      &root,
      "demo-stack",
      &[ExternalServiceEnvBinding {
        variable: String::from("API_SECRET"),
        value: String::from("rotated_v2"),
        services: vec![String::from("api")],
      }],
      &[],
    )
    .await
    .unwrap()
    .expect("should prepare gen2");

    commit_stack_secret_generation_at_root(
      &root,
      "demo-stack",
      Some(&gen2),
      Some(Ok(vec![])),
    )
    .await
    .unwrap();

    assert!(
      !gen1.generation_dir.exists(),
      "unmounted previous generation must be GC'd"
    );
    assert!(
      gen2.generation_dir.exists(),
      "active generation must be retained"
    );

    // 3. Clearing bindings (`None`) with no mounts removes active_generation and cleans up dir
    commit_stack_secret_generation_at_root(
      &root,
      "demo-stack",
      None,
      Some(Ok(vec![])),
    )
    .await
    .unwrap();
    assert!(!gen2.generation_dir.exists());

    let _ = remove_secret_dir_all(&root).await;
  }

  #[tokio::test]
  async fn scoped_build_secret_dir_isolates_0600_files_and_cleans_up_on_drop()
   {
    let root = temp_dir("builds");
    let captured_dir: PathBuf;
    {
      let guard =
        ScopedBuildSecretDir::create_at_root(&root, "build-42")
          .await
          .unwrap();
      captured_dir = guard.path().to_path_buf();
      let flags = guard
        .write_buildkit_secret_args(
          &[EnvironmentVar {
            variable: String::from("LOCAL_SEC"),
            value: String::from("local123"),
          }],
          &[periphery_client::api::ExternalEnvVar {
            variable: String::from("NPM_TOKEN"),
            value: String::from("npm_secret_value"),
          }],
        )
        .await
        .unwrap();
      assert!(flags.contains("--secret id=LOCAL_SEC,src="));
      assert!(flags.contains("--secret id=NPM_TOKEN,src="));
      let f_mode = std::fs::metadata(captured_dir.join("NPM_TOKEN"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
      assert_eq!(f_mode, 0o600);
      // Drop without calling `.cleanup().await` -> Drop removes directory synchronously!
    }
    assert!(
      !captured_dir.exists(),
      "ScopedBuildSecretDir Drop must remove isolated secret dir"
    );
    let _ = remove_secret_dir_all(&root).await;
  }
}
