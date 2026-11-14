use std::str::FromStr;

use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::{delete_one_by_id, find_one_by_id, update_one_by_id},
  mongodb::bson::{doc, oid::ObjectId, to_document},
};
use komodo_client::{
  api::write::*,
  entities::{
    Operation, ResourceTarget, komodo_timestamp,
    provider::{
      CredentialUpdate, GitProviderAccount, ImageRegistryAccount,
      SecretProvider, SecretProviderGrant, SecretProviderKind,
      validate_provider_slug,
    },
  },
};
use mogh_error::AddStatusCodeError;
use mogh_resolver::Resolve;
use reqwest::StatusCode;
use tokio_util::sync::CancellationToken;

use crate::{
  helpers::update::{add_update, make_update},
  secret_provider::{
    adapter::get_adapter,
    authz::{
      collect_secret_provider_usages, find_secret_provider,
      require_real_admin_user, validate_and_normalize_grant_fields,
    },
    crypto::SecretProviderKeyring,
    delete_provider_credential, invalidate_provider_runtime_caches,
    load_provider_credential, rotate_all_provider_credentials,
    save_provider_credential,
  },
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

impl Resolve<WriteArgs> for CreateImageRegistryAccount {
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
  ) -> mogh_error::Result<CreateImageRegistryAccountResponse> {
    if !user.admin {
      return Err(
        anyhow!(
          "Only admins can create docker registry account accounts"
        )
        .status_code(StatusCode::FORBIDDEN),
      );
    }

    let mut account: ImageRegistryAccount = self.account.into();

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

impl Resolve<WriteArgs> for UpdateImageRegistryAccount {
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
  ) -> mogh_error::Result<UpdateImageRegistryAccountResponse> {
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

impl Resolve<WriteArgs> for DeleteImageRegistryAccount {
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
  ) -> mogh_error::Result<DeleteImageRegistryAccountResponse> {
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

impl Resolve<WriteArgs> for CreateSecretProvider {
  #[instrument(
    "CreateSecretProvider",
    skip_all,
    fields(
      operator = user.id,
      slug = self.slug,
      name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CreateSecretProviderResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let slug = self.slug.trim().to_string();
    validate_provider_slug(&slug)
      .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

    let name = self.name.trim().to_string();
    if name.is_empty() {
      return Err(
        anyhow!("Secret provider name cannot be empty")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    let kind: SecretProviderKind = (&self.config).into();
    let adapter = get_adapter(kind);
    adapter
      .validate_config(&self.config)
      .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

    if let Some(cred) = &self.credential {
      adapter
        .validate_credential(cred)
        .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;
      SecretProviderKeyring::from_core_config().map_err(|e| {
        e.context(
          "Cannot store SecretProvider credentials without a valid KOMODO_SECRET_PROVIDER_ENCRYPTION_KEY",
        )
        .status_code(StatusCode::BAD_REQUEST)
      })?;
    }

    let db = db_client();
    if db
      .secret_providers
      .find_one(
        doc! { "$or": [{ "slug": &slug }, { "name": &name }] },
      )
      .await
      .context("Failed to check duplicate secret provider")?
      .is_some()
    {
      return Err(
        anyhow!(
          "A SecretProvider with slug '{}' or name '{}' already exists",
          slug,
          name
        )
        .status_code(StatusCode::CONFLICT),
      );
    }

    let has_credentials = self.credential.is_some();
    let credential_revision = if has_credentials { 1 } else { 0 };
    let mut provider = SecretProvider {
      id: String::new(),
      slug,
      name,
      description: self.description,
      enabled: self.enabled.unwrap_or(true),
      revision: 1,
      credential_revision,
      has_credentials,
      updated_at: komodo_timestamp(),
      config: self.config,
    };

    provider.id = db
      .secret_providers
      .insert_one(&provider)
      .await
      .context("Failed to create SecretProvider in database")?
      .inserted_id
      .as_object_id()
      .context("Inserted SecretProvider id is not ObjectId")?
      .to_string();

    if let Some(cred) = &self.credential
      && let Err(e) =
        save_provider_credential(&provider.id, kind, 1, cred).await
    {
      delete_one_by_id(&db.secret_providers, &provider.id, None)
        .await
        .ok();
      return Err(e.into());
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateSecretProvider,
      user,
    );
    update.push_simple_log(
      "Create Secret Provider",
      format!(
        "Created external secret provider '{}' (slug: {}, kind: {})",
        provider.name, provider.slug, kind
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for CreateSecretProvider | {e:#}"
        )
      })
      .ok();

    Ok(provider)
  }
}

impl Resolve<WriteArgs> for UpdateSecretProvider {
  #[instrument(
    "UpdateSecretProvider",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
      expected_revision = self.expected_revision,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateSecretProviderResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let db = db_client();
    let Some(mut provider) =
      find_one_by_id(&db.secret_providers, &self.id)
        .await
        .context("Failed to query SecretProvider by id")?
    else {
      return Err(
        anyhow!("No SecretProvider found with id '{}'", self.id)
          .status_code(StatusCode::NOT_FOUND),
      );
    };

    if provider.revision != self.expected_revision {
      return Err(
        anyhow!(
          "Revision conflict on SecretProvider '{}' (expected revision {}, current revision {}); reload and retry",
          provider.slug,
          self.expected_revision,
          provider.revision
        )
        .status_code(StatusCode::CONFLICT),
      );
    }

    let existing_kind: SecretProviderKind = (&provider.config).into();
    let adapter = get_adapter(existing_kind);

    if let Some(new_name_raw) = self.name {
      let new_name = new_name_raw.trim().to_string();
      if new_name.is_empty() {
        return Err(
          anyhow!("SecretProvider name cannot be empty")
            .status_code(StatusCode::BAD_REQUEST),
        );
      }
      if new_name != provider.name {
        let oid = ObjectId::from_str(&provider.id)
          .context("Invalid ObjectId on SecretProvider")?;
        if db
          .secret_providers
          .find_one(doc! { "name": &new_name, "_id": { "$ne": oid } })
          .await
          .context("Failed to check SecretProvider name uniqueness")?
          .is_some()
        {
          return Err(
            anyhow!(
              "Another SecretProvider with name '{}' already exists",
              new_name
            )
            .status_code(StatusCode::CONFLICT),
          );
        }
        provider.name = new_name;
      }
    }

    if let Some(new_desc) = self.description {
      provider.description = new_desc;
    }

    if let Some(new_enabled) = self.enabled {
      provider.enabled = new_enabled;
    }

    if let Some(new_cfg) = self.config {
      let new_kind: SecretProviderKind = (&new_cfg).into();
      if new_kind != existing_kind {
        return Err(
          anyhow!(
            "Cannot change SecretProvider kind from '{}' to '{}'",
            existing_kind,
            new_kind
          )
          .status_code(StatusCode::BAD_REQUEST),
        );
      }
      adapter
        .validate_config(&new_cfg)
        .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

      if provider.has_credentials
        && provider.config.target_or_identity_changed(&new_cfg)
        && matches!(self.credential_update, CredentialUpdate::Keep)
      {
        return Err(
          anyhow!(
            "Changing endpoint, client_id, organization_slug, CA certificate, or private-IP policy on SecretProvider '{}' requires explicitly replacing ('Replace') or clearing ('Clear') stored credentials; refusing to silently keep existing credentials for a modified target",
            provider.slug
          )
          .status_code(StatusCode::BAD_REQUEST),
        );
      }
      provider.config = new_cfg;
    }

    match &self.credential_update {
      CredentialUpdate::Keep => {}
      CredentialUpdate::Replace(cred) => {
        adapter
          .validate_credential(cred)
          .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;
        SecretProviderKeyring::from_core_config().map_err(|e| {
          e.context(
            "Cannot store SecretProvider credentials without a valid KOMODO_SECRET_PROVIDER_ENCRYPTION_KEY",
          )
          .status_code(StatusCode::BAD_REQUEST)
        })?;
        let next_cred_rev = provider.credential_revision + 1;
        save_provider_credential(
          &provider.id,
          existing_kind,
          next_cred_rev,
          cred,
        )
        .await?;
        provider.has_credentials = true;
        provider.credential_revision = next_cred_rev;
      }
      CredentialUpdate::Clear => {
        delete_provider_credential(&provider.id).await?;
        provider.has_credentials = false;
        provider.credential_revision += 1;
      }
    }

    provider.revision += 1;
    provider.updated_at = komodo_timestamp();

    let oid = ObjectId::from_str(&provider.id)
      .context("Invalid ObjectId on SecretProvider")?;
    let mut doc_body = to_document(&provider)
      .context("Failed to serialize SecretProvider update")?;
    doc_body.remove("_id");

    let update_res = db
      .secret_providers
      .update_one(
        doc! { "_id": oid, "revision": self.expected_revision },
        doc! { "$set": doc_body },
      )
      .await
      .context("Failed to update SecretProvider in database")?;
    if update_res.matched_count == 0 {
      return Err(
        anyhow!(
          "Concurrent modification detected on SecretProvider '{}'; reload and retry",
          provider.slug
        )
        .status_code(StatusCode::CONFLICT),
      );
    }

    invalidate_provider_runtime_caches(&provider.id);

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateSecretProvider,
      user,
    );
    update.push_simple_log(
      "Update Secret Provider",
      format!(
        "Updated external secret provider '{}' (slug: {}, revision: {}, enabled: {})",
        provider.name,
        provider.slug,
        provider.revision,
        provider.enabled
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for UpdateSecretProvider | {e:#}"
        )
      })
      .ok();

    Ok(provider)
  }
}

impl Resolve<WriteArgs> for DeleteSecretProvider {
  #[instrument(
    "DeleteSecretProvider",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<DeleteSecretProviderResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let provider = find_secret_provider(&self.id)
      .await
      .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;

    let usages = collect_secret_provider_usages(&provider).await?;
    if !usages.is_empty() {
      let summary = usages
        .iter()
        .map(|u| format!("{}:{}", u.resource_type, u.resource_name))
        .collect::<Vec<_>>()
        .join(", ");
      return Err(
        anyhow!(
          "Cannot delete SecretProvider '{}' ({}) while it is referenced by {} resource(s): {}",
          provider.name,
          provider.slug,
          usages.len(),
          summary
        )
        .status_code(StatusCode::CONFLICT),
      );
    }

    let db = db_client();
    db.secret_provider_grants
      .delete_many(doc! { "provider_id": &provider.id })
      .await
      .context("Failed to delete grants for SecretProvider")?;
    delete_provider_credential(&provider.id).await?;
    delete_one_by_id(&db.secret_providers, &provider.id, None)
      .await
      .context("Failed to delete SecretProvider from database")?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::DeleteSecretProvider,
      user,
    );
    update.push_simple_log(
      "Delete Secret Provider",
      format!(
        "Deleted external secret provider '{}' (slug: {})",
        provider.name, provider.slug
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for DeleteSecretProvider | {e:#}"
        )
      })
      .ok();

    Ok(provider)
  }
}

impl Resolve<WriteArgs> for TestSecretProviderConnection {
  #[instrument(
    "TestSecretProviderConnection",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<
    komodo_client::entities::provider::TestSecretProviderResponse,
  > {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let (test_provider, cred) = match &self.id {
      Some(id) if !id.trim().is_empty() => {
        let saved = find_secret_provider(id)
          .await
          .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;
        let effective_config =
          self.config.unwrap_or_else(|| saved.config.clone());
        let kind: SecretProviderKind = (&effective_config).into();
        let adapter = get_adapter(kind);
        adapter
          .validate_config(&effective_config)
          .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

        let cred = match self.credential_update {
          CredentialUpdate::Keep => {
            if saved
              .config
              .target_or_identity_changed(&effective_config)
            {
              return Err(
                anyhow!(
                  "Cannot test modified endpoint, identity, CA, or private-IP settings using stored credentials ('Keep'); supply credentials explicitly via 'Replace' to prevent sending stored credentials to an unverified target"
                )
                .status_code(StatusCode::BAD_REQUEST),
              );
            }
            load_provider_credential(&saved).await?
          }
          CredentialUpdate::Replace(c) => {
            adapter
              .validate_credential(&c)
              .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;
            c
          }
          CredentialUpdate::Clear => {
            return Err(
              anyhow!(
                "Cannot test SecretProvider connection with cleared credentials"
              )
              .status_code(StatusCode::BAD_REQUEST),
            );
          }
        };

        let ephemeral = SecretProvider {
          // Empty ID ensures test calls never overwrite or read cached tokens for the saved provider.
          id: String::new(),
          slug: saved.slug,
          name: saved.name,
          description: saved.description,
          enabled: true,
          revision: saved.revision,
          credential_revision: saved.credential_revision,
          has_credentials: true,
          updated_at: saved.updated_at,
          config: effective_config,
        };
        (ephemeral, cred)
      }
      _ => {
        let cfg = self.config.ok_or_else(|| {
          anyhow!(
            "TestSecretProviderConnection requires 'config' when testing an unsaved provider"
          )
          .status_code(StatusCode::BAD_REQUEST)
        })?;
        let kind: SecretProviderKind = (&cfg).into();
        let adapter = get_adapter(kind);
        adapter
          .validate_config(&cfg)
          .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;
        let cred = match self.credential_update {
          CredentialUpdate::Replace(c) => {
            adapter
              .validate_credential(&c)
              .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;
            c
          }
          _ => {
            return Err(
              anyhow!(
                "TestSecretProviderConnection requires 'credential_update = Replace(...)' when testing an unsaved provider"
              )
              .status_code(StatusCode::BAD_REQUEST),
            );
          }
        };
        let ephemeral = SecretProvider {
          id: String::new(),
          slug: String::from("draft-test"),
          name: String::from("Draft Test"),
          description: String::new(),
          enabled: true,
          revision: 1,
          credential_revision: 1,
          has_credentials: true,
          updated_at: 0,
          config: cfg,
        };
        (ephemeral, cred)
      }
    };

    let adapter = get_adapter((&test_provider.config).into());
    let cancel = CancellationToken::new();
    let res = adapter
      .test_connection(
        &test_provider,
        &cred,
        self.selector.as_ref(),
        self.sample_key.as_deref(),
        &cancel,
      )
      .await?;
    Ok(res)
  }
}

impl Resolve<WriteArgs> for CreateSecretProviderGrant {
  #[instrument(
    "CreateSecretProviderGrant",
    skip_all,
    fields(
      operator = user.id,
      provider_id = self.provider_id,
      name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CreateSecretProviderGrantResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let provider = find_secret_provider(&self.provider_id)
      .await
      .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;

    let normalized_scope = validate_and_normalize_grant_fields(
      &provider,
      &self.subject,
      &self.allowed_resources,
      &self.selector_scope,
      &self.allowed_usages,
    )
    .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

    let mut grant = SecretProviderGrant {
      id: String::new(),
      provider_id: provider.id.clone(),
      name: self.name.trim().to_string(),
      description: self.description,
      enabled: self.enabled.unwrap_or(true),
      revision: 1,
      subject: self.subject,
      allowed_resources: self.allowed_resources,
      selector_scope: normalized_scope,
      allowed_usages: self.allowed_usages,
      updated_at: komodo_timestamp(),
    };

    grant.id = db_client()
      .secret_provider_grants
      .insert_one(&grant)
      .await
      .context("Failed to insert SecretProviderGrant in database")?
      .inserted_id
      .as_object_id()
      .context("Inserted SecretProviderGrant id is not ObjectId")?
      .to_string();

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::CreateSecretProviderGrant,
      user,
    );
    update.push_simple_log(
      "Create Secret Provider Grant",
      format!(
        "Created grant '{}' on secret provider '{}' ({})",
        grant.name, provider.name, provider.slug
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for CreateSecretProviderGrant | {e:#}"
        )
      })
      .ok();

    Ok(grant)
  }
}

impl Resolve<WriteArgs> for UpdateSecretProviderGrant {
  #[instrument(
    "UpdateSecretProviderGrant",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
      expected_revision = self.expected_revision,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateSecretProviderGrantResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let db = db_client();
    let Some(mut grant) =
      find_one_by_id(&db.secret_provider_grants, &self.id)
        .await
        .context("Failed to query SecretProviderGrant by id")?
    else {
      return Err(
        anyhow!("No SecretProviderGrant found with id '{}'", self.id)
          .status_code(StatusCode::NOT_FOUND),
      );
    };

    if grant.revision != self.expected_revision {
      return Err(
        anyhow!(
          "Revision conflict on SecretProviderGrant '{}' (expected revision {}, current revision {}); reload and retry",
          grant.id,
          self.expected_revision,
          grant.revision
        )
        .status_code(StatusCode::CONFLICT),
      );
    }

    let provider = find_secret_provider(&grant.provider_id)
      .await
      .map_err(|e| e.status_code(StatusCode::NOT_FOUND))?;

    if let Some(name) = self.name {
      grant.name = name.trim().to_string();
    }
    if let Some(description) = self.description {
      grant.description = description;
    }
    if let Some(enabled) = self.enabled {
      grant.enabled = enabled;
    }
    if let Some(subject) = self.subject {
      grant.subject = subject;
    }
    if let Some(allowed_resources) = self.allowed_resources {
      grant.allowed_resources = allowed_resources;
    }
    if let Some(selector_scope) = self.selector_scope {
      grant.selector_scope = selector_scope;
    }
    if let Some(allowed_usages) = self.allowed_usages {
      grant.allowed_usages = allowed_usages;
    }

    grant.selector_scope = validate_and_normalize_grant_fields(
      &provider,
      &grant.subject,
      &grant.allowed_resources,
      &grant.selector_scope,
      &grant.allowed_usages,
    )
    .map_err(|e| e.status_code(StatusCode::BAD_REQUEST))?;

    grant.revision += 1;
    grant.updated_at = komodo_timestamp();

    let oid = ObjectId::from_str(&grant.id)
      .context("Invalid ObjectId on SecretProviderGrant")?;
    let mut doc_body = to_document(&grant)
      .context("Failed to serialize SecretProviderGrant update")?;
    doc_body.remove("_id");

    let update_res = db
      .secret_provider_grants
      .update_one(
        doc! { "_id": oid, "revision": self.expected_revision },
        doc! { "$set": doc_body },
      )
      .await
      .context("Failed to update SecretProviderGrant in database")?;
    if update_res.matched_count == 0 {
      return Err(
        anyhow!(
          "Concurrent modification detected on SecretProviderGrant '{}'; reload and retry",
          grant.id
        )
        .status_code(StatusCode::CONFLICT),
      );
    }

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::UpdateSecretProviderGrant,
      user,
    );
    update.push_simple_log(
      "Update Secret Provider Grant",
      format!(
        "Updated grant '{}' on secret provider '{}' ({}, revision: {}, enabled: {})",
        grant.name,
        provider.name,
        provider.slug,
        grant.revision,
        grant.enabled
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for UpdateSecretProviderGrant | {e:#}"
        )
      })
      .ok();

    Ok(grant)
  }
}

impl Resolve<WriteArgs> for DeleteSecretProviderGrant {
  #[instrument(
    "DeleteSecretProviderGrant",
    skip_all,
    fields(
      operator = user.id,
      id = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<DeleteSecretProviderGrantResponse> {
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let db = db_client();
    let Some(grant) =
      find_one_by_id(&db.secret_provider_grants, &self.id)
        .await
        .context("Failed to query SecretProviderGrant by id")?
    else {
      return Err(
        anyhow!("No SecretProviderGrant found with id '{}'", self.id)
          .status_code(StatusCode::NOT_FOUND),
      );
    };

    delete_one_by_id(&db.secret_provider_grants, &self.id, None)
      .await
      .context(
        "Failed to delete SecretProviderGrant from database",
      )?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::DeleteSecretProviderGrant,
      user,
    );
    update.push_simple_log(
      "Delete Secret Provider Grant",
      format!(
        "Deleted grant '{}' (provider_id: {})",
        grant.name, grant.provider_id
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for DeleteSecretProviderGrant | {e:#}"
        )
      })
      .ok();

    Ok(grant)
  }
}

impl Resolve<WriteArgs> for RotateSecretProviderEncryptionKeys {
  #[instrument(
    "RotateSecretProviderEncryptionKeys",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<
    komodo_client::entities::provider::RotateSecretProviderEncryptionKeysResponse,
  >{
    require_real_admin_user(user)
      .map_err(|e| e.status_code(StatusCode::FORBIDDEN))?;

    let res = rotate_all_provider_credentials().await?;

    let mut update = make_update(
      ResourceTarget::system(),
      Operation::RotateSecretProviderKeys,
      user,
    );
    update.push_simple_log(
      "Rotate Secret Provider Encryption Keys",
      format!(
        "Rotated {} stored SecretProvider credential envelope(s) to active key_id '{}'",
        res.rotated_count, res.active_key_id
      ),
    );
    update.finalize();
    add_update(update)
      .await
      .inspect_err(|e| {
        error!(
          "Failed to add update for RotateSecretProviderEncryptionKeys | {e:#}"
        )
      })
      .ok();

    Ok(res)
  }
}
