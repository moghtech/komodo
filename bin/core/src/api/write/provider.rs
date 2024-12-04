use anyhow::{anyhow, Context};
use komodo_client::{
  api::write::*,
  entities::{
    provider::{DockerRegistryAccount, GitProviderAccount},
    Operation, ResourceTarget,
  },
};
use mungos::{
  by_id::{delete_one_by_id, find_one_by_id, update_one_by_id},
  mongodb::bson::{doc, to_document},
};
use resolver_api::Resolve;

use crate::{
  helpers::update::{add_update, make_update},
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateGitProviderAccount {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("only admins can create git provider accounts")
          .into(),
      );
    }

    let mut account: GitProviderAccount = self.account.into();

    if account.domain.is_empty() {
      return Err(anyhow!("domain cannot be empty string.").into());
    }

    if account.username.is_empty() {
      return Err(anyhow!("username cannot be empty string.").into());
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateGitProviderAccount,
      &user,
    );

    account.id = db_client()
      .git_accounts
      .insert_one(&account)
      .await
      .context("failed to create git provider account on db")?
      .inserted_id
      .as_object_id()
      .context("inserted id is not ObjectId")?
      .to_string();

    update.push_simple_log(
      "create git provider account",
      format!(
        "Created git provider account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("failed to add update for create git provider account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for UpdateGitProviderAccount {
  async fn resolve(
    mut self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("only admins can update git provider accounts")
          .into(),
      );
    }

    if let Some(domain) = &self.account.domain {
      if domain.is_empty() {
        return Err(
          anyhow!("cannot update git provider with empty domain")
            .into(),
        );
      }
    }

    if let Some(username) = &self.account.username {
      if username.is_empty() {
        return Err(
          anyhow!("cannot update git provider with empty username")
            .into(),
        );
      }
    }

    // Ensure update does not change id
    self.account.id = None;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateGitProviderAccount,
      &user,
    );

    let account = to_document(&self.account).context(
      "failed to serialize partial git provider account to bson",
    )?;
    let db = db_client();
    update_one_by_id(
      &db.git_accounts,
      &self.id,
      doc! { "$set": account },
      None,
    )
    .await
    .context("failed to update git provider account on db")?;

    let Some(account) = find_one_by_id(&db.git_accounts, &self.id)
      .await
      .context("failed to query db for git accounts")?
    else {
      return Err(anyhow!("no account found with given id").into());
    };

    update.push_simple_log(
      "update git provider account",
      format!(
        "Updated git provider account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("failed to add update for update git provider account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for DeleteGitProviderAccount {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("only admins can delete git provider accounts")
          .into(),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateGitProviderAccount,
      &user,
    );

    let db = db_client();
    let Some(account) = find_one_by_id(&db.git_accounts, &self.id)
      .await
      .context("failed to query db for git accounts")?
    else {
      return Err(anyhow!("no account found with given id").into());
    };
    delete_one_by_id(&db.git_accounts, &self.id, None)
      .await
      .context("failed to delete git account on db")?;

    update.push_simple_log(
      "delete git provider account",
      format!(
        "Deleted git provider account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("failed to add update for delete git provider account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for CreateDockerRegistryAccount {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!(
          "only admins can create docker registry account accounts"
        )
        .into(),
      );
    }

    let mut account: DockerRegistryAccount = self.account.into();

    if account.domain.is_empty() {
      return Err(anyhow!("domain cannot be empty string.").into());
    }

    if account.username.is_empty() {
      return Err(anyhow!("username cannot be empty string.").into());
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateDockerRegistryAccount,
      &user,
    );

    account.id = db_client()
      .registry_accounts
      .insert_one(&account)
      .await
      .context(
        "failed to create docker registry account account on db",
      )?
      .inserted_id
      .as_object_id()
      .context("inserted id is not ObjectId")?
      .to_string();

    update.push_simple_log(
      "create docker registry account",
      format!(
        "Created docker registry account account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("failed to add update for create docker registry account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for UpdateDockerRegistryAccount {
  async fn resolve(
    mut self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("only admins can update docker registry accounts")
          .into(),
      );
    }

    if let Some(domain) = &self.account.domain {
      if domain.is_empty() {
        return Err(
          anyhow!(
            "cannot update docker registry account with empty domain"
          )
          .into(),
        );
      }
    }

    if let Some(username) = &self.account.username {
      if username.is_empty() {
        return Err(
          anyhow!(
          "cannot update docker registry account with empty username"
        )
          .into(),
        );
      }
    }

    self.account.id = None;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateDockerRegistryAccount,
      &user,
    );

    let account = to_document(&self.account).context(
      "failed to serialize partial docker registry account account to bson",
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
      "failed to update docker registry account account on db",
    )?;

    let Some(account) =
      find_one_by_id(&db.registry_accounts, &self.id)
        .await
        .context("failed to query db for registry accounts")?
    else {
      return Err(anyhow!("no account found with given id").into());
    };

    update.push_simple_log(
      "update docker registry account",
      format!(
        "Updated docker registry account account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("failed to add update for update docker registry account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}

impl Resolve<WriteArgs> for DeleteDockerRegistryAccount {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("only admins can delete docker registry accounts")
          .into(),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateDockerRegistryAccount,
      &user,
    );

    let db = db_client();
    let Some(account) =
      find_one_by_id(&db.registry_accounts, &self.id)
        .await
        .context("failed to query db for git accounts")?
    else {
      return Err(anyhow!("no account found with given id").into());
    };
    delete_one_by_id(&db.registry_accounts, &self.id, None)
      .await
      .context("failed to delete registry account on db")?;

    update.push_simple_log(
      "delete registry account",
      format!(
        "Deleted registry account for {} with username {}",
        account.domain, account.username
      ),
    );

    update.finalize();

    add_update(update)
      .await
      .inspect_err(|e| {
        error!("failed to add update for delete docker registry account | {e:#}")
      })
      .ok();

    Ok(account)
  }
}
