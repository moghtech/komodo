use anyhow::{Context, Result as AnyhowResult};
use bytes::Bytes;
use mogh_error::{deserialize_error_bytes, serialize_error_bytes};

use crate::{CastBytes, Decode, Encode};

/// Message wrapper to handle Error unwrapping
/// anywhere in the en/decoding chain.
/// ```markdown
/// | -- u8[] -- | ------------------ u8 ------------------ |
/// | <CONTENTS> | 0: Ok, 1: Err, 2: Pending, 3: Progress |
/// ```
/// Progress is sent while the request is pending,
/// and is decoded as Pending by receivers which don't handle it.
#[derive(Clone, Debug)]
pub struct EncodedResponse<T>(T);

impl_wrapper!(EncodedResponse);

pub enum Response<T> {
  Ok(T),
  Err(anyhow::Error),
  Pending,
  Progress(T),
}

impl<T> Response<T> {
  pub fn into_anyhow(self) -> AnyhowResult<Option<T>> {
    self.into()
  }

  pub fn map<R>(self, map: impl FnOnce(T) -> R) -> Response<R> {
    use Response::*;
    match self {
      Ok(t) => Ok(map(t)),
      Err(e) => Err(e),
      Pending => Pending,
      Progress(t) => Progress(map(t)),
    }
  }

  pub fn map_encode<B: CastBytes + Send>(self) -> EncodedResponse<B>
  where
    T: Encode<B>,
  {
    self.map(Encode::encode).encode()
  }

  pub fn map_decode<D>(self) -> anyhow::Result<Option<D>>
  where
    T: CastBytes + Send + Decode<D>,
  {
    match self.map(Decode::decode) {
      Response::Ok(res) => res.map(Some),
      Response::Err(e) => Err(e),
      Response::Pending | Response::Progress(_) => Ok(None),
    }
  }
}

impl<T: CastBytes + Send> Encode<EncodedResponse<T>> for Response<T> {
  fn encode(self) -> EncodedResponse<T> {
    use Response::*;
    let bytes = match self {
      Ok(data) => {
        let mut bytes = data.into_vec();
        bytes.push(0);
        bytes
      }
      Err(e) => {
        let mut bytes = serialize_error_bytes(&e);
        bytes.push(1);
        bytes
      }
      Pending => {
        vec![2]
      }
      Progress(data) => {
        let mut bytes = data.into_vec();
        bytes.push(3);
        bytes
      }
    };
    EncodedResponse(T::from_vec(bytes))
  }
}

impl<T: CastBytes + Send> Encode<EncodedResponse<T>>
  for AnyhowResult<T>
{
  fn encode(self) -> EncodedResponse<T> {
    Response::from(self).encode()
  }
}

impl<T: CastBytes> Encode<EncodedResponse<T>> for &anyhow::Error {
  fn encode(self) -> EncodedResponse<T> {
    let mut bytes = serialize_error_bytes(self);
    bytes.push(1);
    EncodedResponse::from_vec(bytes)
  }
}

impl<T: CastBytes> Decode<Option<T>> for EncodedResponse<T> {
  fn decode(self) -> AnyhowResult<Option<T>> {
    self.decode_response()?.into()
  }
}

impl<T: CastBytes> EncodedResponse<T> {
  /// Like [Decode::decode], but keeps Pending and Progress apart.
  pub fn decode_response(self) -> AnyhowResult<Response<T>> {
    let mut bytes = self.0.into_vec();
    let result_byte =
      bytes.pop().context("ResultWrapper bytes cannot be empty")?;
    match result_byte {
      0 => Ok(Response::Ok(T::from_vec(bytes))),
      1 => {
        Err(deserialize_error_bytes(&bytes).context(
          "Decoded error message over Core-Periphery communication channel",
        ))
      }
      3 => Ok(Response::Progress(T::from_vec(bytes))),
      _ => Ok(Response::Pending),
    }
  }
}

impl<T> From<AnyhowResult<T>> for Response<T> {
  fn from(value: AnyhowResult<T>) -> Self {
    match value {
      Ok(t) => Self::Ok(t),
      Err(e) => Self::Err(e),
    }
  }
}

impl<T> From<Response<T>> for AnyhowResult<Option<T>> {
  fn from(value: Response<T>) -> Self {
    match value {
      Response::Ok(t) => Ok(Some(t)),
      Response::Err(e) => Err(e),
      Response::Pending | Response::Progress(_) => Ok(None),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn progress_is_decoded_as_pending_unless_handled() {
    let encoded: EncodedResponse<Vec<u8>> =
      Response::Progress(b"log".to_vec()).encode();
    assert_eq!(encoded.clone().into_vec(), b"log\x03");
    assert!(matches!(
      encoded.clone().decode_response(),
      Ok(Response::Progress(log)) if log == b"log"
    ));
    // Like Pending, which receivers that don't handle Progress expect.
    assert!(matches!(encoded.decode(), Ok(None)));
  }
}
