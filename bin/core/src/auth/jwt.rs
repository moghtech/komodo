use anyhow::Context;
use async_timing_util::{get_timelength_in_ms, unix_timestamp_ms};
use database::mungos::mongodb::bson::doc;
use jsonwebtoken::{
  DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use komodo_client::{
  api::auth::JwtResponse,
  entities::{config::core::CoreConfig, random_string},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct JwtClaims {
  pub id: String,
  pub iat: u128,
  pub exp: u128,
}

pub struct JwtClient {
  header: Header,
  validation: Validation,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  ttl_ms: u128,
}

impl JwtClient {
  pub fn new(config: &CoreConfig) -> anyhow::Result<JwtClient> {
    let secret = if config.jwt_secret.is_empty() {
      random_string(40)
    } else {
      config.jwt_secret.clone()
    };
    Ok(JwtClient {
      header: Header::default(),
      validation: Validation::new(Default::default()),
      encoding_key: EncodingKey::from_secret(secret.as_bytes()),
      decoding_key: DecodingKey::from_secret(secret.as_bytes()),
      ttl_ms: get_timelength_in_ms(
        config.jwt_ttl.to_string().parse()?,
      ),
    })
  }

  pub fn encode(
    &self,
    user_id: String,
  ) -> anyhow::Result<JwtResponse> {
    let iat = unix_timestamp_ms();
    let exp = iat + self.ttl_ms;
    let claims = JwtClaims {
      id: user_id.clone(),
      iat,
      exp,
    };
    let jwt = encode(&self.header, &claims, &self.encoding_key)
      .context("Failed at signing claim")?;
    Ok(JwtResponse { user_id, jwt })
  }

  pub fn decode(&self, jwt: &str) -> anyhow::Result<JwtClaims> {
    decode::<JwtClaims>(jwt, &self.decoding_key, &self.validation)
      .map(|res| res.claims)
      .context("Failed to decode token claims")
  }
}
