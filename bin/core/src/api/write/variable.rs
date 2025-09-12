use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::doc;
use komodo_client::{
  api::write::*,
  entities::{Operation, ResourceTarget, variable::Variable},
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError;

use crate::{
  helpers::{
    query::get_variable,
    update::{add_update, make_update},
  },
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateVariable {
  #[instrument(name = "CreateVariable", skip(user, self), fields(name = &self.name))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateVariableResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can create variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let CreateVariable {
      name,
      value,
      description,
      is_secret,
    } = self;

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
      .context("Failed to create variable on db")?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateVariable,
      user,
    );

    update
      .push_simple_log("create variable", format!("{variable:#?}"));
    update.finalize();

    add_update(update).await?;

    Ok(get_variable(&variable.name).await?)
  }
}

impl Resolve<WriteArgs> for UpdateVariableValue {
  #[instrument(name = "UpdateVariableValue", skip(user, self), fields(name = &self.name))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateVariableValueResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can update variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let UpdateVariableValue { name, value } = self;

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
  #[instrument(name = "UpdateVariableDescription", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateVariableDescriptionResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can update variables")
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
  #[instrument(name = "UpdateVariableIsSecret", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateVariableIsSecretResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can update variables")
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
      .context("Failed to update variable is secret on db")?;
    Ok(get_variable(&self.name).await?)
  }
}

impl Resolve<WriteArgs> for DeleteVariable {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteVariableResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can delete variables")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    let variable = get_variable(&self.name).await?;
    db_client()
      .variables
      .delete_one(doc! { "name": &self.name })
      .await
      .context("Failed to delete variable on db")?;

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
