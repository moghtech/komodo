use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::{delete_one_by_id, find_one_by_id, update_one_by_id},
  mongodb::bson::{doc, to_document},
};
use komodo_client::{
  api::write::*,
  entities::{
    Operation, ResourceTarget,
    provider::{DockerRegistryAccount, GitProviderAccount},
  },
};
use mogh_error::AddStatusCodeError;
use mogh_resolver::Resolve;
use reqwest::StatusCode;

use crate::{
  helpers::update::{add_update, make_update},
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateGitProviderAccount {
  #[instrument(
    "CreateGitProviderAccount",
    skip_all,
    fields(
      operator = user.id,
      domain = self.account.domain,
      username = self.account.username,
      https = self.account.https.unwrap_or(true),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CreateGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can create git provider accounts")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let mut account: GitProviderAccount = self.account.into();

    if account.domain.is_empty() {
      return Err(
        anyhow!("Domain cannot be empty string.")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    if account.username.is_empty() {
      return Err(
        anyhow!("Username cannot be empty string.")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateGitProviderAccount,
      user,
    );

    account.id = db_client()
      .git_accounts
      .insert_one(&account)
      .await
      .context("Failed to create git provider account on db")?
      .inserted_id
      .as_object_id()
      .context("Inserted id is not ObjectId")?
      .to_string();

    update.push_simple_log(
      "Create git provider account",
      format!(
        "Created git provider account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("Failed to add update for create git provider account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for UpdateGitProviderAccount {
  #[instrument(
    "UpdateGitProviderAccount",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
      domain = self.account.domain,
      username = self.account.username,
      https = self.account.https.unwrap_or(true),
    )
  )]
  async fn resolve(
    mut self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can update git provider accounts")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    if let Some(domain) = &self.account.domain
      && domain.is_empty()
    {
      return Err(
        anyhow!("Cannot update git provider with empty domain")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    if let Some(username) = &self.account.username
      && username.is_empty()
    {
      return Err(
        anyhow!("Cannot update git provider with empty username")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    // Ensure update does not change id
    self.account.id = None;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateGitProviderAccount,
      user,
    );

    let account = to_document(&self.account).context(
      "Failed to serialize partial git provider account to bson",
    )?;
    let db = db_client();
    update_one_by_id(
      &db.git_accounts,
      &self.id,
      doc! { "$set": account },
      None,
    )
    .await
    .context("Failed to update git provider account on db")?;

    let Some(account) = find_one_by_id(&db.git_accounts, &self.id)
      .await
      .context("Failed to query db for git accounts")?
    else {
      return Err(anyhow!("No account found with given id").into());
    };

    update.push_simple_log(
      "Update git provider account",
      format!(
        "Updated git provider account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("Failed to add update for update git provider account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for DeleteGitProviderAccount {
  #[instrument(
    "DeleteGitProviderAccount",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<DeleteGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can delete git provider accounts")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateGitProviderAccount,
      user,
    );

    let db = db_client();
    let Some(account) = find_one_by_id(&db.git_accounts, &self.id)
      .await
      .context("Failed to query db for git accounts")?
    else {
      return Err(
        anyhow!("No account found with given id")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };
    delete_one_by_id(&db.git_accounts, &self.id, None)
      .await
      .context("failed to delete git account on db")?;

    update.push_simple_log(
      "Delete git provider account",
      format!(
        "Deleted git provider account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("Failed to add update for delete git provider account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for CreateDockerRegistryAccount {
  #[instrument(
    "CreateDockerRegistryAccount",
    skip_all,
    fields(
      operator = user.id,
      domain = self.account.domain,
      username = self.account.username,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CreateDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!(
          "Only admins can create docker registry account accounts"
        )
        .status_code(StatusCode::FORBIDDEN),
      );
    }

    let mut account: DockerRegistryAccount = self.account.into();

    if account.domain.is_empty() {
      return Err(
        anyhow!("Domain cannot be empty string.")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    if account.username.is_empty() {
      return Err(
        anyhow!("Username cannot be empty string.")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateDockerRegistryAccount,
      user,
    );

    account.id = db_client()
      .registry_accounts
      .insert_one(&account)
      .await
      .context(
        "Failed to create docker registry account account on db",
      )?
      .inserted_id
      .as_object_id()
      .context("Inserted id is not ObjectId")?
      .to_string();

    update.push_simple_log(
      "Create docker registry account",
      format!(
        "Created docker registry account account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("Failed to add update for create docker registry account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for UpdateDockerRegistryAccount {
  #[instrument(
    "UpdateDockerRegistryAccount",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
      domain = self.account.domain,
      username = self.account.username,
    )
  )]
  async fn resolve(
    mut self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can update docker registry accounts")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    if let Some(domain) = &self.account.domain
      && domain.is_empty()
    {
      return Err(
        anyhow!(
          "Cannot update docker registry account with empty domain"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    }

    if let Some(username) = &self.account.username
      && username.is_empty()
    {
      return Err(
        anyhow!(
          "Cannot update docker registry account with empty username"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    }

    self.account.id = None;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateDockerRegistryAccount,
      user,
    );

    let account = to_document(&self.account).context(
      "Failed to serialize partial docker registry account account to bson",
    )?;

    let db = db_client();
    update_one_by_id(
      &db.registry_accounts,
      &self.id,
      doc! { "$set": account },
      None,
    )
    .await
    .context(
      "Failed to update docker registry account account on db",
    )?;

    let Some(account) =
      find_one_by_id(&db.registry_accounts, &self.id)
        .await
        .context("Failed to query db for registry accounts")?
    else {
      return Err(anyhow!("No account found with given id").into());
    };

    update.push_simple_log(
      "Update docker registry account",
      format!(
        "Updated docker registry account account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("Failed to add update for update docker registry account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for DeleteDockerRegistryAccount {
  #[instrument(
    "DeleteDockerRegistryAccount",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<DeleteDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can delete docker registry accounts")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateDockerRegistryAccount,
      user,
    );

    let db = db_client();
    let Some(account) =
      find_one_by_id(&db.registry_accounts, &self.id)
        .await
        .context("Failed to query db for git accounts")?
    else {
      return Err(
        anyhow!("No account found with given id")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };
    delete_one_by_id(&db.registry_accounts, &self.id, None)
      .await
      .context("Failed to delete registry account on db")?;

    update.push_simple_log(
      "Delete registry account",
      format!(
        "Deleted registry account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("Failed to add update for delete docker registry account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}
