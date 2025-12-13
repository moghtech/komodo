use anyhow::{Context as _, anyhow};
use data_encoding::BASE32_NOPAD;
use database::{
  bson::doc, hash_password, mungos::by_id::update_one_by_id,
};
use komodo_client::{
  api::user::{
    BeginTotpEnrollment, BeginTotpEnrollmentResponse,
    ConfirmTotpEnrollment, ConfirmTotpEnrollmentResponse,
    UnenrollTotp, UnenrollTotpResponse,
  },
  entities::{komodo_timestamp, random_bytes, random_string},
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use serror::AddStatusCodeError as _;

use crate::{auth::totp::make_totp, state::db_client};

use super::UserArgs;

const TOTP_ENROLLMENT_SECRET_LENGTH: usize = 20;

#[derive(Serialize, Deserialize)]
struct SessionTotpEnrollment {
  secret: Vec<u8>,
}
impl SessionTotpEnrollment {
  const KEY: &str = "totp-enrollment";
}

impl Resolve<UserArgs> for BeginTotpEnrollment {
  #[instrument(
    "BeginTotpEnrollment",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, session }: &UserArgs,
  ) -> serror::Result<BeginTotpEnrollmentResponse> {
    super::check_locked(&user.username)?;

    let session = session.as_ref().context(
      "Method called in invalid context. This should not happen",
    )?;

    let secret = random_bytes(TOTP_ENROLLMENT_SECRET_LENGTH);
    let totp = make_totp(secret.clone(), user.id.clone())?;
    let png = totp
      .get_qr_base64()
      .map_err(|e| anyhow::Error::msg(e))
      .context("Failed to generate QR code png")?;
    session
      .insert(
        SessionTotpEnrollment::KEY,
        SessionTotpEnrollment { secret },
      )
      .await?;

    Ok(BeginTotpEnrollmentResponse {
      uri: totp.get_url(),
      png,
    })
  }
}

impl Resolve<UserArgs> for ConfirmTotpEnrollment {
  #[instrument(
    "ConfirmTotpEnrollment",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, session }: &UserArgs,
  ) -> serror::Result<ConfirmTotpEnrollmentResponse> {
    let session = session.as_ref().context(
      "Method called in invalid context. This should not happen",
    )?;

    let SessionTotpEnrollment { secret } = session
      .remove(SessionTotpEnrollment::KEY)
      .await
      .context("Totp enrollment was not initiated correctly")?
      .context(
        "Totp enrollment was not initiated correctly or timed out",
      )?;

    let encoded_secret = BASE32_NOPAD.encode(&secret);

    let totp = make_totp(secret, None)?;

    let valid = totp
      .check_current(&self.code)
      .context("Failed to check code validity")?;

    if !valid {
      return Err(anyhow!(
        "The provided code was not valid. Please try BeginTotpEnrollment flow again."
      ).status_code(StatusCode::BAD_REQUEST));
    }

    let recovery_codes =
      (0..10).map(|_| random_string(20)).collect::<Vec<_>>();
    let hashed_recovery_codes = recovery_codes
      .iter()
      .map(|code| hash_password(code))
      .collect::<anyhow::Result<Vec<_>>>()
      .context("Failed to generate valid recovery codes")?;

    update_one_by_id(
      &db_client().users,
      &user.id,
      doc! {
        "$set": {
          "totp.secret": encoded_secret,
          "totp.confirmed_at": komodo_timestamp(),
          "totp.recovery_codes": hashed_recovery_codes,
        }
      },
      None,
    )
    .await
    .context("Failed to update user totp fields on database")?;

    Ok(ConfirmTotpEnrollmentResponse { recovery_codes })
  }
}

impl Resolve<UserArgs> for UnenrollTotp {
  #[instrument(
    "UnenrollTotp",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<UnenrollTotpResponse> {
    update_one_by_id(
      &db_client().users,
      &user.id,
      doc! {
        "$set": {
          "totp.secret": "",
          "totp.confirmed_at": 0,
          "totp.recovery_codes": [],
        }
      },
      None,
    )
    .await
    .context("Failed to clear user totp fields on database")?;
    Ok(UnenrollTotpResponse {})
  }
}
