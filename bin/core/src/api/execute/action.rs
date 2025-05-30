use std::{
  collections::HashSet,
  path::{Path, PathBuf},
  str::FromStr,
  sync::OnceLock,
};

use anyhow::Context;
use command::run_komodo_command;
use komodo_client::{
  api::{
    execute::{BatchExecutionResponse, BatchRunAction, RunAction},
    user::{CreateApiKey, CreateApiKeyResponse, DeleteApiKey},
  },
  entities::{
    action::Action,
    alert::{Alert, AlertData, SeverityLevel},
    config::core::CoreConfig,
    komodo_timestamp,
    permission::PermissionLevel,
    update::Update,
    user::action_user,
  },
};
use mungos::{by_id::update_one_by_id, mongodb::bson::to_document};
use resolver_api::Resolve;
use tokio::fs;

use crate::{
  alert::send_alerts,
  api::{execute::ExecuteRequest, user::UserArgs},
  config::core_config,
  helpers::{
    interpolate::{
      add_interp_update_log,
      interpolate_variables_secrets_into_string,
    },
    query::get_variables_and_secrets,
    random_string,
    update::update_update,
  },
  permission::get_check_permissions,
  resource::refresh_action_state_cache,
  state::{action_states, db_client},
};

use super::ExecuteArgs;

impl super::BatchExecute for BatchRunAction {
  type Resource = Action;
  fn single_request(action: String) -> ExecuteRequest {
    ExecuteRequest::RunAction(RunAction { action })
  }
}

impl Resolve<ExecuteArgs> for BatchRunAction {
  #[instrument(name = "BatchRunAction", skip(self, user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchRunAction>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for RunAction {
  #[instrument(name = "RunAction", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let mut action = get_check_permissions::<Action>(
      &self.action,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the action (or insert default).
    let action_state = action_states()
      .action
      .get_or_insert_default(&action.id)
      .await;

    // This will set action state back to default when dropped.
    // Will also check to ensure action not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.running = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let CreateApiKeyResponse { key, secret } = CreateApiKey {
      name: update.id.clone(),
      expires: 0,
    }
    .resolve(&UserArgs {
      user: action_user().to_owned(),
    })
    .await?;

    let contents = &mut action.config.file_contents;

    // Wrap the file contents in the execution context.
    *contents = full_contents(contents, &key, &secret);

    let replacers =
      interpolate(contents, &mut update, key.clone(), secret.clone())
        .await?
        .into_iter()
        .collect::<Vec<_>>();

    let file = format!("{}.ts", random_string(10));
    let path = core_config().action_directory.join(&file);

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)
        .await
        .with_context(|| format!("Failed to initialize Action file parent directory {parent:?}"))?;
    }

    fs::write(&path, contents).await.with_context(|| {
      format!("Failed to write action file to {path:?}")
    })?;

    let CoreConfig { ssl_enabled, .. } = core_config();

    let https_cert_flag = if *ssl_enabled {
      " --unsafely-ignore-certificate-errors=localhost"
    } else {
      ""
    };

    let reload = if action.config.reload_deno_deps {
      " --reload"
    } else {
      ""
    };

    let mut res = run_komodo_command(
      // Keep this stage name as is, the UI will find the latest update log by matching the stage name
      "Execute Action",
      None,
      format!(
        "deno run --allow-all{https_cert_flag}{reload} {}",
        path.display()
      ),
    )
    .await;

    res.stdout = svi::replace_in_string(&res.stdout, &replacers)
      .replace(&key, "<ACTION_API_KEY>");
    res.stderr = svi::replace_in_string(&res.stderr, &replacers)
      .replace(&secret, "<ACTION_API_SECRET>");

    cleanup_run(file + ".js", &path).await;

    if let Err(e) = (DeleteApiKey { key })
      .resolve(&UserArgs {
        user: action_user().to_owned(),
      })
      .await
    {
      warn!(
        "Failed to delete API key after action execution | {:#}",
        e.error
      );
    };

    update.logs.push(res);
    update.finalize();

    // Need to manually update the update before cache refresh,
    // and before broadcast with update_update.
    // The Err case of to_document should be unreachable,
    // but will fail to update cache in that case.
    if let Ok(update_doc) = to_document(&update) {
      let _ = update_one_by_id(
        &db_client().updates,
        &update.id,
        mungos::update::Update::Set(update_doc),
        None,
      )
      .await;
      refresh_action_state_cache().await;
    }

    update_update(update.clone()).await?;

    if !update.success && action.config.failure_alert {
      warn!("action unsuccessful, alerting...");
      let target = update.target.clone();
      tokio::spawn(async move {
        let alert = Alert {
          id: Default::default(),
          target,
          ts: komodo_timestamp(),
          resolved_ts: Some(komodo_timestamp()),
          resolved: true,
          level: SeverityLevel::Warning,
          data: AlertData::ActionFailed {
            id: action.id,
            name: action.name,
          },
        };
        send_alerts(&[alert]).await
      });
    }

    Ok(update)
  }
}

async fn interpolate(
  contents: &mut String,
  update: &mut Update,
  key: String,
  secret: String,
) -> serror::Result<HashSet<(String, String)>> {
  let mut vars_and_secrets = get_variables_and_secrets().await?;

  vars_and_secrets
    .secrets
    .insert(String::from("ACTION_API_KEY"), key);
  vars_and_secrets
    .secrets
    .insert(String::from("ACTION_API_SECRET"), secret);

  let mut global_replacers = HashSet::new();
  let mut secret_replacers = HashSet::new();

  interpolate_variables_secrets_into_string(
    &vars_and_secrets,
    contents,
    &mut global_replacers,
    &mut secret_replacers,
  )?;

  add_interp_update_log(update, &global_replacers, &secret_replacers);

  Ok(secret_replacers)
}

fn full_contents(contents: &str, key: &str, secret: &str) -> String {
  let CoreConfig {
    port, ssl_enabled, ..
  } = core_config();
  let protocol = if *ssl_enabled { "https" } else { "http" };
  let base_url = format!("{protocol}://localhost:{port}");
  format!(
    "import {{ KomodoClient, Types }} from '{base_url}/client/lib.js';
import * as __YAML__ from 'jsr:@std/yaml';
import * as __TOML__ from 'jsr:@std/toml';

const YAML = {{
  stringify: __YAML__.stringify,
  parse: __YAML__.parse,
  parseAll: __YAML__.parseAll,
  parseDockerCompose: __YAML__.parse,
}}

const TOML = {{
  stringify: __TOML__.stringify,
  parse: __TOML__.parse,
  parseResourceToml: __TOML__.parse,
  parseCargoToml: __TOML__.parse,
}}

const komodo = KomodoClient('{base_url}', {{
  type: 'api-key',
  params: {{ key: '{key}', secret: '{secret}' }}
}});

async function main() {{
{contents}

console.log('🦎 Action completed successfully 🦎');
}}

main()
.catch(error => {{
  console.error('🚨 Action exited early with errors 🚨')
  if (error.status !== undefined && error.result !== undefined) {{
    console.error('Status:', error.status);
    console.error(JSON.stringify(error.result, null, 2));
  }} else {{
    console.error(error);
  }}
  Deno.exit(1)
}});"
  )
}

/// Cleans up file at given path.
/// ALSO if $DENO_DIR is set,
/// will clean up the generated file matching "file"
async fn cleanup_run(file: String, path: &Path) {
  if let Err(e) = fs::remove_file(path).await {
    warn!(
      "Failed to delete action file after action execution | {e:#}"
    );
  }
  // If $DENO_DIR is set (will be in container),
  // will clean up the generated file matching "file" (NOT under path)
  let Some(deno_dir) = deno_dir() else {
    return;
  };
  delete_file(deno_dir.join("gen/file"), file).await;
}

fn deno_dir() -> Option<&'static Path> {
  static DENO_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
  DENO_DIR
    .get_or_init(|| {
      let deno_dir = std::env::var("DENO_DIR").ok()?;
      PathBuf::from_str(&deno_dir).ok()
    })
    .as_deref()
}

/// file is just the terminating file path,
/// it may be nested multiple folder under path,
/// this will find the nested file and delete it.
/// Assumes the file is only there once.
fn delete_file(
  dir: PathBuf,
  file: String,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
{
  Box::pin(async move {
    let Ok(mut dir) = fs::read_dir(dir).await else {
      return false;
    };
    // Collect the nested folders for recursing
    // only after checking all the files in directory.
    let mut folders = Vec::<PathBuf>::new();

    while let Ok(Some(entry)) = dir.next_entry().await {
      let Ok(meta) = entry.metadata().await else {
        continue;
      };
      if meta.is_file() {
        let Ok(name) = entry.file_name().into_string() else {
          continue;
        };
        if name == file {
          if let Err(e) = fs::remove_file(entry.path()).await {
            warn!(
              "Failed to clean up generated file after action execution | {e:#}"
            );
          };
          return true;
        }
      } else {
        folders.push(entry.path());
      }
    }

    if folders.len() == 1 {
      // unwrap ok, folders definitely is not empty
      let folder = folders.pop().unwrap();
      delete_file(folder, file).await
    } else {
      // Check folders with file.clone
      for folder in folders {
        if delete_file(folder, file.clone()).await {
          return true;
        }
      }
      false
    }
  })
}
