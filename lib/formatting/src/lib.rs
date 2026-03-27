use mogh_error::Serror;

pub fn muted(content: impl std::fmt::Display) -> String {
  format!(
    "<span style=\"color: var(--mantine-color-dimmed)\">{content}</span>"
  )
}

pub fn bold(content: impl std::fmt::Display) -> String {
  format!("<span style=\"font-weight: bolder\">{content}</span>")
}

pub fn colored(
  content: impl std::fmt::Display,
  color: Color,
) -> String {
  format!(
    "<span style=\"color: var(--mantine-color-{color}-6)\">{content}</span>"
  )
}

pub enum Color {
  Red,
  Green,
  Blue,
}

impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Color::Red => f.write_str("red"),
      Color::Green => f.write_str("green"),
      Color::Blue => f.write_str("blue"),
    }
  }
}

pub fn format_serror(Serror { error, trace }: &Serror) -> String {
  let trace = if !trace.is_empty() {
    let mut out = format!("\n\n{}:", muted("TRACE"));

    for (i, msg) in trace.iter().enumerate() {
      out.push_str(&format!("\n\t{}: {msg}", muted(i + 1)));
    }

    out
  } else {
    Default::default()
  };
  format!("{}: {error}{trace}", colored("ERROR", Color::Red))
}
