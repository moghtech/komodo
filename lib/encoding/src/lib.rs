//! Utilities for type-safe byte encoding and decoding.

use bytes::Bytes;

#[macro_use]
mod macros;

mod channel;
mod json;
mod response;

pub use channel::*;
pub use json::*;
pub use response::*;

pub trait Encode<Target>: Sized + Send {
  fn encode(self) -> Target;
  fn encode_into<T>(self) -> T
  where
    Target: Encode<T>,
  {
    self.encode().encode()
  }
}

pub trait Decode<Target>: Sized {
  fn decode(self) -> anyhow::Result<Target>;
  fn decode_into<T>(self) -> anyhow::Result<T>
  where
    Target: Decode<T>,
  {
    self.decode()?.decode()
  }
}

impl_identity!(Bytes);
impl_identity!(Vec<u8>);

/// Helps cast between the top level message types.
/// Implement whichever ones are most convenient for the source type.
pub trait CastBytes: Sized {
  fn from_bytes(bytes: Bytes) -> Self {
    Self::from_vec(bytes.into())
  }
  fn into_bytes(self) -> Bytes {
    self.into_vec().into()
  }
  fn from_vec(bytes: Vec<u8>) -> Self {
    Self::from_bytes(bytes.into())
  }
  fn into_vec(self) -> Vec<u8> {
    self.into_bytes().into()
  }
}

impl CastBytes for Bytes {
  fn from_bytes(bytes: Bytes) -> Self {
    bytes
  }
  fn into_bytes(self) -> Bytes {
    self
  }
}

impl CastBytes for Vec<u8> {
  fn from_vec(vec: Vec<u8>) -> Self {
    vec
  }
  fn into_vec(self) -> Vec<u8> {
    self
  }
}
