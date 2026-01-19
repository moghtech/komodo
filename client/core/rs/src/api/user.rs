use mogh_resolver::{HasResponse, Resolve};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, NoData, ResourceTarget};

pub trait KomodoUserRequest: HasResponse {}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/user/PushRecentlyViewed",
  description = "Add a resource to user recently viewed.",
  request_body(content = PushRecentlyViewed),
  responses(
    (status = 200, description = "Successful", body = PushRecentlyViewedResponse),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn push_recently_viewed() {}

/// Push a resource to the front of the users 10 most recently viewed resources.
/// Response: [NoData].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoUserRequest)]
#[response(PushRecentlyViewedResponse)]
#[error(mogh_error::Error)]
pub struct PushRecentlyViewed {
  /// The target to push.
  pub resource: ResourceTarget,
}

#[typeshare]
pub type PushRecentlyViewedResponse = NoData;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/user/SetLastSeenUpdate",
  description = "Set the time the user last opened the UI updates.",
  request_body(content = SetLastSeenUpdate),
  responses(
    (status = 200, description = "Successful", body = SetLastSeenUpdateResponse),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn set_last_seen_update() {}

/// Set the time the user last opened the UI updates.
/// Used for unseen notification dot.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoUserRequest)]
#[response(SetLastSeenUpdateResponse)]
#[error(mogh_error::Error)]
pub struct SetLastSeenUpdate {}

#[typeshare]
pub type SetLastSeenUpdateResponse = NoData;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/user/CreateApiKey",
  description = "Create an api key for the calling user.",
  request_body(content = CreateApiKey),
  responses(
    (status = 200, description = "The api key and secret. The secret is not available again after this response is returned.", body = CreateApiKeyResponse),
    (status = 400, description = "Invalid api key name", body = mogh_error::Serror),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn create_api_key() {}

/// Create an api key for the calling user.
/// Response: [CreateApiKeyResponse].
///
/// Note. After the response is served, there will be no way
/// to get the secret later.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoUserRequest)]
#[response(CreateApiKeyResponse)]
#[error(mogh_error::Error)]
pub struct CreateApiKey {
  /// The name for the api key.
  pub name: String,

  /// A unix timestamp in millseconds specifying api key expire time.
  /// Default is 0, which means no expiry.
  #[serde(default)]
  pub expires: I64,
}

/// Response for [CreateApiKey].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateApiKeyResponse {
  /// X-API-KEY
  pub key: String,

  /// X-API-SECRET
  ///
  /// Note.
  /// There is no way to get the secret again after it is distributed in this message
  pub secret: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/user/DeleteApiKey",
  description = "Delete an api key for the calling user.",
  request_body(content = DeleteApiKey),
  responses(
    (status = 200, description = "Api key deleted.", body = DeleteApiKeyResponse),
    (status = 403, description = "Api key belongs to another user", body = mogh_error::Serror),
    (status = 404, description = "Api key not found", body = mogh_error::Serror),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn delete_api_key() {}

/// Delete an api key for the calling user.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoUserRequest)]
#[response(DeleteApiKeyResponse)]
#[error(mogh_error::Error)]
pub struct DeleteApiKey {
  /// The key which the user intends to delete.
  pub key: String,
}

#[typeshare]
pub type DeleteApiKeyResponse = NoData;
