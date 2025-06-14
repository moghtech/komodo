use std::{sync::OnceLock, time::Duration};

use anyhow::Context;
use arc_swap::ArcSwapOption;
use openidconnect::{
  Client, ClientId, ClientSecret, EmptyAdditionalClaims,
  EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl,
  RedirectUrl, StandardErrorResponse, core::*,
};

use crate::config::core_config;

type OidcClient = Client<
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

pub fn oidc_client() -> &'static ArcSwapOption<OidcClient> {
  static OIDC_CLIENT: OnceLock<ArcSwapOption<OidcClient>> =
    OnceLock::new();
  OIDC_CLIENT.get_or_init(Default::default)
}

/// The OIDC client must be reinitialized to
/// pick up the latest provider JWKs. This
/// function spawns a management thread to do this
/// on a loop.
pub async fn spawn_oidc_client_management() {
  let config = core_config();
  if !config.oidc_enabled
    || config.oidc_provider.is_empty()
    || config.oidc_client_id.is_empty()
  {
    return;
  }
  if let Err(e) = reset_oidc_client().await {
    error!("Failed to initialize OIDC client | {e:#}");
  }
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(Duration::from_secs(60)).await;
      if let Err(e) = reset_oidc_client().await {
        warn!("Failed to reinitialize OIDC client | {e:#}");
      }
    }
  });
}

async fn reset_oidc_client() -> anyhow::Result<()> {
  let config = core_config();
  // Use OpenID Connect Discovery to fetch the provider metadata.
  let provider_metadata = CoreProviderMetadata::discover_async(
    IssuerUrl::new(config.oidc_provider.clone())?,
    super::reqwest_client(),
  )
  .await
  .context("Failed to get OIDC /.well-known/openid-configuration")?;

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

  oidc_client().store(Some(client.into()));

  Ok(())
}
