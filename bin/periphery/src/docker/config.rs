use anyhow::{Context, anyhow};
use command::run_komodo_standard_command;
use komodo_client::entities::docker::config::{
  SwarmConfig, SwarmConfigListItem,
};

pub async fn list_swarm_configs()
-> anyhow::Result<Vec<SwarmConfigListItem>> {
  let res = run_komodo_standard_command(
    "List Swarm Configs",
    None,
    "docker config ls --format json",
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(
      "Failed to list swarm configs using 'docker config ls'",
    ));
  }

  // The output is in JSONL, need to convert to standard JSON vec.
  let mut res = serde_json::from_str::<Vec<SwarmConfigListItem>>(
    &format!("[{}]", res.stdout.trim().replace('\n', ",")),
  )
  .context("Failed to parse 'docker config ls' response from json")?;

  res.sort_by(|a, b| {
    a.name
      .cmp(&b.name)
      .then_with(|| b.updated_at.cmp(&a.updated_at))
  });

  Ok(res)
}

pub async fn inspect_swarm_config(
  config: &str,
) -> anyhow::Result<Vec<SwarmConfig>> {
  let res = run_komodo_standard_command(
    "Inspect Swarm Config",
    None,
    format!(r#"docker config inspect "{config}""#),
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(format!(
      "Failed to inspect swarm config using 'docker config inspect {config}'"
    )));
  }

  serde_json::from_str(&res.stdout).context(
    "Failed to parse 'docker config inspect' response from json",
  )
}
