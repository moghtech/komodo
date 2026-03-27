use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::doc;
use komodo_client::{
  api::write::*,
  entities::{Operation, ResourceTarget, variable::Variable},
};
use mogh_error::{AddStatusCode as _, AddStatusCodeError};
use mogh_resolver::Resolve;
use reqwest::StatusCode;

use crate::{
  helpers::{
    query::get_variable,
    update::{add_update, make_update},
    validations::{validate_variable_name, validate_variable_value},
  },
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateVariable {
  #[instrument(
    "CreateVariable",
    skip_all,
    fields(
      operator = user.id,
      variable = self.name,
      description = self.description,
      is_secret = self.is_secret,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CreateVariableResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can create Variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let CreateVariable {
      name,
      value,
      description,
      is_secret,
    } = self;

    validate_variable_name(&name)
      .status_code(StatusCode::BAD_REQUEST)?;
    validate_variable_value(&value)
      .status_code(StatusCode::BAD_REQUEST)?;

    let variable = Variable {
      name,
      value,
      description,
      is_secret,
    };

    db_client()
      .variables
      .insert_one(&variable)
      .await
      .context("Failed to create Variable on db")?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateVariable,
      user,
    );

    update
      .push_simple_log("Create Variable", format!("{variable:#?}"));

    update.finalize();

    add_update(update).await?;

    Ok(get_variable(&variable.name).await?)
  }
}

impl Resolve<WriteArgs> for UpdateVariableValue {
  #[instrument(
    "UpdateVariableValue",
    skip_all,
    fields(
      operator = user.id,
      variable = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateVariableValueResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can update Variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let UpdateVariableValue { name, value } = self;

    validate_variable_name(&name)
      .status_code(StatusCode::BAD_REQUEST)?;
    validate_variable_value(&value)
      .status_code(StatusCode::BAD_REQUEST)?;

    let variable = get_variable(&name).await?;

    if value == variable.value {
      return Ok(variable);
    }

    db_client()
      .variables
      .update_one(
        doc! { "name": &name },
        doc! { "$set": { "value": &value } },
      )
      .await
      .context("Failed to update variable value on db")?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateVariableValue,
      user,
    );

    let log = if variable.is_secret {
      format!(
        "<span class=\"text-muted-foreground\">variable</span>: '{name}'\n<span class=\"text-muted-foreground\">from</span>: <span class=\"text-red-500\">{}</span>\n<span class=\"text-muted-foreground\">to</span>:   <span class=\"text-green-500\">{value}</span>",
        variable.value.replace(|_| true, "#")
      )
    } else {
      format!(
        "<span class=\"text-muted-foreground\">variable</span>: '{name}'\n<span class=\"text-muted-foreground\">from</span>: <span class=\"text-red-500\">{}</span>\n<span class=\"text-muted-foreground\">to</span>:   <span class=\"text-green-500\">{value}</span>",
        variable.value
      )
    };

    update.push_simple_log("Update Variable Value", log);
    update.finalize();

    add_update(update).await?;

    Ok(get_variable(&name).await?)
  }
}

impl Resolve<WriteArgs> for UpdateVariableDescription {
  #[instrument(
    "UpdateVariableDescription",
    skip_all,
    fields(
      operator = user.id,
      variable = self.name,
      description = self.description,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateVariableDescriptionResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can update Variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    db_client()
      .variables
      .update_one(
        doc! { "name": &self.name },
        doc! { "$set": { "description": &self.description } },
      )
      .await
      .context("Failed to update variable description on db")?;

    Ok(get_variable(&self.name).await?)
  }
}

impl Resolve<WriteArgs> for UpdateVariableIsSecret {
  #[instrument(
    "UpdateVariableIsSecret",
    skip_all,
    fields(
      operator = user.id,
      variable = self.name,
      is_secret = self.is_secret,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateVariableIsSecretResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can update Variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    db_client()
      .variables
      .update_one(
        doc! { "name": &self.name },
        doc! { "$set": { "is_secret": self.is_secret } },
      )
      .await
      .context("Failed to update Variable 'is_secret' on db")?;

    Ok(get_variable(&self.name).await?)
  }
}

impl Resolve<WriteArgs> for DeleteVariable {
  #[instrument(
    "DeleteVariable",
    skip_all,
    fields(
      operator = user.id,
      variable = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<DeleteVariableResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can delete Variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let variable = get_variable(&self.name).await?;

    db_client()
      .variables
      .delete_one(doc! { "name": &self.name })
      .await
      .context("Failed to delete Variable on db")?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::DeleteVariable,
      user,
    );

    update
      .push_simple_log("Delete Variable", format!("{variable:#?}"));
    update.finalize();

    add_update(update).await?;

    Ok(variable)
  }
}
