use anyhow::Context;
use axum::http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use serde::Serialize;
use serror::serialize_error;

pub struct Response(pub axum::response::Response);

impl<T> From<T> for Response
where
  T: Serialize,
{
  fn from(value: T) -> Response {
    let res = match serde_json::to_string(&value)
      .context("failed to serialize response body")
    {
      Ok(body) => axum::response::Response::builder()
        .header(
          CONTENT_TYPE,
          HeaderValue::from_static("application/json"),
        )
        .body(axum::body::Body::from(body))
        .unwrap(),
      Err(e) => axum::response::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(
          CONTENT_TYPE,
          HeaderValue::from_static("application/json"),
        )
        .body(axum::body::Body::from(serialize_error(&e)))
        .unwrap(),
    };
    Response(res)
  }
}

pub enum JsonString {
  Ok(String),
  Err(serde_json::Error),
}

impl<T> From<T> for JsonString
where
  T: Serialize,
{
  fn from(value: T) -> JsonString {
    match serde_json::to_string(&value) {
      Ok(body) => JsonString::Ok(body),
      Err(e) => JsonString::Err(e),
    }
  }
}

impl JsonString {
  pub fn into_response(self) -> axum::response::Response {
    match self {
      JsonString::Ok(body) => axum::response::Response::builder()
        .header(
          CONTENT_TYPE,
          HeaderValue::from_static("application/json"),
        )
        .body(axum::body::Body::from(body))
        .unwrap(),
      JsonString::Err(error) => axum::response::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(
          CONTENT_TYPE,
          HeaderValue::from_static("application/json"),
        )
        .body(axum::body::Body::from(serialize_error(
          &anyhow::Error::from(error)
            .context("failed to serialize response body"),
        )))
        .unwrap(),
    }
  }
}
