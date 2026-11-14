use std::collections::BTreeSet;

use database::mungos::{find::find_collect, mongodb::bson::doc};
use komodo_client::{
  entities::{
    deployment::Deployment,
    docker::{
      container::Container,
      service::SwarmService,
      task::{SwarmTask, TaskSpec},
    },
    secret_provider::StackSecretTarget,
    stack::Stack,
    update::Log,
  },
  parsers::parse_key_value_list,
};

use crate::state::{all_resources_cache, db_client};

use super::authz::collect_inline_vault_refs_in_str;
use super::execution::get_resource_deployment_metadata;

pub const REDACTED_PLACEHOLDER: &str = "<REDACTED>";

/// Structurally redact secret-bearing environment variable keys inside `docker compose config` YAML output.
/// Preserves YAML structure and non-secret keys while ensuring `secret_env_keys` never appear in `StackInfo.deployed_config`
/// even if a value is short, empty, or reformatted by Docker Compose.
pub fn redact_compose_merged_config_yaml(
  yaml: &str,
  secret_env_keys: &[String],
) -> String {
  if yaml.is_empty() || secret_env_keys.is_empty() {
    return yaml.to_string();
  }
  let key_set: BTreeSet<&str> =
    secret_env_keys.iter().map(String::as_str).collect();

  if let Ok(mut doc) =
    serde_yaml_ng::from_str::<serde_yaml_ng::Value>(yaml)
    && let Some(root_map) = doc.as_mapping_mut()
  {
    let services_key =
      serde_yaml_ng::Value::String(String::from("services"));
    if let Some(services_val) = root_map.get_mut(&services_key)
      && let Some(services_map) = services_val.as_mapping_mut()
    {
      let env_key =
        serde_yaml_ng::Value::String(String::from("environment"));
      for (_svc_name, svc_val) in services_map.iter_mut() {
        let Some(svc_map) = svc_val.as_mapping_mut() else {
          continue;
        };
        let Some(env_val) = svc_map.get_mut(&env_key) else {
          continue;
        };
        match env_val {
          serde_yaml_ng::Value::Mapping(env_map) => {
            for (k, v) in env_map.iter_mut() {
              if let Some(k_str) = k.as_str()
                && key_set.contains(k_str)
              {
                *v = serde_yaml_ng::Value::String(
                  REDACTED_PLACEHOLDER.to_string(),
                );
              }
            }
          }
          serde_yaml_ng::Value::Sequence(env_seq) => {
            for item in env_seq.iter_mut() {
              if let Some(entry) = item.as_str()
                && let Some((k, _)) = entry.split_once('=')
                && key_set.contains(k)
              {
                *item = serde_yaml_ng::Value::String(format!(
                  "{k}={REDACTED_PLACEHOLDER}"
                ));
              }
            }
          }
          _ => {}
        }
      }
    }

    // Mask internal host file paths under top-level `secrets:` if they point into Komodo's secret-runtime directory
    let secrets_key =
      serde_yaml_ng::Value::String(String::from("secrets"));
    if let Some(secrets_val) = root_map.get_mut(&secrets_key)
      && let Some(secrets_map) = secrets_val.as_mapping_mut()
    {
      let file_key =
        serde_yaml_ng::Value::String(String::from("file"));
      for (_sec_name, sec_val) in secrets_map.iter_mut() {
        if let Some(sec_map) = sec_val.as_mapping_mut()
          && let Some(file_val) = sec_map.get_mut(&file_key)
          && let Some(path_str) = file_val.as_str()
          && (path_str.contains("/secret-runtime/stacks/")
            || path_str.contains(".komodo-secrets"))
        {
          *file_val = serde_yaml_ng::Value::String(String::from(
            "<MANAGED_RUNTIME_SECRET_FILE>",
          ));
        }
      }
    }

    if let Ok(serialized) = serde_yaml_ng::to_string(&doc) {
      return serialized;
    }
  }

  // Fallback line-based scrubber if YAML parsing fails
  redact_text_env_assignments(yaml, &key_set)
}

/// Mask `KEY=VALUE` entries in a Docker `Env` vector for any `KEY` in `secret_keys`.
pub fn mask_env_vec(
  env: &mut [String],
  secret_keys: &BTreeSet<String>,
) {
  if secret_keys.is_empty() {
    return;
  }
  for entry in env.iter_mut() {
    if let Some((key, _)) = entry.split_once('=')
      && secret_keys.contains(key)
    {
      *entry = format!("{key}={REDACTED_PLACEHOLDER}");
    }
  }
}

/// Mask secret-bearing environment variables in `Container.config.env`.
pub fn mask_container_env(
  container: &mut Container,
  secret_keys: &BTreeSet<String>,
) {
  if secret_keys.is_empty() {
    return;
  }
  if let Some(cfg) = container.config.as_mut() {
    mask_env_vec(&mut cfg.env, secret_keys);
  }
}

/// Mask secret-bearing environment variables in a Swarm `TaskSpec`.
pub fn mask_task_spec_env(
  spec: &mut TaskSpec,
  secret_keys: &BTreeSet<String>,
) {
  if secret_keys.is_empty() {
    return;
  }
  if let Some(container_spec) = spec.container_spec.as_mut()
    && let Some(env) = container_spec.env.as_mut()
  {
    mask_env_vec(env, secret_keys);
  }
}

/// Mask secret-bearing environment variables in `SwarmService`.
pub fn mask_swarm_service_env(
  service: &mut SwarmService,
  secret_keys: &BTreeSet<String>,
) {
  if secret_keys.is_empty() {
    return;
  }
  if let Some(spec) = service.spec.as_mut()
    && let Some(task_template) = spec.task_template.as_mut()
  {
    mask_task_spec_env(task_template, secret_keys);
  }
}

/// Mask secret-bearing environment variables in `SwarmTask`.
pub fn mask_swarm_task_env(
  task: &mut SwarmTask,
  secret_keys: &BTreeSet<String>,
) {
  if secret_keys.is_empty() {
    return;
  }
  if let Some(spec) = task.spec.as_mut() {
    mask_task_spec_env(spec, secret_keys);
  }
}

fn redact_text_env_assignments(
  input: &str,
  secret_keys: &BTreeSet<&str>,
) -> String {
  if input.is_empty() || secret_keys.is_empty() {
    return input.to_string();
  }
  let mut out = String::with_capacity(input.len());
  for chunk in input.split_inclusive('\n') {
    let (line, ending) =
      if let Some(stripped) = chunk.strip_suffix("\r\n") {
        (stripped, "\r\n")
      } else if let Some(stripped) = chunk.strip_suffix('\n') {
        (stripped, "\n")
      } else {
        (chunk, "")
      };

    out.push_str(&redact_single_line_assignment(line, secret_keys));
    out.push_str(ending);
  }
  out
}

fn is_ident_byte(b: u8) -> bool {
  b.is_ascii_alphanumeric() || b == b'_'
}

fn redact_single_line_assignment(
  line: &str,
  secret_keys: &BTreeSet<&str>,
) -> String {
  // Check earliest occurrence of any `KEY=` (or YAML `KEY: `) at an identifier boundary on this line
  let bytes = line.as_bytes();
  let mut earliest: Option<(usize, &str, &str)> = None;

  for &key in secret_keys {
    for sep in ["=", ": "] {
      let pattern = format!("{key}{sep}");
      let mut search_from = 0;
      while let Some(rel_idx) = line[search_from..].find(&pattern) {
        let abs_idx = search_from + rel_idx;
        let boundary_ok =
          abs_idx == 0 || !is_ident_byte(bytes[abs_idx - 1]);
        if boundary_ok {
          if earliest
            .is_none_or(|(prev_idx, _, _)| abs_idx < prev_idx)
          {
            earliest = Some((abs_idx, key, sep));
          }
          break;
        }
        search_from = abs_idx + key.len();
      }
    }
  }

  if let Some((idx, key, sep)) = earliest {
    format!("{}{}{}{}", &line[..idx], key, sep, REDACTED_PLACEHOLDER)
  } else {
    line.to_string()
  }
}

/// Best-effort redaction of `KEY=...` / `export KEY=...` env dump lines on runtime container/compose logs.
pub fn redact_log_env_assignments(
  log: &mut Log,
  secret_keys: &BTreeSet<String>,
) {
  if secret_keys.is_empty() {
    return;
  }
  let refs: BTreeSet<&str> =
    secret_keys.iter().map(String::as_str).collect();
  if !log.stdout.is_empty() {
    log.stdout = redact_text_env_assignments(&log.stdout, &refs);
  }
  if !log.stderr.is_empty() {
    log.stderr = redact_text_env_assignments(&log.stderr, &refs);
  }
}

/// Extract external-secret-bearing env var names statically configured on a `Stack`.
pub fn configured_stack_secret_env_keys(
  stack: &Stack,
) -> BTreeSet<String> {
  let mut keys = BTreeSet::new();
  if let Ok(parsed) = parse_key_value_list(&stack.config.environment)
  {
    for (k, v) in parsed {
      let has_vault = collect_inline_vault_refs_in_str(&v)
        .map(|r| !r.is_empty())
        .unwrap_or_else(|_| v.contains("[[vault"));
      if has_vault {
        keys.insert(k);
      }
    }
  }
  for binding in &stack.config.secret_bindings {
    match &binding.target {
      StackSecretTarget::ComposeEnv { variable }
      | StackSecretTarget::ServiceEnv { variable, .. } => {
        if !variable.trim().is_empty() {
          keys.insert(variable.trim().to_string());
        }
      }
      StackSecretTarget::ComposeFileSecret { .. } => {}
    }
  }
  keys
}

/// Extract external-secret-bearing env var names statically configured on a `Deployment`.
pub fn configured_deployment_secret_env_keys(
  deployment: &Deployment,
) -> BTreeSet<String> {
  let mut keys = BTreeSet::new();
  if let Ok(parsed) =
    parse_key_value_list(&deployment.config.environment)
  {
    for (k, v) in parsed {
      let has_vault = collect_inline_vault_refs_in_str(&v)
        .map(|r| !r.is_empty())
        .unwrap_or_else(|_| v.contains("[[vault"));
      if has_vault {
        keys.insert(k);
      }
    }
  }
  for binding in &deployment.config.secret_bindings {
    if !binding.variable.trim().is_empty() {
      keys.insert(binding.variable.trim().to_string());
    }
  }
  keys
}

/// Resolve all secret-bearing env var names for a `Stack` (live config unioned with last deployed metadata).
pub async fn stack_secret_env_keys(
  stack: &Stack,
) -> BTreeSet<String> {
  let mut keys = configured_stack_secret_env_keys(stack);
  if let Ok(Some(meta)) =
    get_resource_deployment_metadata("Stack", &stack.id).await
  {
    keys.extend(meta.secret_env_keys);
  }
  keys
}

/// Resolve all secret-bearing env var names for a `Deployment` (live config unioned with last deployed metadata).
pub async fn deployment_secret_env_keys(
  deployment: &Deployment,
) -> BTreeSet<String> {
  let mut keys = configured_deployment_secret_env_keys(deployment);
  if let Ok(Some(meta)) =
    get_resource_deployment_metadata("Deployment", &deployment.id)
      .await
  {
    keys.extend(meta.secret_env_keys);
  }
  keys
}

fn container_matches_compose_project(
  container_name: &str,
  compose_project_label: Option<&str>,
  project_name: &str,
) -> bool {
  if project_name.is_empty() {
    return false;
  }
  if compose_project_label == Some(project_name) {
    return true;
  }
  container_name == project_name
    || container_name.starts_with(&format!("{project_name}-"))
    || container_name.starts_with(&format!("{project_name}_"))
}

/// Resolve secret-bearing env var names for an arbitrary container on `server_id`
/// (used by Server-level `InspectContainer`, `GetContainerLog`, and `SearchContainerLog`).
pub async fn server_container_secret_env_keys(
  server_id: &str,
  requested_name: &str,
  inspected: Option<&Container>,
) -> BTreeSet<String> {
  let mut keys = BTreeSet::new();
  let container_name = inspected
    .and_then(|c| c.name.as_deref())
    .map(|n| n.trim_start_matches('/'))
    .unwrap_or_else(|| requested_name.trim_start_matches('/'));
  let compose_project = inspected
    .and_then(|c| c.config.as_ref())
    .and_then(|cfg| cfg.labels.get("com.docker.compose.project"))
    .map(String::as_str);

  if let Ok(metas) = find_collect(
    &db_client().secret_deployment_metadata,
    doc! { "server_id": server_id },
    None,
  )
  .await
  {
    for meta in metas {
      let matched = match meta.resource_type.as_str() {
        "Deployment" => {
          meta.runtime_project_or_container == container_name
        }
        "Stack" => container_matches_compose_project(
          container_name,
          compose_project,
          &meta.runtime_project_or_container,
        ),
        _ => false,
      };
      if matched {
        keys.extend(meta.secret_env_keys);
      }
    }
  }

  let cache = all_resources_cache().load();
  for deployment in cache.deployments.values() {
    if deployment.config.server_id == server_id
      && (deployment.deployed_name() == container_name
        || deployment.custom_name() == container_name)
    {
      keys.extend(configured_deployment_secret_env_keys(deployment));
    }
  }
  for stack in cache.stacks.values() {
    if stack.config.server_id == server_id
      && (container_matches_compose_project(
        container_name,
        compose_project,
        &stack.project_name(false),
      ) || container_matches_compose_project(
        container_name,
        compose_project,
        &stack.project_name(true),
      ))
    {
      keys.extend(configured_stack_secret_env_keys(stack));
    }
  }

  keys
}

#[cfg(test)]
mod tests {
  use super::*;
  use komodo_client::entities::docker::{
    ContainerConfig, service::ServiceSpec,
    task::TaskSpecContainerSpec,
  };

  #[test]
  fn redacts_compose_merged_config_mapping_and_sequence_envs() {
    let raw_yaml = r#"
name: demo
services:
  api:
    image: ghcr.io/example/api:latest
    environment:
      APP_ENV: production
      DATABASE_URL: postgres://user:p%40ss%23word@db:5432/app
      SHORT_PIN: "1"
  worker:
    image: ghcr.io/example/worker:latest
    environment:
      - QUEUE_NAME=jobs
      - API_SECRET=super-secret-token
secrets:
  tls_key:
    file: /etc/komodo/secret-runtime/stacks/demo/generations/g1/files/tls_key
"#;
    let redacted = redact_compose_merged_config_yaml(
      raw_yaml,
      &[
        String::from("DATABASE_URL"),
        String::from("SHORT_PIN"),
        String::from("API_SECRET"),
      ],
    );
    assert!(redacted.contains("APP_ENV: production"));
    assert!(redacted.contains("QUEUE_NAME=jobs"));
    assert!(!redacted.contains("p%40ss%23word"));
    assert!(!redacted.contains("super-secret-token"));
    assert!(!redacted.contains("/secret-runtime/stacks/"));
    assert!(redacted.contains("DATABASE_URL: <REDACTED>"));
    assert!(redacted.contains("SHORT_PIN: <REDACTED>"));
    assert!(redacted.contains("API_SECRET=<REDACTED>"));
    assert!(redacted.contains("<MANAGED_RUNTIME_SECRET_FILE>"));
  }

  #[test]
  fn masks_container_and_swarm_service_env_keys() {
    let mut keys = BTreeSet::new();
    keys.insert(String::from("DATABASE_URL"));
    keys.insert(String::from("JWT_SECRET"));

    let mut container = Container {
      config: Some(ContainerConfig {
        env: vec![
          String::from("PUBLIC_MODE=1"),
          String::from("DATABASE_URL=postgres://u:secret@db/app"),
          String::from("MY_DATABASE_URL=keep-me"),
        ],
        ..Default::default()
      }),
      ..Default::default()
    };
    mask_container_env(&mut container, &keys);
    let env = &container.config.unwrap().env;
    assert_eq!(env[0], "PUBLIC_MODE=1");
    assert_eq!(env[1], "DATABASE_URL=<REDACTED>");
    assert_eq!(env[2], "MY_DATABASE_URL=keep-me");

    let mut svc = SwarmService {
      spec: Some(ServiceSpec {
        task_template: Some(TaskSpec {
          container_spec: Some(TaskSpecContainerSpec {
            env: Some(vec![
              String::from("JWT_SECRET=top-secret"),
              String::from("PORT=8080"),
            ]),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      }),
      ..Default::default()
    };
    mask_swarm_service_env(&mut svc, &keys);
    let s_env = svc
      .spec
      .unwrap()
      .task_template
      .unwrap()
      .container_spec
      .unwrap()
      .env
      .unwrap();
    assert_eq!(s_env[0], "JWT_SECRET=<REDACTED>");
    assert_eq!(s_env[1], "PORT=8080");
  }

  #[test]
  fn redacts_runtime_log_env_assignments_preserving_other_lines() {
    let mut keys = BTreeSet::new();
    keys.insert(String::from("DATABASE_URL"));
    keys.insert(String::from("API_TOKEN"));

    let mut log = Log::simple(
      "Get Stack Log",
      String::from(
        "api-1  | Starting server on :8080\napi-1  | DATABASE_URL=postgres://u:secret@db:5432/app\nexport API_TOKEN=tok_live_123\nMY_API_TOKEN=public\n",
      ),
    );
    redact_log_env_assignments(&mut log, &keys);
    assert!(log.stdout.contains("Starting server on :8080"));
    assert!(log.stdout.contains("api-1  | DATABASE_URL=<REDACTED>"));
    assert!(log.stdout.contains("export API_TOKEN=<REDACTED>"));
    assert!(log.stdout.contains("MY_API_TOKEN=public"));
    assert!(!log.stdout.contains("secret@db"));
    assert!(!log.stdout.contains("tok_live_123"));
  }
}
