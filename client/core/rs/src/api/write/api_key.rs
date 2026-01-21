use mogh_auth_client::api::manage::CreateApiKeyResponse;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, NoData};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateApiKeyForServiceUser",
  description = "**Admin only**. Create an api key for a service user.",
  request_body(content = CreateApiKeyForServiceUser),
  responses(
    (status = 200, description = "The new alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn create_api_key_for_service_user() {}

/// **Admin only**. Create an api key for a service user.
/// Response: [CreateApiKeyResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateApiKeyForServiceUserResponse)]
#[error(mogh_error::Error)]
pub struct CreateApiKeyForServiceUser {
  /// Must be service user
  pub user_id: String,
  /// The name for the api key
  pub name: String,
  /// A unix timestamp in millseconds specifying api key expire time.
  /// Default is 0, which means no expiry.
  #[serde(default)]
  pub expires: I64,
}

#[typeshare]
pub type CreateApiKeyForServiceUserResponse = CreateApiKeyResponse;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteApiKeyForServiceUser",
  description = "**Admin only.** Delete an api key for a service user.",
  request_body(content = DeleteApiKeyForServiceUser),
  responses(
    (status = 200, description = "The new alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn delete_api_key_for_service_user() {}

/// **Admin only.** Delete an api key for a service user.
/// Response: [NoData].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteApiKeyForServiceUserResponse)]
#[error(mogh_error::Error)]
pub struct DeleteApiKeyForServiceUser {
  pub key: String,
}

#[typeshare]
pub type DeleteApiKeyForServiceUserResponse = NoData;
