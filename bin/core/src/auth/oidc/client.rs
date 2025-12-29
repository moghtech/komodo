use std::sync::{Arc, OnceLock};

use anyhow::{Context, anyhow};
use arc_swap::ArcSwapOption;
use openidconnect::{
  AccessToken, AccessTokenHash, AuthorizationCode, Client, ClientId,
  ClientSecret, CsrfToken, EmptyAdditionalClaims, EndpointMaybeSet,
  EndpointNotSet, EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse,
  PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
  StandardErrorResponse, SubjectIdentifier, TokenResponse as _,
  UserInfoClaims, core::*,
};

use crate::config::core_config;

static APP_USER_AGENT: &str =
  concat!("Komodo/", env!("CARGO_PKG_VERSION"),);

fn oidc_reqwest_client() -> &'static reqwest::Client {
  static REQWEST: OnceLock<reqwest::Client> = OnceLock::new();
  REQWEST.get_or_init(|| {
    reqwest::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .user_agent(APP_USER_AGENT)
      .build()
      .expect("Invalid OIDC reqwest client")
  })
}

pub type InnerOidcClient = Client<
  EmptyAdditionalClaims,
  CoreAuthDisplay,
  CoreGenderClaim,
  CoreJweContentEncryptionAlgorithm,
  CoreJsonWebKey,
  CoreAuthPrompt,
  StandardErrorResponse<CoreErrorResponseType>,
  CoreTokenResponse,
  CoreTokenIntrospectionResponse,
  CoreRevocableToken,
  CoreRevocationErrorResponse,
  EndpointSet,
  EndpointNotSet,
  EndpointNotSet,
  EndpointNotSet,
  EndpointMaybeSet,
  EndpointMaybeSet,
>;

/// Cache discovery data for 1min
const CLIENT_VALID_FOR_MS: u128 = 60_000;

fn oidc_client() -> &'static ArcSwapOption<OidcClient> {
  static OIDC_CLIENT: OnceLock<ArcSwapOption<OidcClient>> =
    OnceLock::new();
  OIDC_CLIENT.get_or_init(Default::default)
}

pub async fn load_oidc_client() -> Option<Arc<OidcClient>> {
  let now = async_timing_util::unix_timestamp_ms();

  if let Some(curr) = oidc_client().load().as_ref()
    && curr.1 > now
  {
    return Some(curr.clone());
  }

  let client = match OidcClient::new(now + CLIENT_VALID_FOR_MS).await
  {
    Ok(client) => Arc::new(client),
    Err(e) => {
      error!("Failed to initialize OIDC client | {e:#}");
      return None;
    }
  };

  oidc_client().store(Some(client.clone()));

  Some(client)
}

pub struct OidcClient(InnerOidcClient, u128);

impl OidcClient {
  /// Initialize a new OIDC client using the configured provider's
  /// discovery endpoint.
  pub async fn new(valid_until: u128) -> anyhow::Result<OidcClient> {
    let config = core_config();

    if !config.oidc_enabled() {
      return Err(anyhow!(
        "OIDC provider is disabled or not configured."
      ));
    }

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover_async(
      IssuerUrl::new(config.oidc_provider.clone())?,
      oidc_reqwest_client(),
    )
    .await
    .context(
      "Failed to get OIDC /.well-known/openid-configuration",
    )?;

    let client = CoreClient::from_provider_metadata(
      provider_metadata,
      ClientId::new(config.oidc_client_id.to_string()),
      // The secret may be empty / ommitted if auth provider supports PKCE
      if config.oidc_client_secret.is_empty() {
        None
      } else {
        Some(ClientSecret::new(config.oidc_client_secret.to_string()))
      },
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new(format!(
      "{}/auth/oidc/callback",
      core_config().host
    ))?);

    Ok(OidcClient(client, valid_until))
  }

  pub fn authorize_url(
    &self,
    pkce_challenge: PkceCodeChallenge,
  ) -> (url::Url, CsrfToken, Nonce) {
    self
      .0
      .authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
      )
      .set_pkce_challenge(pkce_challenge)
      .add_scope(Scope::new("openid".to_string()))
      .add_scope(Scope::new("profile".to_string()))
      .add_scope(Scope::new("email".to_string()))
      .url()
  }

  /// Applied security validations and extracts the
  /// oidc user id.
  pub async fn validate_extract_subject_and_token(
    &self,
    (client, server): (CsrfToken, String),
    code: String,
    pkce_verifier: PkceCodeVerifier,
    nonce: Nonce,
  ) -> anyhow::Result<(SubjectIdentifier, AccessToken)> {
    // Validate CSRF tokens match
    if client.secret() != &server {
      return Err(anyhow!("CSRF token invalid"));
    }

    let reqwest_client = oidc_reqwest_client();
    let token_response = self
      .0
      .exchange_code(AuthorizationCode::new(code))
      .context("Failed to get Oauth token at exchange code")?
      .set_pkce_verifier(pkce_verifier)
      .request_async(reqwest_client)
      .await
      .context("Failed to get Oauth token")?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
      .id_token()
      .context("OIDC Server did not return an ID token")?;

    // Some providers attach additional audiences, they must be added here
    // so token verification succeeds.
    let verifier = self.0.id_token_verifier();
    let additional_audiences =
      &core_config().oidc_additional_audiences;
    let verifier = if additional_audiences.is_empty() {
      verifier
    } else {
      verifier.set_other_audience_verifier_fn(|aud| {
        additional_audiences.contains(aud)
      })
    };

    let claims = id_token
      .claims(&verifier, &nonce)
      .context("Failed to verify token claims. This issue may be temporary (60 seconds max).")?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) =
      claims.access_token_hash()
    {
      let actual_access_token_hash = AccessTokenHash::from_token(
        &token_response.access_token().clone(),
        id_token.signing_alg()?,
        id_token.signing_key(&verifier)?,
      )?;
      if actual_access_token_hash != *expected_access_token_hash {
        return Err(anyhow!("Invalid access token"));
      }
    }

    Ok((
      claims.subject().clone(),
      token_response.access_token().clone(),
    ))
  }

  pub async fn fetch_user_info(
    &self,
    access_token: AccessToken,
    subject: SubjectIdentifier,
  ) -> anyhow::Result<
    UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim>,
  > {
    self
      .0
      .user_info(access_token, Some(subject))
      .context("Invalid user info request")?
      .request_async::<EmptyAdditionalClaims, _, CoreGenderClaim>(
        oidc_reqwest_client(),
      )
      .await
      .context("Failed to fetch OIDC user info")
  }
}
