use std::collections::HashMap;

use anyhow::Context;
use formatting::{Color, bold, colored, muted};
use komodo_client::{
  api::write::*,
  entities::{
    sync::DiffData, update::Log, user::sync_user, variable::Variable,
  },
};
use mungos::find::find_collect;
use resolver_api::Resolve;

use crate::{api::write::WriteArgs, state::db_client};

use super::toml::TOML_PRETTY_OPTIONS;

pub fn variable_to_toml(
  variable: &Variable,
) -> anyhow::Result<String> {
  let inner = toml_pretty::to_string(variable, TOML_PRETTY_OPTIONS)
    .context("failed to serialize variable to toml")?;
  Ok(format!("[[variable]]\n{inner}"))
}

pub struct ToUpdateItem {
  pub variable: Variable,
  pub update_value: bool,
  pub update_description: bool,
  pub update_is_secret: bool,
}

pub async fn get_updates_for_view(
  variables: &[Variable],
  delete: bool,
) -> anyhow::Result<Vec<DiffData>> {
  let map = find_collect(&db_client().variables, None, None)
    .await
    .context("failed to query db for variables")?
    .into_iter()
    .map(|v| (v.name.clone(), v))
    .collect::<HashMap<_, _>>();

  let mut diffs = Vec::<DiffData>::new();

  if delete {
    for variable in map.values() {
      if !variables.iter().any(|v| v.name == variable.name) {
        diffs.push(DiffData::Delete {
          current: variable_to_toml(variable)?,
        });
      }
    }
  }

  for variable in variables {
    match map.get(&variable.name) {
      Some(original) => {
        if original.value == variable.value
          && original.description == variable.description
        {
          continue;
        }
        diffs.push(DiffData::Update {
          proposed: variable_to_toml(variable)?,
          current: variable_to_toml(original)?,
        });
      }
      None => {
        diffs.push(DiffData::Create {
          name: variable.name.clone(),
          proposed: variable_to_toml(variable)?,
        });
      }
    }
  }

  Ok(diffs)
}

pub async fn get_updates_for_execution(
  variables: Vec<Variable>,
  delete: bool,
) -> anyhow::Result<(Vec<Variable>, Vec<ToUpdateItem>, Vec<String>)> {
  let map = find_collect(&db_client().variables, None, None)
    .await
    .context("failed to query db for variables")?
    .into_iter()
    .map(|v| (v.name.clone(), v))
    .collect::<HashMap<_, _>>();

  let mut to_create = Vec::<Variable>::new();
  let mut to_update = Vec::<ToUpdateItem>::new();
  let mut to_delete = Vec::<String>::new();

  if delete {
    for variable in map.values() {
      if !variables.iter().any(|v| v.name == variable.name) {
        to_delete.push(variable.name.clone());
      }
    }
  }

  for variable in variables {
    match map.get(&variable.name) {
      Some(original) => {
        let item = ToUpdateItem {
          update_value: original.value != variable.value,
          update_description: original.description
            != variable.description,
          update_is_secret: original.is_secret != variable.is_secret,
          variable,
        };
        if !item.update_value
          && !item.update_description
          && !item.update_is_secret
        {
          continue;
        }

        to_update.push(item);
      }
      None => to_create.push(variable),
    }
  }

  Ok((to_create, to_update, to_delete))
}

pub async fn run_updates(
  to_create: Vec<Variable>,
  to_update: Vec<ToUpdateItem>,
  to_delete: Vec<String>,
) -> Option<Log> {
  if to_create.is_empty()
    && to_update.is_empty()
    && to_delete.is_empty()
  {
    return None;
  }

  let mut has_error = false;
  let mut log = String::from("running updates on Variables");

  for variable in to_create {
    if let Err(e) = (CreateVariable {
      name: variable.name.clone(),
      value: variable.value,
      description: variable.description,
      is_secret: variable.is_secret,
    })
    .resolve(&WriteArgs {
      user: sync_user().to_owned(),
    })
    .await
    {
      has_error = true;
      log.push_str(&format!(
        "\n{}: failed to create variable '{}' | {:#}",
        colored("ERROR", Color::Red),
        bold(&variable.name),
        e.error
      ));
    } else {
      log.push_str(&format!(
        "\n{}: {} variable '{}'",
        muted("INFO"),
        colored("created", Color::Green),
        bold(&variable.name)
      ))
    };
  }

  for ToUpdateItem {
    variable,
    update_value,
    update_description,
    update_is_secret,
  } in to_update
  {
    if update_value {
      if let Err(e) = (UpdateVariableValue {
        name: variable.name.clone(),
        value: variable.value,
      })
      .resolve(&WriteArgs {
        user: sync_user().to_owned(),
      })
      .await
      {
        has_error = true;
        log.push_str(&format!(
          "\n{}: failed to update variable value for '{}' | {:#}",
          colored("ERROR", Color::Red),
          bold(&variable.name),
          e.error
        ))
      } else {
        log.push_str(&format!(
          "\n{}: {} variable '{}' value",
          muted("INFO"),
          colored("updated", Color::Blue),
          bold(&variable.name)
        ))
      };
    }
    if update_description {
      if let Err(e) = (UpdateVariableDescription {
        name: variable.name.clone(),
        description: variable.description,
      })
      .resolve(&WriteArgs {
        user: sync_user().to_owned(),
      })
      .await
      {
        has_error = true;
        log.push_str(&format!(
          "\n{}: failed to update variable description for '{}' | {:#}",
          colored("ERROR", Color::Red),
          bold(&variable.name),
          e.error
        ))
      } else {
        log.push_str(&format!(
          "\n{}: {} variable '{}' description",
          muted("INFO"),
          colored("updated", Color::Blue),
          bold(&variable.name)
        ))
      };
    }
    if update_is_secret {
      if let Err(e) = (UpdateVariableIsSecret {
        name: variable.name.clone(),
        is_secret: variable.is_secret,
      })
      .resolve(&WriteArgs {
        user: sync_user().to_owned(),
      })
      .await
      {
        has_error = true;
        log.push_str(&format!(
          "\n{}: failed to update variable is secret for '{}' | {:#}",
          colored("ERROR", Color::Red),
          bold(&variable.name),
          e.error,
        ))
      } else {
        log.push_str(&format!(
          "\n{}: {} variable '{}' is secret",
          muted("INFO"),
          colored("updated", Color::Blue),
          bold(&variable.name)
        ))
      };
    }
  }

  for variable in to_delete {
    if let Err(e) = (DeleteVariable {
      name: variable.clone(),
    })
    .resolve(&WriteArgs {
      user: sync_user().to_owned(),
    })
    .await
    {
      has_error = true;
      log.push_str(&format!(
        "\n{}: failed to delete variable '{}' | {:#}",
        colored("ERROR", Color::Red),
        bold(&variable),
        e.error
      ))
    } else {
      log.push_str(&format!(
        "\n{}: {} variable '{}'",
        muted("INFO"),
        colored("deleted", Color::Red),
        bold(&variable)
      ))
    }
  }

  let stage = "Update Variables";
  Some(if has_error {
    Log::error(stage, log)
  } else {
    Log::simple(stage, log)
  })
}
