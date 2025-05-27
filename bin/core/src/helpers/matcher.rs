use anyhow::Context;

pub enum Matcher<'a> {
  Wildcard(wildcard::Wildcard<'a>),
  Regex(regex::Regex),
}

impl<'a> Matcher<'a> {
  pub fn new(pattern: &'a str) -> anyhow::Result<Self> {
    if pattern.starts_with('\\') && pattern.ends_with('\\') {
      let inner = &pattern[1..(pattern.len() - 1)];
      let regex = regex::Regex::new(inner)
        .with_context(|| format!("invalid regex. got: {inner}"))?;
      Ok(Self::Regex(regex))
    } else {
      let wildcard = wildcard::Wildcard::new(pattern.as_bytes())
        .with_context(|| {
          format!("invalid wildcard. got: {pattern}")
        })?;
      Ok(Self::Wildcard(wildcard))
    }
  }

  pub fn is_match(&self, source: &str) -> bool {
    match self {
      Matcher::Wildcard(wildcard) => {
        wildcard.is_match(source.as_bytes())
      }
      Matcher::Regex(regex) => regex.is_match(source),
    }
  }
}
