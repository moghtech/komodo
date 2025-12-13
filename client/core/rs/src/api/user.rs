use derive_empty_traits::EmptyTraits;
use resolver_api::{HasResponse, Resolve};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use webauthn_rs_proto::{
  CreationChallengeResponse, RegisterPublicKeyCredential,
};

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

/// Begin linking flow for a third party login. Response: [NoData].
///
/// First call this method when authenticated, then
/// redirect user to /api/auth/{provider}/link.
///
/// 'provider' can be:
/// - github
/// - google
/// - oidc
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(BeginThirdPartyLoginLinkResponse)]
#[error(serror::Error)]
pub struct BeginThirdPartyLoginLink {}

#[typeshare]
pub type BeginThirdPartyLoginLinkResponse = NoData;

//

/// Unlink a login. Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(UnlinkLoginResponse)]
#[error(serror::Error)]
pub struct UnlinkLogin {
  /// 'provider' can be:
  /// - Local
  /// - Github
  /// - Google
  /// - Oidc
  pub provider: String,
}

#[typeshare]
pub type UnlinkLoginResponse = NoData;

//

/// Update the calling users username.
/// Response: [NoData].
///
/// Will fail if the new username is invalid or already taken.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(UpdateUsernameResponse)]
#[error(serror::Error)]
pub struct UpdateUsername {
  pub username: String,
}

#[typeshare]
pub type UpdateUsernameResponse = NoData;

//

/// Update the calling user's password. Response: [NoData].
///
/// If the User was created using third party login method,
/// using [UpdatePassword] adds or updates the Local linked (additional) login method.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(UpdatePasswordResponse)]
#[error(serror::Error)]
pub struct UpdatePassword {
  pub password: String,
}

#[typeshare]
pub type UpdatePasswordResponse = NoData;

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
/// Response: [ConfirmTotpEnrollmentResponse]
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

/// Response for [ConfirmTotpEnrollment].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmTotpEnrollmentResponse {
  pub recovery_codes: Vec<String>,
}

//

/// Unenrolls user in TOTP 2FA.
/// Response: [UnenrollTotpResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(UnenrollTotpResponse)]
#[error(serror::Error)]
pub struct UnenrollTotp {}

/// Response for [UnenrollTotp].
#[typeshare]
pub type UnenrollTotpResponse = NoData;

//

#[typeshare(serialized_as = "any")]
pub type _CreationChallengeResponse = CreationChallengeResponse;

/// Starts enrollment flow for WebAuthn passkey auth support.
/// Response: [BeginPasskeyEnrollmentResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(BeginPasskeyEnrollmentResponse)]
#[error(serror::Error)]
pub struct BeginPasskeyEnrollment {}

/// Response for [BeginPasskeyEnrollment].
#[typeshare]
pub type BeginPasskeyEnrollmentResponse = _CreationChallengeResponse;

//

#[typeshare(serialized_as = "any")]
pub type _RegisterPublicKeyCredential = RegisterPublicKeyCredential;

/// Confirm enrollment flow for TOTP 2FA auth support
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(ConfirmPasskeyEnrollmentResponse)]
#[error(serror::Error)]
pub struct ConfirmPasskeyEnrollment {
  pub credential: _RegisterPublicKeyCredential,
}

/// Response for [ConfirmPasskeyEnrollment].
#[typeshare]
pub type ConfirmPasskeyEnrollmentResponse = NoData;

//

/// Unenrolls user in TOTP 2FA.
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(UnenrollPasskeyResponse)]
#[error(serror::Error)]
pub struct UnenrollPasskey {}

/// Response for [UnenrollPasskey].
#[typeshare]
pub type UnenrollPasskeyResponse = NoData;

//

/// Updates's whether user can skip 2fa when logging in using a third party (Oauth / OIDC).
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoUserRequest)]
#[response(UpdateThirdPartySkip2faResponse)]
#[error(serror::Error)]
pub struct UpdateThirdPartySkip2fa {
  pub skip: bool,
}

/// Response for [UpdateThirdPartySkip2fa].
#[typeshare]
pub type UpdateThirdPartySkip2faResponse = NoData;
