use serde::{Deserializer, de::Visitor};

/// Using this ensures the command string is trimmed of trailing whitespace.
/// Unlike file_contents_deserializer, this does NOT add a trailing newline,
/// as commands should not have trailing newlines when executed.
pub fn command_deserializer<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(CommandVisitor)
}

struct CommandVisitor;

impl Visitor<'_> for CommandVisitor {
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
    Ok(v.trim_end().to_string())
  }
}
