use anyhow::anyhow;
use bytes::Bytes;
use uuid::Uuid;

use crate::{CastBytes, Decode, Encode};

/// Message wrapper to handle Error unwrapping
/// anywhere in the en/decoding chain.
/// ```markdown
/// | -- u8[] -- | -- [u8; 16] -- |
/// | <CONTENTS> |  Channel Uuid  |
/// ```
#[derive(Clone, Debug)]
pub struct EncodedChannel<T>(T);

impl_wrapper!(EncodedChannel);

impl<B: CastBytes> EncodedChannel<B> {
  pub fn decode_map<T>(self) -> anyhow::Result<WithChannel<T>>
  where
    B: Decode<T>,
  {
    let WithChannel { channel, data } = self.decode()?;
    let data = data.decode()?;
    Ok(WithChannel { channel, data })
  }
}

pub struct WithChannel<T> {
  pub channel: Uuid,
  pub data: T,
}

impl<T> WithChannel<T> {
  pub fn map<R>(self, map: impl FnOnce(T) -> R) -> WithChannel<R> {
    WithChannel {
      channel: self.channel,
      data: map(self.data),
    }
  }

  pub fn map_encode<B: CastBytes + Send>(self) -> EncodedChannel<B>
  where
    T: Encode<B>,
  {
    self.map(Encode::encode).encode()
  }

  pub fn map_decode<D>(self) -> anyhow::Result<WithChannel<D>>
  where
    T: CastBytes + Send + Decode<D>,
  {
    let WithChannel { channel, data } = self;
    let data = data.decode()?;
    Ok(WithChannel { channel, data })
  }
}

impl<T: CastBytes + Send> Encode<EncodedChannel<T>>
  for WithChannel<T>
{
  fn encode(self) -> EncodedChannel<T> {
    let mut bytes = self.data.into_vec();
    bytes.extend(self.channel.into_bytes());
    EncodedChannel(T::from_vec(bytes))
  }
}

impl<T: CastBytes> Decode<WithChannel<T>> for EncodedChannel<T> {
  fn decode(self) -> anyhow::Result<WithChannel<T>> {
    let mut bytes = self.0.into_vec();
    let len = bytes.len();
    if bytes.len() < 16 {
      return Err(anyhow!(
        "ChannelMessage bytes too short to include uuid"
      ));
    }
    let mut channel = [0u8; 16];
    for (i, byte) in bytes.drain(len - 16..).enumerate() {
      channel[i] = byte;
    }
    Ok(WithChannel {
      channel: Uuid::from_bytes(channel),
      data: T::from_vec(bytes),
    })
  }
}
