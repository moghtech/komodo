use anyhow::{anyhow, Context};
use komodo_client::api::read::*;
use mongo_indexed::{doc, Document};
use mungos::{
  by_id::find_one_by_id, find::find_collect,
  mongodb::options::FindOptions,
};
use resolver_api::Resolve;

use crate::state::db_client;

use super::ReadArgs;

impl Resolve<ReadArgs> for GetGitProviderAccount {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetGitProviderAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can read git provider accounts").into(),
      );
    }
    let res = find_one_by_id(&db_client().git_accounts, &self.id)
      .await
      .context("failed to query db for git provider accounts")?
      .context(
        "did not find git provider account with the given id",
      )?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListGitProviderAccounts {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListGitProviderAccountsResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can read git provider accounts").into(),
      );
    }
    let mut filter = Document::new();
    if let Some(domain) = self.domain {
      filter.insert("domain", domain);
    }
    if let Some(username) = self.username {
      filter.insert("username", username);
    }
    let res = find_collect(
      &db_client().git_accounts,
      filter,
      FindOptions::builder()
        .sort(doc! { "domain": 1, "username": 1 })
        .build(),
    )
    .await
    .context("failed to query db for git provider accounts")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetDockerRegistryAccount {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetDockerRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can read docker registry accounts")
          .into(),
      );
    }
    let res =
      find_one_by_id(&db_client().registry_accounts, &self.id)
        .await
        .context("failed to query db for docker registry accounts")?
        .context(
          "did not find docker registry account with the given id",
        )?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerRegistryAccounts {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListDockerRegistryAccountsResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only admins can read docker registry accounts")
          .into(),
      );
    }
    let mut filter = Document::new();
    if let Some(domain) = self.domain {
      filter.insert("domain", domain);
    }
    if let Some(username) = self.username {
      filter.insert("username", username);
    }
    let res = find_collect(
      &db_client().registry_accounts,
      filter,
      FindOptions::builder()
        .sort(doc! { "domain": 1, "username": 1 })
        .build(),
    )
    .await
    .context("failed to query db for docker registry accounts")?;
    Ok(res)
  }
}
