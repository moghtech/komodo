use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::{Document, doc};
use komodo_client::{
  api::write::{
    CreateOnboardingKey, CreateOnboardingKeyResponse,
    DeleteOnboardingKey, DeleteOnboardingKeyResponse,
    UpdateOnboardingKey, UpdateOnboardingKeyResponse,
  },
  entities::{
    komodo_timestamp, onboarding_key::OnboardingKey, random_string,
  },
};
use mogh_error::{AddStatusCode, AddStatusCodeError};
use mogh_pki::EncodedKeyPair;
use mogh_resolver::Resolve;
use reqwest::StatusCode;

use crate::{api::write::WriteArgs, state::db_client};

//

impl Resolve<WriteArgs> for CreateOnboardingKey {
  #[instrument(
    "CreateOnboardingKey",
    skip_all,
    fields(
      operator = admin.id,
      name = self.name,
      expires = self.expires,
      tags = format!("{:?}", self.tags),
      copy_server = self.copy_server,
      create_builder = self.create_builder,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> mogh_error::Result<CreateOnboardingKeyResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This call is admin only")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    let private_key = if let Some(private_key) = self.private_key {
      private_key
    } else {
      format!("O_{}_O", random_string(28))
    };
    let public_key = EncodedKeyPair::from_private_key(
      mogh_pki::PkiKind::Mutual,
      &private_key,
    )?
    .public
    .into_inner();
    let onboarding_key = OnboardingKey {
      public_key,
      name: self.name,
      enabled: true,
      onboarded: Default::default(),
      created_at: komodo_timestamp(),
      expires: self.expires,
      tags: self.tags,
      privileged: self.privileged,
      copy_server: self.copy_server,
      create_builder: self.create_builder,
    };
    let db = db_client();
    // Create the key
    db.onboarding_keys
      .insert_one(&onboarding_key)
      .await
      .context(
        "Failed to create Server onboarding key on database",
      )?;
    let created = db
      .onboarding_keys
      .find_one(doc! { "public_key": &onboarding_key.public_key })
      .await
      .context("Failed to query database for Server onboarding keys")?
      .context(
        "No Server onboarding key found on database after create",
      )?;
    Ok(CreateOnboardingKeyResponse {
      private_key,
      created,
    })
  }
}

//

impl Resolve<WriteArgs> for UpdateOnboardingKey {
  #[instrument(
    "UpdateOnboardingKey",
    skip_all,
    fields(
      operator = admin.id,
      public_key = self.public_key,
      update = format!("{:?}", self),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> mogh_error::Result<UpdateOnboardingKeyResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This call is admin only")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let query = doc! { "public_key": &self.public_key };

    // No changes
    if self.is_none() {
      return db_client()
        .onboarding_keys
        .find_one(query)
        .await
        .context("Failed to query database for onboarding key")?
        .context("No matching onboarding key found")
        .status_code(StatusCode::NOT_FOUND);
    }

    let mut update = Document::new();

    if let Some(enabled) = self.enabled {
      update.insert("enabled", enabled);
    }

    if let Some(name) = self.name {
      update.insert("name", name);
    }

    if let Some(expires) = self.expires {
      update.insert("expires", expires);
    }

    if let Some(tags) = self.tags {
      update.insert("tags", tags);
    }

    if let Some(privileged) = self.privileged {
      update.insert("privileged", privileged);
    }

    if let Some(copy_server) = self.copy_server {
      update.insert("copy_server", copy_server);
    }

    if let Some(create_builder) = self.create_builder {
      update.insert("create_builder", create_builder);
    }

    db_client()
      .onboarding_keys
      .update_one(query.clone(), doc! { "$set": update })
      .await
      .context("Failed to update onboarding key on database")?;

    db_client()
      .onboarding_keys
      .find_one(query)
      .await
      .context("Failed to query database for onboarding key")?
      .context("No matching onboarding key found")
      .status_code(StatusCode::NOT_FOUND)
  }
}

//

impl Resolve<WriteArgs> for DeleteOnboardingKey {
  #[instrument(
    "DeleteOnboardingKey",
    skip_all,
    fields(
      operator = admin.id,
      public_key = self.public_key,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> mogh_error::Result<DeleteOnboardingKeyResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This call is admin only")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    let db = db_client();
    let query = doc! { "public_key": &self.public_key };
    let creation_key = db
      .onboarding_keys
      .find_one(query.clone())
      .await
      .context("Failed to query database for Server onboarding keys")?
      .context("Server onboarding key matching provided public key not found")
      .status_code(StatusCode::NOT_FOUND)?;
    db.onboarding_keys.delete_one(query).await.context(
      "Failed to delete Server onboarding key from database",
    )?;
    Ok(creation_key)
  }
}
