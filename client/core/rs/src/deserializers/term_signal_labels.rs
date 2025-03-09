use serde::{
  Deserialize, Deserializer,
  de::{Visitor, value::SeqAccessDeserializer},
};

use crate::entities::deployment::TerminationSignalLabel;

pub fn term_labels_deserializer<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(TermSignalLabelVisitor)
}

pub fn option_term_labels_deserializer<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(OptionTermSignalLabelVisitor)
}

struct TermSignalLabelVisitor;

impl<'de> Visitor<'de> for TermSignalLabelVisitor {
  type Value = String;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "string or Vec<TerminationSignalLabel>")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let out = v.to_string();
    if out.is_empty() || out.ends_with('\n') {
      Ok(out)
    } else {
      Ok(out + "\n")
    }
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let res = Vec::<TerminationSignalLabel>::deserialize(
      SeqAccessDeserializer::new(seq),
    )?
    .into_iter()
    .map(|TerminationSignalLabel { signal, label }| {
      format!("  {signal}: {label}")
    })
    .collect::<Vec<_>>()
    .join("\n");
    let extra = if res.is_empty() { "" } else { "\n" };
    Ok(res + extra)
  }
}

struct OptionTermSignalLabelVisitor;

impl<'de> Visitor<'de> for OptionTermSignalLabelVisitor {
  type Value = Option<String>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "null or string or Vec<TerminationSignalLabel>")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    TermSignalLabelVisitor.visit_str(v).map(Some)
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    TermSignalLabelVisitor.visit_seq(seq).map(Some)
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
