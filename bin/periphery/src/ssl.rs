use crate::config::periphery_config;

pub async fn ensure_certs() {
  let config = periphery_config();
  if !config.ssl_cert_file().is_file()
    || !config.ssl_key_file().is_file()
  {
    generate_self_signed_ssl_certs().await
  }
}

#[instrument]
async fn generate_self_signed_ssl_certs() {
  info!("Generating certs...");

  let config = periphery_config();

  let ssl_key_file = config.ssl_key_file();
  let ssl_cert_file = config.ssl_cert_file();

  // ensure cert folders exist
  if let Some(parent) = ssl_key_file.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if let Some(parent) = ssl_cert_file.parent() {
    let _ = std::fs::create_dir_all(parent);
  }

  let key_path = ssl_key_file.display();
  let cert_path = ssl_cert_file.display();

  let command = format!(
    "openssl req -x509 -newkey rsa:4096 -keyout {key_path} -out {cert_path} -sha256 -days 3650 -nodes -subj \"/C=XX/CN=periphery\""
  );
  let log = run_command::async_run_command(&command).await;

  if log.success() {
    info!("✅ SSL Certs generated");
  } else {
    panic!(
      "🚨 Failed to generate SSL Certs | stdout: {} | stderr: {}",
      log.stdout, log.stderr
    );
  }
}
