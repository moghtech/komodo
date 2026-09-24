use std::time::Duration;

use anyhow::{Context, anyhow};
use command::{CommandOptions, run_komodo_standard_command};
use komodo_client::entities::stack::*;
use serde::{Deserialize, Serialize};

use crate::config::periphery_config;

pub fn docker_compose() -> &'static str {
  if periphery_config().legacy_compose_cli {
    "docker-compose"
  } else {
    "docker compose"
  }
}

pub async fn list_compose_projects()
-> anyhow::Result<Vec<ComposeProject>> {
  let docker_compose = docker_compose();
  let res = run_komodo_standard_command(
    "List Projects",
    format!("{docker_compose} ls --all --format json"),
    CommandOptions::default().timeout(Duration::from_secs(5)),
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(format!(
      "Failed to list compose projects using {docker_compose} ls"
    )));
  }

  let mut res =
    serde_json::from_str::<Vec<DockerComposeLsItem>>(&res.stdout)
      .with_context(|| res.stdout.clone())
      .with_context(|| {
        format!(
          "Failed to parse '{docker_compose} ls' response from json"
        )
      })?
      .into_iter()
      .filter(|item| !item.name.is_empty())
      .map(|item| ComposeProject {
        name: item.name,
        status: item.status,
        compose_files: item
          .config_files
          .split(',')
          .map(str::to_string)
          .collect(),
      })
      .collect::<Vec<_>>();

  res.sort_by(|a, b| {
    a.status.cmp(&b.status).then_with(|| a.name.cmp(&b.name))
  });

  Ok(res)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeLsItem {
  #[serde(default, alias = "Name")]
  pub name: String,
  #[serde(alias = "Status")]
  pub status: Option<String>,
  /// Comma seperated list of paths
  #[serde(default, alias = "ConfigFiles")]
  pub config_files: String,
}

pub fn parse_compose_services(
  raw_config: &str,
  project_name: &str,
  services: &mut Vec<StackServiceNames>,
) -> anyhow::Result<()> {
  let compose = serde_yaml_ng::from_str::<ComposeFile>(raw_config)
    .context("Failed to parse compose contents")?;

  for (
    service_name,
    ComposeService {
      container_name,
      deploy,
      image,
    },
  ) in compose.services
  {
    let image = image.unwrap_or_default();
    match deploy {
      Some(ComposeServiceDeploy {
        replicas: Some(replicas),
      }) if replicas > 1 => {
        for i in 1..1 + replicas {
          services.push(StackServiceNames {
            container_name: format!(
              "{project_name}-{service_name}-{i}"
            ),
            service_name: format!("{service_name}-{i}"),
            image: image.clone(),
            image_digest: None,
          });
        }
      }
      _ => {
        services.push(StackServiceNames {
          container_name: container_name.unwrap_or_else(|| {
            format!("{project_name}-{service_name}")
          }),
          service_name,
          image,
          image_digest: None,
        });
      }
    }
  }

  Ok(())
}

/// Returns the `docker compose config` output with secrets removed,
/// safe to log or store. Values resolved from `env_file:` never pass
/// through Komodo's interpolator, so `replacers` alone cannot catch them.
pub fn redact_compose_config(
  raw_config: &str,
  replacers: &[(String, String)],
) -> String {
  use serde_yaml_ng::Value;

  const REDACTED: &str = "<redacted>";

  let Ok(mut config) = serde_yaml_ng::from_str::<Value>(raw_config)
  else {
    // Fail closed: output that cannot be parsed cannot be redacted.
    return String::from(
      "<compose config withheld: output could not be parsed for redaction>",
    );
  };

  if let Some(services) =
    config.get_mut("services").and_then(Value::as_mapping_mut)
  {
    for service in services.values_mut() {
      match service.get_mut("environment") {
        Some(Value::Mapping(environment)) => {
          for value in environment.values_mut() {
            if !value.is_null() {
              *value = Value::from(REDACTED);
            }
          }
        }
        Some(Value::Sequence(environment)) => {
          for entry in environment.iter_mut() {
            if let Some((key, _)) =
              entry.as_str().and_then(|entry| entry.split_once('='))
            {
              *entry = Value::from(format!("{key}={REDACTED}"));
            }
          }
        }
        _ => {}
      }
    }
  }

  let redacted = serde_yaml_ng::to_string(&config).unwrap_or_else(|_| {
    String::from(
      "<compose config withheld: redacted output could not be serialized>",
    )
  });
  svi::replace_in_string(&redacted, replacers)
}

#[cfg(test)]
mod tests {
  use super::*;

  const CANARY: &str = "sk_live_CANARY_7f3a91";

  fn resolved_config() -> String {
    format!(
      "name: leak-repro
services:
  app:
    command:
      - sleep
      - \"3600\"
    environment:
      CANARY_SECRET: {CANARY}
      EMPTY: \"\"
    image: busybox:1.36
    networks:
      default: null
  worker:
    environment:
      - LIST_SECRET={CANARY}
    image: busybox:1.36
networks:
  default:
    name: leak-repro_default
"
    )
  }

  #[test]
  fn redacts_environment_values_resolved_from_env_files() {
    let redacted = redact_compose_config(&resolved_config(), &[]);
    assert!(!redacted.contains(CANARY), "{redacted}");
  }

  #[test]
  fn keeps_environment_keys_and_non_secret_structure() {
    let redacted = redact_compose_config(&resolved_config(), &[]);
    assert!(redacted.contains("CANARY_SECRET"), "{redacted}");
    assert!(redacted.contains("LIST_SECRET"), "{redacted}");
    assert!(redacted.contains("image: busybox:1.36"), "{redacted}");
    let mut services = Vec::new();
    parse_compose_services(&redacted, "leak-repro", &mut services)
      .expect("redacted config still parses");
    assert_eq!(services.len(), 2);
  }

  #[test]
  fn applies_known_secret_replacers_outside_environment() {
    let config = "services:\n  app:\n    image: busybox\n    command: [\"run\", \"--token=tok_ABC123\"]\n";
    let redacted = redact_compose_config(
      config,
      &[(String::from("tok_ABC123"), String::from("<API_TOKEN>"))],
    );
    assert!(!redacted.contains("tok_ABC123"), "{redacted}");
    assert!(redacted.contains("<API_TOKEN>"), "{redacted}");
  }

  #[test]
  fn fails_closed_when_config_is_not_yaml() {
    let redacted = redact_compose_config(
      &format!("services: [unclosed\n  CANARY_SECRET: {CANARY}"),
      &[],
    );
    assert!(!redacted.contains(CANARY), "{redacted}");
  }
}
