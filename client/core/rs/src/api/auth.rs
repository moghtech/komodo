use derive_empty_traits::EmptyTraits;
use resolver_api::{HasResponse, Resolve};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use webauthn_rs_proto::{
  PublicKeyCredential, RequestChallengeResponse,
};

use crate::entities::user::User;

pub trait KomodoAuthRequest: HasResponse {}

/// JSON containing a jwt authentication token.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwtResponse {
  /// User ID for signed in user.
  pub user_id: String,
  /// A token the user can use to authenticate their requests.
  pub jwt: String,
}

#[typeshare(serialized_as = "any")]
pub type _RequestChallengeResponse = RequestChallengeResponse;

/// JSON containing either an authentication token or a TwoFactor pending token.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JwtOrTwoFactor {
  Jwt(JwtResponse),
  Passkey(_RequestChallengeResponse),
  Totp {},
}

/// JSON containing either a user id or the required 2fa auth check.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserIdOrTwoFactor {
  UserId(String),
  Passkey(_RequestChallengeResponse),
  Totp {},
}

//

/// Non authenticated route to see the available options
/// users have to login to Komodo, eg. local auth, github, google.
/// Response: [GetLoginOptionsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(GetLoginOptionsResponse)]
#[error(serror::Error)]
pub struct GetLoginOptions {}

/// The response for [GetLoginOptions].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GetLoginOptionsResponse {
  /// Whether local auth is enabled.
  pub local: bool,
  /// Whether github login is enabled.
  pub github: bool,
  /// Whether google login is enabled.
  pub google: bool,
  /// Whether OIDC login is enabled.
  pub oidc: bool,
  /// Whether user registration (Sign Up) has been disabled
  pub registration_disabled: bool,
}

//

/// Sign up a new local user account. Will fail if a user with the
/// given username already exists.
/// Response: [SignUpLocalUserResponse].
///
/// Note. This method is only available if the core api has `local_auth` enabled,
/// and if user registration is not disabled (after the first user).
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(SignUpLocalUserResponse)]
#[error(serror::Error)]
pub struct SignUpLocalUser {
  /// The username for the new user.
  pub username: String,
  /// The password for the new user.
  /// This cannot be retreived later.
  pub password: String,
}

/// Response for [SignUpLocalUser].
#[typeshare]
pub type SignUpLocalUserResponse = JwtResponse;

//

/// Login as a local user. Will fail if the users credentials don't match
/// any local user.
///
/// Note. This method is only available if the core api has `local_auth` enabled.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(LoginLocalUserResponse)]
#[error(serror::Error)]
pub struct LoginLocalUser {
  /// The user's username
  pub username: String,
  /// The user's password
  pub password: String,
}

/// The response for [LoginLocalUser]
#[typeshare]
pub type LoginLocalUserResponse = JwtOrTwoFactor;

//

/// Retrieve a JWT after completing third party login flows.
/// Response: [ExchangeForJwtResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(ExchangeForJwtResponse)]
#[error(serror::Error)]
pub struct ExchangeForJwt {}

/// Response for [ExchangeForJwt].
#[typeshare]
pub type ExchangeForJwtResponse = JwtResponse;

//

/// Confirm a single use 2fa pending token + time-dependent user totp code for a jwt.
/// Response: [CompleteTotpLoginResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(CompleteTotpLoginResponse)]
#[error(serror::Error)]
pub struct CompleteTotpLogin {
  /// The time dependent totp code for user.
  pub code: String,
}

/// Response for [CompleteTotpLogin].
#[typeshare]
pub type CompleteTotpLoginResponse = JwtResponse;

//

#[typeshare(serialized_as = "any")]
pub type _PublicKeyCredential = PublicKeyCredential;

/// Confirm a single use 2fa pending token + time-dependent user totp code for a jwt.
/// Response: [CompletePasskeyLoginResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(CompletePasskeyLoginResponse)]
#[error(serror::Error)]
pub struct CompletePasskeyLogin {
  pub credential: _PublicKeyCredential,
}

/// Response for [CompletePasskeyLogin].
#[typeshare]
pub type CompletePasskeyLoginResponse = JwtResponse;

//

/// Get the user extracted from the request headers.
/// Response: [User].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoAuthRequest)]
#[response(GetUserResponse)]
#[error(serror::Error)]
pub struct GetUser {}

#[typeshare]
pub type GetUserResponse = User;

//
