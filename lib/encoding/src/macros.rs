#[macro_export]
macro_rules! impl_cast_bytes_vec {
  ($typ:ty, $through:ident) => {
    impl CastBytes for $typ {
      fn from_vec(bytes: Vec<u8>) -> Self {
        Self($through::from_vec(bytes))
      }
      fn into_vec(self) -> Vec<u8> {
        self.0.into_vec()
      }
    }
  };
}

#[macro_export]
macro_rules! impl_from_for_wrapper {
  ($typ:ty, $through:ty) => {
    impl From<$through> for $typ {
      fn from(value: $through) -> Self {
        Self(value)
      }
    }
  };
}

macro_rules! impl_identity {
  ($typ:ty) => {
    impl Encode<$typ> for $typ {
      fn encode(self) -> $typ {
        self
      }
    }
    impl Decode<$typ> for $typ {
      fn decode(self) -> anyhow::Result<$typ> {
        Ok(self)
      }
    }
  };
}

macro_rules! impl_wrapper {
  ($struct:ident) => {
    impl<T> From<T> for $struct<T> {
      fn from(value: T) -> Self {
        Self(value)
      }
    }
    impl<T: CastBytes> CastBytes for $struct<T> {
      fn from_bytes(bytes: Bytes) -> Self {
        Self(T::from_bytes(bytes))
      }
      fn into_bytes(self) -> Bytes {
        self.0.into_bytes()
      }
      fn from_vec(vec: Vec<u8>) -> Self {
        Self(T::from_vec(vec))
      }
      fn into_vec(self) -> Vec<u8> {
        self.0.into_vec()
      }
    }
  };
}
