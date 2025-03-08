use serde::{Deserializer, de::Visitor};

/// Using this ensures the file contents end with trailing '\n'
pub fn file_contents_deserializer<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(FileContentsVisitor)
}

/// Using this ensures the file contents end with trailing '\n'
pub fn option_file_contents_deserializer<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(OptionFileContentsVisitor)
}

struct FileContentsVisitor;

impl Visitor<'_> for FileContentsVisitor {
  type Value = String;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "string")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let out = v.trim_end().to_string();
    if out.is_empty() {
      Ok(out)
    } else {
      Ok(out + "\n")
    }
  }
}

struct OptionFileContentsVisitor;

impl Visitor<'_> for OptionFileContentsVisitor {
  type Value = Option<String>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "null or string")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    FileContentsVisitor.visit_str(v).map(Some)
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
