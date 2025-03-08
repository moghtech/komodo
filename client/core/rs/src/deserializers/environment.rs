use serde::{
  Deserialize, Deserializer,
  de::{Visitor, value::SeqAccessDeserializer},
};

use crate::entities::EnvironmentVar;

pub fn env_vars_deserializer<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(EnvironmentVarVisitor)
}

pub fn option_env_vars_deserializer<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(OptionEnvVarVisitor)
}

struct EnvironmentVarVisitor;

impl<'de> Visitor<'de> for EnvironmentVarVisitor {
  type Value = String;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "string or Vec<EnvironmentVar>")
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
    let vars = Vec::<EnvironmentVar>::deserialize(
      SeqAccessDeserializer::new(seq),
    )?;
    let vars = vars
      .iter()
      .map(|EnvironmentVar { variable, value }| {
        format!("  {variable} = {value}")
      })
      .collect::<Vec<_>>()
      .join("\n");
    let extra = if vars.is_empty() { "" } else { "\n" };
    Ok(vars + extra)
  }
}

struct OptionEnvVarVisitor;

impl<'de> Visitor<'de> for OptionEnvVarVisitor {
  type Value = Option<String>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "null or string or Vec<EnvironmentVar>")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    EnvironmentVarVisitor.visit_str(v).map(Some)
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    EnvironmentVarVisitor.visit_seq(seq).map(Some)
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
