use std::collections::HashMap;

use anyhow::{Context, anyhow};
use async_timing_util::{
  Timelength, get_timelength_in_ms, unix_timestamp_ms,
};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use komodo_client::entities::config::core::CoreConfig;
use mungos::mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::Mutex;

use crate::helpers::random_string;

type ExchangeTokenMap = Mutex<HashMap<String, (String, u128)>>;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
  pub id: String,
  pub iat: u128,
  pub exp: u128,
}

pub struct JwtClient {
  pub key: Hmac<Sha256>,
  ttl_ms: u128,
  exchange_tokens: ExchangeTokenMap,
}

impl JwtClient {
  pub fn new(config: &CoreConfig) -> anyhow::Result<JwtClient> {
    let secret = if config.jwt_secret.is_empty() {
      random_string(40)
    } else {
      config.jwt_secret.clone()
    };
    let key = Hmac::new_from_slice(secret.as_bytes())
      .context("failed at taking HmacSha256 of jwt secret")?;
    Ok(JwtClient {
      key,
      ttl_ms: get_timelength_in_ms(
        config.jwt_ttl.to_string().parse()?,
      ),
      exchange_tokens: Default::default(),
    })
  }

  pub fn generate(&self, user_id: String) -> anyhow::Result<String> {
    let iat = unix_timestamp_ms();
    let exp = iat + self.ttl_ms;
    let claims = JwtClaims {
      id: user_id,
      iat,
      exp,
    };
    let jwt = claims
      .sign_with_key(&self.key)
      .context("failed at signing claim")?;
    Ok(jwt)
  }

  #[instrument(level = "debug", skip_all)]
  pub async fn create_exchange_token(&self, jwt: String) -> String {
    let exchange_token = random_string(40);
    self.exchange_tokens.lock().await.insert(
      exchange_token.clone(),
      (
        jwt,
        unix_timestamp_ms()
          + get_timelength_in_ms(Timelength::OneMinute),
      ),
    );
    exchange_token
  }
  #[instrument(level = "debug", skip(self))]
  pub async fn redeem_exchange_token(
    &self,
    exchange_token: &str,
  ) -> anyhow::Result<String> {
    let (jwt, valid_until) = self
      .exchange_tokens
      .lock()
      .await
      .remove(exchange_token)
      .context("invalid exchange token: unrecognized")?;
    if unix_timestamp_ms() < valid_until {
      Ok(jwt)
    } else {
      Err(anyhow!("invalid exchange token: expired"))
    }
  }
}
