use anyhow::{Context, anyhow};
use database::mongo_indexed::{Document, doc};
use database::mungos::{
  by_id::find_one_by_id, find::find_collect,
  mongodb::options::FindOptions,
};
use komodo_client::{
  api::read::*,
  entities::{
    ResourceTarget, build::Build, deployment::Deployment,
    permission::PermissionLevel, provider::*, stack::Stack,
  },
};
use mogh_error::AddStatusCodeError;
use mogh_resolver::Resolve;
use reqwest::StatusCode;
use tokio_util::sync::CancellationToken;

use crate::{
  permission::get_check_permissions,
  secret_provider::{
    adapter::get_adapter,
    authz::{
      authorize_secret_access, collect_secret_provider_usages,
      find_secret_provider, list_available_secret_providers_for_user,
      require_real_admin_user, resolve_selector_for_provider,
    },
    load_provider_credential,
    resolver::preview_resource_secrets,
  },
  state::db_client,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetGitProviderAccount {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetGitProviderAccountResponse> {
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
  ) -> mogh_error::Result<ListGitProviderAccountsResponse> {
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

impl Resolve<ReadArgs> for GetImageRegistryAccount {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetImageRegistryAccountResponse> {
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

impl Resolve<ReadArgs> for ListImageRegistryAccounts {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListImageRegistryAccountsResponse> {
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

impl Resolve<ReadArgs> for GetSecretProvider {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetSecretProviderResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;
    let provider = find_secret_provider(&self.id_or_slug)
      .await
      .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;
    Ok(provider)
  }
}

impl Resolve<ReadArgs> for ListSecretProviders {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListSecretProvidersResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;
    let mut providers = find_collect(
      &db_client().secret_providers,
      Document::new(),
      FindOptions::builder().sort(doc! { "slug": 1 }).build(),
    )
    .await
    .context("failed to query db for secret providers")?;
    if let Some(kind) = self.kind {
      providers
        .retain(|p| SecretProviderKind::from(&p.config) == kind);
    }
    Ok(providers)
  }
}

impl Resolve<ReadArgs> for ListAvailableSecretProviders {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListAvailableSecretProvidersResponse> {
    let summaries = list_available_secret_providers_for_user(
      user,
      self.target.as_ref(),
    )
    .await?;
    Ok(summaries)
  }
}

impl Resolve<ReadArgs> for ListSecretProviderGrants {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListSecretProviderGrantsResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;
    let mut filter = Document::new();
    if let Some(provider_id) = self.provider_id
      && !provider_id.trim().is_empty()
    {
      filter.insert("provider_id", provider_id.trim());
    }
    let grants = find_collect(
      &db_client().secret_provider_grants,
      filter,
      FindOptions::builder()
        .sort(doc! { "provider_id": 1, "name": 1 })
        .build(),
    )
    .await
    .context("failed to query db for secret provider grants")?;
    Ok(grants)
  }
}

impl Resolve<ReadArgs> for ListSecretProviderUsages {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListSecretProviderUsagesResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;
    let provider = find_secret_provider(&self.provider_id)
      .await
      .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;
    let usages = collect_secret_provider_usages(&provider).await?;
    Ok(usages)
  }
}

impl Resolve<ReadArgs> for ListExternalSecretKeys {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListExternalSecretKeysResponse> {
    let (canonical_target, resource_sources) = match &self.target {
      ResourceTarget::Stack(id_or_name) => {
        let stack = get_check_permissions::<Stack>(
          id_or_name,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
        (ResourceTarget::Stack(stack.id), stack.config.secret_sources)
      }
      ResourceTarget::Deployment(id_or_name) => {
        let dep = get_check_permissions::<Deployment>(
          id_or_name,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
        (
          ResourceTarget::Deployment(dep.id),
          dep.config.secret_sources,
        )
      }
      ResourceTarget::Build(id_or_name) => {
        let build = get_check_permissions::<Build>(
          id_or_name,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
        (ResourceTarget::Build(build.id), build.config.secret_sources)
      }
      _ => {
        return Err(
          anyhow!(
            "External secret metadata listing is only supported for Stack, Deployment, and Build targets"
          )
          .status_code(StatusCode::BAD_REQUEST),
        );
      }
    };

    let provider = find_secret_provider(&self.provider)
      .await
      .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;
    let selector = resolve_selector_for_provider(
      &provider,
      &resource_sources,
      &self.provider,
      self.selector.as_ref(),
    )
    .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

    let provider = authorize_secret_access(
      user,
      &provider.id,
      &canonical_target,
      &selector,
      SecretUsage::MetadataList,
      Some(PermissionLevel::Read),
    )
    .await
    .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let adapter = get_adapter((&provider.config).into());
    if !adapter.capabilities().supports_metadata_listing {
      return Err(
        anyhow!(
          "Secret provider '{}' does not support metadata listing",
          provider.slug
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    }

    let cred = load_provider_credential(&provider).await?;
    let cancel = CancellationToken::new();
    let items = adapter
      .list_metadata(&provider, &cred, &selector, &cancel)
      .await?;
    Ok(items)
  }
}

impl Resolve<ReadArgs> for PreviewResourceSecrets {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<PreviewResourceSecretsResponse> {
    let preview =
      preview_resource_secrets(&self.target, user).await?;
    Ok(preview)
  }
}
