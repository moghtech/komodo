use serde::{Deserializer, de::Visitor};

pub fn maybe_string_i64_deserializer<'de, D>(
  deserializer: D,
) -> Result<i64, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(MaybeStringI64Visitor)
}

pub fn option_maybe_string_i64_deserializer<'de, D>(
  deserializer: D,
) -> Result<Option<i64>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(OptionMaybeStringI64Visitor)
}

struct MaybeStringI64Visitor;

impl Visitor<'_> for MaybeStringI64Visitor {
  type Value = i64;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "number or string number")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    v.parse::<i64>().map_err(E::custom)
  }

  fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v)
  }

  fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(v as i64)
  }
}

struct OptionMaybeStringI64Visitor;

impl Visitor<'_> for OptionMaybeStringI64Visitor {
  type Value = Option<i64>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "null or number or string number")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    MaybeStringI64Visitor.visit_str(v).map(Some)
  }

  fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v))
  }

  fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Some(v as i64))
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(None)
  }

  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(None)
  }
}
