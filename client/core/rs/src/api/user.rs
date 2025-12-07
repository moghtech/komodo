use derive_empty_traits::EmptyTraits;
use resolver_api::{HasResponse, Resolve};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, NoData, ResourceTarget};

pub trait KomodoUserRequest: HasResponse {}

//

/// Push a resource to the front of the users 10 most recently viewed resources.
/// Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(PushRecentlyViewedResponse)]
#[error(serror::Error)]
pub struct PushRecentlyViewed {
  /// The target to push.
  pub resource: ResourceTarget,
}

#[typeshare]
pub type PushRecentlyViewedResponse = NoData;

//

/// Set the time the user last opened the UI updates.
/// Used for unseen notification dot.
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(SetLastSeenUpdateResponse)]
#[error(serror::Error)]
pub struct SetLastSeenUpdate {}

#[typeshare]
pub type SetLastSeenUpdateResponse = NoData;

//

/// Create an api key for the calling user.
/// Response: [CreateApiKeyResponse].
///
/// Note. After the response is served, there will be no way
/// to get the secret later.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(CreateApiKeyResponse)]
#[error(serror::Error)]
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

/// Delete an api key for the calling user.
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(DeleteApiKeyResponse)]
#[error(serror::Error)]
pub struct DeleteApiKey {
  /// The key which the user intends to delete.
  pub key: String,
}

#[typeshare]
pub type DeleteApiKeyResponse = NoData;

//

/// Starts enrollment flow for TOTP 2FA auth support.
/// Response: [BeginTotpEnrollmentResponse]
///
/// This generates an otpauth URI for the user. User must confirm
/// by providing a valid 6 digit code for the URI to [ConfirmTotpEnrollment].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(BeginTotpEnrollmentResponse)]
#[error(serror::Error)]
pub struct BeginTotpEnrollment {}

/// Response for [BeginTotpEnrollment].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeginTotpEnrollmentResponse {
  pub uri: String,
  /// Base64 encoded PNG embeddable in HTML to display uri QR code.
  pub png: String,
}

//

/// Confirm enrollment flow for TOTP 2FA auth support
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(ConfirmTotpEnrollmentResponse)]
#[error(serror::Error)]
pub struct ConfirmTotpEnrollment {
  pub code: String,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmTotpEnrollmentResponse {
  pub recovery_codes: Vec<String>,
}
