use serde::{
  Deserialize, Deserializer,
  de::{Visitor, value::SeqAccessDeserializer},
};

use crate::entities::deployment::Conversion;

pub fn conversions_deserializer<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(ConversionVisitor)
}

pub fn option_conversions_deserializer<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(OptionConversionVisitor)
}

struct ConversionVisitor;

impl<'de> Visitor<'de> for ConversionVisitor {
  type Value = String;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "string or Vec<Conversion>")
  }

  fn visit_string<E>(self, out: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    if out.is_empty() || out.ends_with('\n') {
      Ok(out)
    } else {
      Ok(out + "\n")
    }
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Self::visit_string(self, v.to_string())
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let res = Vec::<Conversion>::deserialize(
      SeqAccessDeserializer::new(seq),
    )?;
    let res = res
      .iter()
      .map(|Conversion { local, container }| {
        format!("  {local}: {container}")
      })
      .collect::<Vec<_>>()
      .join("\n");
    let extra = if res.is_empty() { "" } else { "\n" };
    Ok(res + extra)
  }
}

struct OptionConversionVisitor;

impl<'de> Visitor<'de> for OptionConversionVisitor {
  type Value = Option<String>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "null or string or Vec<Conversion>")
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    ConversionVisitor.visit_string(v).map(Some)
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    ConversionVisitor.visit_str(v).map(Some)
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    ConversionVisitor.visit_seq(seq).map(Some)
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
