use std::str::FromStr;

use anyhow::{Context, anyhow};
use komodo_client::entities::{
  action::Action,
  alert::Alert,
  alerter::Alerter,
  api_key::ApiKey,
  build::Build,
  builder::Builder,
  config::DatabaseConfig,
  deployment::Deployment,
  onboarding_key::OnboardingKey,
  permission::Permission,
  procedure::Procedure,
  provider::{DockerRegistryAccount, GitProviderAccount},
  repo::Repo,
  server::Server,
  stack::Stack,
  stats::SystemStatsRecord,
  swarm::Swarm,
  sync::ResourceSync,
  tag::Tag,
  update::Update,
  user::{User, UserConfig},
  user_group::UserGroup,
  variable::Variable,
};
use mongo_indexed::{create_index, create_unique_index};
use mungos::{
  init::MongoBuilder,
  mongodb::{
    Collection, Database,
    bson::{doc, oid::ObjectId},
  },
};

pub use mongo_indexed;
pub use mungos;
pub use mungos::mongodb::bson;

pub mod utils;

#[derive(Debug)]
pub struct Client {
  pub users: Collection<User>,
  pub user_groups: Collection<UserGroup>,
  pub permissions: Collection<Permission>,
  pub api_keys: Collection<ApiKey>,
  pub onboarding_keys: Collection<OnboardingKey>,
  pub tags: Collection<Tag>,
  pub variables: Collection<Variable>,
  pub git_accounts: Collection<GitProviderAccount>,
  pub registry_accounts: Collection<DockerRegistryAccount>,
  pub updates: Collection<Update>,
  pub alerts: Collection<Alert>,
  pub stats: Collection<SystemStatsRecord>,
  // RESOURCES
  pub swarms: Collection<Swarm>,
  pub servers: Collection<Server>,
  pub deployments: Collection<Deployment>,
  pub builds: Collection<Build>,
  pub builders: Collection<Builder>,
  pub repos: Collection<Repo>,
  pub procedures: Collection<Procedure>,
  pub actions: Collection<Action>,
  pub alerters: Collection<Alerter>,
  pub resource_syncs: Collection<ResourceSync>,
  pub stacks: Collection<Stack>,
  //
  pub db: Database,
}

impl Client {
  pub async fn new(
    config: &DatabaseConfig,
  ) -> anyhow::Result<Client> {
    let db = init(config).await?;
    Self::from_database(db).await
  }

  pub async fn from_database(db: Database) -> anyhow::Result<Client> {
    let client = Client {
      users: mongo_indexed::collection(&db, true).await?,
      user_groups: mongo_indexed::collection(&db, true).await?,
      permissions: mongo_indexed::collection(&db, true).await?,
      api_keys: mongo_indexed::collection(&db, true).await?,
      onboarding_keys: mongo_indexed::collection(&db, true).await?,
      tags: mongo_indexed::collection(&db, true).await?,
      variables: mongo_indexed::collection(&db, true).await?,
      git_accounts: mongo_indexed::collection(&db, true).await?,
      registry_accounts: mongo_indexed::collection(&db, true).await?,
      updates: mongo_indexed::collection(&db, true).await?,
      alerts: mongo_indexed::collection(&db, true).await?,
      stats: mongo_indexed::collection(&db, true).await?,
      // RESOURCES
      swarms: resource_collection(&db, "Swarm").await?,
      servers: resource_collection(&db, "Server").await?,
      deployments: resource_collection(&db, "Deployment").await?,
      builds: resource_collection(&db, "Build").await?,
      builders: resource_collection(&db, "Builder").await?,
      repos: resource_collection(&db, "Repo").await?,
      alerters: resource_collection(&db, "Alerter").await?,
      procedures: resource_collection(&db, "Procedure").await?,
      actions: resource_collection(&db, "Action").await?,
      resource_syncs: resource_collection(&db, "ResourceSync")
        .await?,
      stacks: resource_collection(&db, "Stack").await?,
      //
      db,
    };
    Ok(client)
  }

  /// Updates a user's password using a DB call.
  pub async fn set_user_password(
    &self,
    user: &User,
    password: &str,
  ) -> anyhow::Result<()> {
    let UserConfig::Local { .. } = user.config else {
      return Err(anyhow!(
        "User is not a 'Local' (username / password) user"
      ));
    };
    if password.is_empty() {
      return Err(anyhow!("Password cannot be empty."));
    }
    let id = ObjectId::from_str(&user.id)
      .context("User id not valid ObjectId.")?;
    let hashed_password =
      hash_password(password).context("Failed to hash password")?;
    self
      .users
      .update_one(
        doc! { "_id": id },
        doc! { "$set": {
          "config.data.password": hashed_password
        } },
      )
      .await
      .context("Failed to update user password on database.")?;
    Ok(())
  }

  /// Clears users configured passkey / totp 2FA methods.
  /// Useful if user gets locked out after losing access to their second factor.
  pub async fn clear_user_2fa_methods(
    &self,
    // Username or id
    id_or_username: &str,
  ) -> anyhow::Result<()> {
    let query = match ObjectId::from_str(id_or_username) {
      Ok(id) => doc! { "_id": id },
      Err(_) => doc! { "username": id_or_username },
    };
    self
      .users
      .update_one(
        query,
        doc! {
          "$unset": {
            "passkey": "",
            "totp": "",
          }
        },
      )
      .await
      .context("Failed to clear user 2FA methods on database.")?;
    Ok(())
  }
}

/// Initializes unindexed database handle.
pub async fn init(
  DatabaseConfig {
    uri,
    address,
    username,
    password,
    app_name,
    db_name,
  }: &DatabaseConfig,
) -> anyhow::Result<Database> {
  let mut client = MongoBuilder::default().app_name(app_name);

  match (
    !uri.is_empty(),
    !address.is_empty(),
    !username.is_empty(),
    !password.is_empty(),
  ) {
    (true, _, _, _) => {
      client = client.uri(uri);
    }
    (_, true, true, true) => {
      client = client
        .address(address)
        .username(username)
        .password(password);
    }
    (_, true, _, _) => {
      client = client.address(address);
    }
    _ => {
      return Err(anyhow!(
        "'config.database' not configured correctly. must pass either 'config.database.uri', or 'config.database.address' + 'config.database.username' + 'config.database.password'"
      ));
    }
  }

  let client = client
    .build()
    .await
    .context("Failed to initialize database connection.")?;

  Ok(client.database(db_name))
}

async fn resource_collection<T: Send + Sync>(
  db: &Database,
  collection_name: &str,
) -> anyhow::Result<Collection<T>> {
  let coll = db.collection::<T>(collection_name);

  create_unique_index(&coll, "name").await?;

  create_index(&coll, "tags").await?;

  Ok(coll)
}

const BCRYPT_COST: u32 = 10;
pub fn hash_password<P>(password: P) -> anyhow::Result<String>
where
  P: AsRef<[u8]>,
{
  bcrypt::hash(password, BCRYPT_COST)
    .context("failed to hash password")
}
