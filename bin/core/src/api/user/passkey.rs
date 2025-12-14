use anyhow::Context as _;
use database::{
  bson::{doc, to_bson},
  mungos::by_id::update_one_by_id,
};
use komodo_client::{
  api::user::{
    BeginPasskeyEnrollment, BeginPasskeyEnrollmentResponse,
    ConfirmPasskeyEnrollment, ConfirmPasskeyEnrollmentResponse,
    UnenrollPasskey, UnenrollPasskeyResponse,
  },
  entities::komodo_timestamp,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::PasskeyRegistration;

use crate::state::{db_client, webauthn};

use super::UserArgs;

#[derive(Serialize, Deserialize)]
struct SessionPasskeyEnrollment {
  user_id: String,
  state: PasskeyRegistration,
}
impl SessionPasskeyEnrollment {
  const KEY: &str = "passkey-enrollment";
}

impl Resolve<UserArgs> for BeginPasskeyEnrollment {
  #[instrument(
    "BeginPasskeyEnrollment",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, session }: &UserArgs,
  ) -> serror::Result<BeginPasskeyEnrollmentResponse> {
    super::check_locked(&user.username)?;

    let session = session.as_ref().context(
      "Method called in invalid context. This should not happen",
    )?;

    let webauthn = webauthn().context(
      "No webauthn provider available, invalid KOMODO_HOST config",
    )?;

    // Get two parts from this, the first is returned to the client.
    // The second must stay server side and is used in confirmation flow.
    let (challenge, state) = webauthn.start_passkey_registration(
      Uuid::new_v4(),
      &user.username,
      &user.username,
      None,
    )?;

    session
      .insert(
        SessionPasskeyEnrollment::KEY,
        SessionPasskeyEnrollment {
          user_id: user.id.clone(),
          state
        },
      )
      .await
      .context(
        "Failed to store passkey enrollment state in server side client session",
      )?;

    Ok(challenge)
  }
}

//

impl Resolve<UserArgs> for ConfirmPasskeyEnrollment {
  #[instrument(
    "ConfirmPasskeyEnrollment",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, session }: &UserArgs,
  ) -> serror::Result<ConfirmPasskeyEnrollmentResponse> {
    let session = session.as_ref().context(
      "Method called in invalid context. This should not happen",
    )?;

    let webauthn = webauthn().context(
      "No webauthn provider available, invalid KOMODO_HOST config",
    )?;

    let SessionPasskeyEnrollment { user_id, state } = session
      .remove(SessionPasskeyEnrollment::KEY)
      .await
      .context("Passkey enrollment was not initiated correctly")?
      .context(
        "Passkey enrollment was not initiated correctly or timed out",
      )?;

    let passkey = webauthn
      .finish_passkey_registration(&self.credential, &state)
      .context("Failed to finish passkey registration")?;

    let passkey = to_bson(&passkey)
      .context("Failed to serialize passkey to BSON")?;

    let update = doc! {
      "$set": {
        "passkey.passkey": passkey,
        "passkey.created_at": komodo_timestamp()
      }
    };

    update_one_by_id(&db_client().users, &user_id, update, None)
      .await
      .context("Failed to update user passkey options on database")?;

    Ok(ConfirmPasskeyEnrollmentResponse {})
  }
}

//

impl Resolve<UserArgs> for UnenrollPasskey {
  #[instrument(
    "UnenrollPasskey",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<UnenrollPasskeyResponse> {
    let update = doc! {
      "$set": {
        "passkey.passkey": null,
        "passkey.created_at": 0
      }
    };
    update_one_by_id(&db_client().users, &user.id, update, None)
      .await
      .context("Failed to update user passkey options on database")?;
    Ok(UnenrollPasskeyResponse {})
  }
}
