#[derive(Debug, Clone, clap::Parser)]
pub struct Connect {
  /// The server to connect to.
  pub server: String,

  /// Custom command to use to start the shell, eg `bash`.
  /// Defaults to Periphery default.
  pub command: Option<String>,

  /// The terminal name to connect to. Default: `ssh`
  #[arg(long, short = 'n', default_value_t = String::from("ssh"))]
  pub name: String,

  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Exec {
  /// The container (name) to connect to.
  /// Will error if matches multiple containers but no Server is defined.
  pub container: String,
  /// The shell, eg `bash`.
  pub shell: String,
  /// Specify Server.
  /// Required if multiple servers have same container name.
  /// (alias: `s`)
  #[arg(long, short = 's')]
  pub server: Option<String>,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Attach {
  /// The container (name) to attach to.
  /// Will error if matches multiple containers but no Server is defined.
  pub container: String,
  /// Specify Server.
  /// Required if multiple servers have same container name.
  /// (alias: `s`)
  #[arg(long, short = 's')]
  pub server: Option<String>,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}
