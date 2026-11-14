#![allow(unused_crate_dependencies)]

use std::{
  os::unix::fs::PermissionsExt,
  path::{Path, PathBuf},
  process::{Child, Command, Stdio},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use data_encoding::BASE64;
use database::mungos::mongodb::{self, bson::Document};
use futures_util::TryStreamExt;
use indexmap::IndexSet;
use komodo_client::{
  KomodoClient,
  api::{
    execute::{
      Deploy, DeployStack, DeployStackIfChanged, DestroyDeployment,
      DestroyStack, RestartStack, RunBuild, StartStack, StopStack,
    },
    read::{
      ExportAllResourcesToToml, GetDeploymentLog, GetSecretProvider,
      GetServerState, GetStack, GetStackLog, GetUpdate,
      InspectDeploymentContainer, InspectStackContainer,
      ListAvailableSecretProviders, ListExternalSecretKeys,
      ListSecretProviderUsages, ListSecretProviders, ListUpdates,
      PreviewResourceSecrets,
    },
    write::{
      CreateBuild, CreateBuilder, CreateDeployment,
      CreateSecretProvider, CreateSecretProviderGrant, CreateServer,
      CreateStack, CreateVariable, DeleteBuild, DeleteDeployment,
      DeleteSecretProvider, DeleteStack,
      RotateSecretProviderEncryptionKeys,
      TestSecretProviderConnection, UpdatePermissionOnTarget,
      UpdateSecretProvider, UpdateServer, UpdateUserBasePermissions,
    },
  },
  entities::{
    ResourceTarget,
    build::PartialBuildConfig,
    builder::{_PartialServerBuilderConfig, PartialBuilderConfig},
    deployment::{DeploymentImage, PartialDeploymentConfig},
    permission::{
      PermissionLevel, PermissionLevelAndSpecifics,
      SpecificPermission, UserTarget,
    },
    secret_provider::{
      BuildSecretBinding, CredentialUpdate, DeploymentSecretBinding,
      InfisicalProviderConfig, InfisicalSecretSelector,
      InfisicalSelectorScope, SecretCheckStageStatus,
      SecretConflictPolicy, SecretGrantSubject, SecretProviderConfig,
      SecretProviderCredentialInput, SecretResourceScope,
      SecretSelector, SecretSelectorScope, SecretUsage,
      SecretValueFormat, StackSecretBinding, StackSecretTarget,
    },
    server::{PartialServerConfig, ServerState},
    stack::PartialStackConfig,
    update::Update,
  },
};
use mogh_pki::{PkiKind, RotatableKeyPair};
use reqwest::Client;
use rustls::pki_types::{
  CertificateDer, PrivateKeyDer, pem::PemObject,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

struct GeneratedPki {
  ca_cert_pem: String,
  server_cert_pem: String,
  server_key_pem: String,
  server_cert_path: PathBuf,
  server_key_path: PathBuf,
}

fn generate_pki(dir: &Path, cn_label: &str) -> GeneratedPki {
  std::fs::create_dir_all(dir).expect("create PKI dir");
  let ca_key = dir.join("ca.key");
  let ca_crt = dir.join("ca.crt");
  let ca_cnf = dir.join("ca.cnf");
  std::fs::write(
    &ca_cnf,
    format!(
      "[req]\ndistinguished_name=dn\nx509_extensions=v3_ca\nprompt=no\n[dn]\nCN={cn_label}\n[v3_ca]\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,keyCertSign,cRLSign\nsubjectKeyIdentifier=hash\n"
    ),
  )
  .unwrap();

  let out = Command::new("openssl")
    .args([
      "req",
      "-x509",
      "-newkey",
      "ec",
      "-pkeyopt",
      "ec_paramgen_curve:prime256v1",
      "-days",
      "30",
      "-nodes",
      "-keyout",
      ca_key.to_str().unwrap(),
      "-out",
      ca_crt.to_str().unwrap(),
      "-config",
      ca_cnf.to_str().unwrap(),
    ])
    .output()
    .expect("run openssl req -x509");
  assert!(
    out.status.success(),
    "openssl CA gen failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  let srv_key = dir.join("server.key");
  let srv_csr = dir.join("server.csr");
  let srv_crt = dir.join("server.crt");
  let srv_ext = dir.join("server.ext");
  std::fs::write(
    &srv_ext,
    "basicConstraints=CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=IP:127.0.0.1,DNS:localhost\n",
  )
  .unwrap();

  let out = Command::new("openssl")
    .args([
      "req",
      "-newkey",
      "ec",
      "-pkeyopt",
      "ec_paramgen_curve:prime256v1",
      "-nodes",
      "-keyout",
      srv_key.to_str().unwrap(),
      "-out",
      srv_csr.to_str().unwrap(),
      "-subj",
      "/CN=127.0.0.1",
    ])
    .output()
    .expect("run openssl csr");
  assert!(
    out.status.success(),
    "openssl CSR failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  let out = Command::new("openssl")
    .args([
      "x509",
      "-req",
      "-in",
      srv_csr.to_str().unwrap(),
      "-CA",
      ca_crt.to_str().unwrap(),
      "-CAkey",
      ca_key.to_str().unwrap(),
      "-CAcreateserial",
      "-out",
      srv_crt.to_str().unwrap(),
      "-days",
      "30",
      "-extfile",
      srv_ext.to_str().unwrap(),
    ])
    .output()
    .expect("run openssl x509 sign");
  assert!(
    out.status.success(),
    "openssl cert sign failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  GeneratedPki {
    ca_cert_pem: std::fs::read_to_string(&ca_crt).unwrap(),
    server_cert_pem: std::fs::read_to_string(&srv_crt).unwrap(),
    server_key_pem: std::fs::read_to_string(&srv_key).unwrap(),
    server_cert_path: srv_crt,
    server_key_path: srv_key,
  }
}

fn build_tls_acceptor(pki: &GeneratedPki) -> TlsAcceptor {
  let _ =
    rustls::crypto::aws_lc_rs::default_provider().install_default();
  let certs: Vec<CertificateDer<'static>> =
    CertificateDer::pem_slice_iter(pki.server_cert_pem.as_bytes())
      .map(|c| c.unwrap())
      .collect();
  let key =
    PrivateKeyDer::from_pem_slice(pki.server_key_pem.as_bytes())
      .unwrap();
  let cfg = rustls::ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .unwrap();
  TlsAcceptor::from(Arc::new(cfg))
}

fn run_docker(args: &[&str]) -> String {
  let out = Command::new("docker")
    .args(args)
    .output()
    .expect("failed to invoke docker CLI");
  assert!(
    out.status.success(),
    "docker {:?} failed:\nstdout: {}\nstderr: {}",
    args,
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn pick_unused_port() -> u16 {
  let listener =
    std::net::TcpListener::bind("127.0.0.1:0").expect("bind 0");
  listener.local_addr().unwrap().port()
}

struct LiveE2eHarness {
  temp_root: PathBuf,
  network_name: String,
  mongo_container: String,
  pg_container: String,
  redis_container: String,
  infisical_container: String,
  compose_project: String,
  deployment_container: String,
  built_image_name: String,
  core_child: Option<Child>,
  periphery_child: Option<Child>,
}

impl LiveE2eHarness {
  fn cleanup(&mut self) {
    if let Some(mut child) = self.core_child.take() {
      let _ = child.kill();
      let _ = child.wait();
    }
    if let Some(mut child) = self.periphery_child.take() {
      let _ = child.kill();
      let _ = child.wait();
    }

    let _ = Command::new("docker")
      .args([
        "compose",
        "-p",
        &self.compose_project,
        "down",
        "-v",
        "--remove-orphans",
      ])
      .output();

    let _ = Command::new("docker")
      .args(["rm", "-f", &self.deployment_container])
      .output();

    for tag in ["latest", "0.0.1", "0.0", "0"] {
      let _ = Command::new("docker")
        .args([
          "rmi",
          "-f",
          &format!("{}:{tag}", self.built_image_name),
        ])
        .output();
    }

    for container in [
      &self.infisical_container,
      &self.redis_container,
      &self.pg_container,
      &self.mongo_container,
    ] {
      let _ = Command::new("docker")
        .args(["rm", "-f", container])
        .output();
    }

    let _ = Command::new("docker")
      .args(["network", "rm", &self.network_name])
      .output();

    let _ = std::fs::remove_dir_all(&self.temp_root);
  }
}

impl Drop for LiveE2eHarness {
  fn drop(&mut self) {
    self.cleanup();
  }
}

async fn execute_and_wait<T>(
  client: &KomodoClient,
  req: T,
) -> anyhow::Result<Update>
where
  T: serde::Serialize
    + komodo_client::api::execute::KomodoExecuteRequest<
      Response = Update,
    >,
{
  let initial: Update = client.execute(req).await?;
  if initial.id.is_empty()
    || initial.status
      == komodo_client::entities::update::UpdateStatus::Complete
  {
    return Ok(initial);
  }
  client.poll_update_until_complete(&initial.id).await
}

fn format_update_logs(update: &Update) -> String {
  update
    .logs
    .iter()
    .map(|l| {
      format!(
        "[{} ok={}] stdout={} stderr={}",
        l.stage, l.success, l.stdout, l.stderr
      )
    })
    .collect::<Vec<_>>()
    .join("\n")
}

async fn create_local_user_api_client(
  http: &Client,
  core_url: &str,
  username: &str,
  password: &str,
  sign_up: bool,
  enable_via_admin: Option<&KomodoClient>,
) -> (String, KomodoClient) {
  let jwt = if sign_up {
    let resp: Value = http
      .post(format!("{core_url}/auth/login/SignUpLocalUser"))
      .json(&json!({
        "username": username,
        "password": password,
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap()
      .json()
      .await
      .unwrap();
    resp["jwt"]
      .as_str()
      .expect("missing jwt on signup")
      .to_string()
  } else {
    let resp: Value = http
      .post(format!("{core_url}/auth/login/LoginLocalUser"))
      .json(&json!({
        "username": username,
        "password": password,
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap()
      .json()
      .await
      .unwrap();
    resp["data"]["jwt"]
      .as_str()
      .expect("missing data.jwt on login")
      .to_string()
  };

  let user_resp: Value = http
    .get(format!("{core_url}/user"))
    .header("authorization", &jwt)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let user_id = user_resp["_id"]["$oid"]
    .as_str()
    .or_else(|| user_resp["id"].as_str())
    .or_else(|| user_resp["_id"].as_str())
    .expect("missing user id on /user")
    .to_string();

  if let Some(admin) = enable_via_admin {
    admin
      .write(UpdateUserBasePermissions {
        user_id: user_id.clone(),
        enabled: Some(true),
        create_servers: Some(false),
        create_builds: Some(false),
      })
      .await
      .expect("enable newly signed up user");
  }

  let key_resp: Value = http
    .post(format!("{core_url}/auth/manage/CreateApiKey"))
    .header("authorization", &jwt)
    .json(&json!({
      "name": format!("e2e-key-{username}"),
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();

  let api_key = key_resp["key"].as_str().unwrap().to_string();
  let api_secret = key_resp["secret"].as_str().unwrap().to_string();
  (user_id, KomodoClient::new(core_url, api_key, api_secret))
}

fn assert_no_canaries(
  label: &str,
  haystack: &str,
  canaries: &[String],
) {
  for canary in canaries {
    assert!(
      !haystack.contains(canary),
      "SECURITY LEAK DETECTED in {label}: found canary substring '{canary}'!"
    );
  }
}

#[tokio::test]
async fn live_end_to_end_external_secrets_compose_deployment_buildkit_and_canary_audit()
 {
  let suffix = Uuid::new_v4().simple().to_string()[..8].to_string();
  let raw_temp =
    std::env::temp_dir().join(format!("komodo-e2e-{suffix}"));
  std::fs::create_dir_all(&raw_temp).unwrap();
  let temp_root = std::fs::canonicalize(&raw_temp).unwrap();

  let mut harness = LiveE2eHarness {
    temp_root: temp_root.clone(),
    network_name: format!("komodo-e2e-net-{suffix}"),
    mongo_container: format!("komodo-e2e-mongo-{suffix}"),
    pg_container: format!("komodo-e2e-pg-{suffix}"),
    redis_container: format!("komodo-e2e-valkey-{suffix}"),
    infisical_container: format!("komodo-e2e-infisical-{suffix}"),
    compose_project: format!("komodo_e2e_stack_{suffix}"),
    deployment_container: format!("komodo-e2e-deploy-{suffix}"),
    built_image_name: format!("komodo-e2e-build-{suffix}"),
    core_child: None,
    periphery_child: None,
  };

  // Ensure `periphery` binary exists next to `core` binary
  let core_bin = PathBuf::from(env!("CARGO_BIN_EXE_core"));
  let periphery_bin = core_bin.with_file_name("periphery");
  if !periphery_bin.exists() {
    let status = Command::new("cargo")
      .args(["build", "--bin", "periphery"])
      .status()
      .expect("failed to build periphery binary");
    assert!(status.success(), "cargo build --bin periphery failed");
  }

  // 1. Start MongoDB, Postgres, Valkey, and Infisical v0.162.19
  run_docker(&["network", "create", &harness.network_name]);

  run_docker(&[
    "run",
    "-d",
    "--name",
    &harness.mongo_container,
    "-p",
    "127.0.0.1::27017",
    "mongo:7.0.34",
  ]);
  let mongo_port_str =
    run_docker(&["port", &harness.mongo_container, "27017/tcp"]);
  let mongo_addr =
    mongo_port_str.lines().next().unwrap().trim().to_string();

  run_docker(&[
    "run",
    "-d",
    "--name",
    &harness.pg_container,
    "--network",
    &harness.network_name,
    "--network-alias",
    "infisical-db",
    "-e",
    "POSTGRES_USER=infisical",
    "-e",
    "POSTGRES_PASSWORD=infisical_e2e_pw",
    "-e",
    "POSTGRES_DB=infisical",
    "postgres:18.1-alpine",
  ]);

  run_docker(&[
    "run",
    "-d",
    "--name",
    &harness.redis_container,
    "--network",
    &harness.network_name,
    "--network-alias",
    "infisical-redis",
    "valkey/valkey:9.0.1-alpine",
  ]);

  // Wait for MongoDB & Postgres readiness
  let infra_deadline = Instant::now() + Duration::from_secs(40);
  loop {
    let mongo_ok = Command::new("docker")
      .args([
        "exec",
        &harness.mongo_container,
        "mongosh",
        "--quiet",
        "--eval",
        "db.adminCommand('ping').ok",
      ])
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false);
    let pg_ok = Command::new("docker")
      .args([
        "exec",
        &harness.pg_container,
        "pg_isready",
        "-U",
        "infisical",
      ])
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false);
    if mongo_ok && pg_ok {
      break;
    }
    assert!(
      Instant::now() < infra_deadline,
      "MongoDB or Postgres did not become ready in time"
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
  }

  run_docker(&[
    "run",
    "-d",
    "--name",
    &harness.infisical_container,
    "--network",
    &harness.network_name,
    "-p",
    "127.0.0.1::8080",
    "-e",
    "ENCRYPTION_KEY=6c1fe4e407b8911c104518103505b218",
    "-e",
    "AUTH_SECRET=55f9ce8f3e3b46b29a2e9a2a8f7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b",
    "-e",
    "DB_CONNECTION_URI=postgres://infisical:infisical_e2e_pw@infisical-db:5432/infisical",
    "-e",
    "REDIS_URL=redis://infisical-redis:6379",
    "-e",
    "TELEMETRY_ENABLED=false",
    "-e",
    "LEGACY_IDENTITY_ACCESS_TOKEN_EXPIRATION_ENFORCED_AT=2099-01-01T00:00:00.000Z",
    "infisical/infisical:v0.162.19",
  ]);

  let inf_port_out =
    run_docker(&["port", &harness.infisical_container, "8080/tcp"]);
  let inf_upstream_addr =
    inf_port_out.lines().next().unwrap().trim().to_string();

  // 2. Front Infisical with an isolated Root CA TLS 1.3 proxy counting `/api/v4/secrets` requests
  let infisical_pki = generate_pki(
    &temp_root.join("pki-infisical"),
    "Infisical E2E Root CA",
  );
  let inf_acceptor = build_tls_acceptor(&infisical_pki);
  let tls_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let inf_tls_port = tls_listener.local_addr().unwrap().port();

  let v4_value_fetches = Arc::new(AtomicUsize::new(0));
  let v4_metadata_fetches = Arc::new(AtomicUsize::new(0));
  let v4_val_counter = v4_value_fetches.clone();
  let v4_meta_counter = v4_metadata_fetches.clone();
  let upstream_target_addr = inf_upstream_addr.clone();

  let _inf_proxy = tokio::spawn(async move {
    while let Ok((client_tcp, _)) = tls_listener.accept().await {
      let acceptor = inf_acceptor.clone();
      let upstream_addr = upstream_target_addr.clone();
      let val_ctr = v4_val_counter.clone();
      let meta_ctr = v4_meta_counter.clone();
      tokio::spawn(async move {
        let Ok(tls_stream) = acceptor.accept(client_tcp).await else {
          return;
        };
        let Ok(upstream_tcp) =
          TcpStream::connect(&upstream_addr).await
        else {
          return;
        };
        let (mut c_read, mut c_write) = tokio::io::split(tls_stream);
        let (mut u_read, mut u_write) =
          tokio::io::split(upstream_tcp);

        let c_to_u = async move {
          let mut buf = [0u8; 8192];
          let mut rolling = String::new();
          while let Ok(n) = c_read.read(&mut buf).await {
            if n == 0 {
              break;
            }
            rolling.push_str(&String::from_utf8_lossy(&buf[..n]));
            while let Some(line_end) = rolling.find("\r\n") {
              let line = rolling[..line_end].to_string();
              rolling.drain(..line_end + 2);
              if let Some(idx) = line.find("GET /api/v4/secrets?") {
                let req_line = &line[idx..];
                if req_line.contains("viewSecretValue=false") {
                  meta_ctr.fetch_add(1, Ordering::SeqCst);
                } else {
                  val_ctr.fetch_add(1, Ordering::SeqCst);
                }
              }
            }
            if rolling.len() > 16384 {
              rolling.clear();
            }
            if u_write.write_all(&buf[..n]).await.is_err() {
              break;
            }
          }
          let _ = u_write.shutdown().await;
        };

        let u_to_c = async move {
          let _ = tokio::io::copy(&mut u_read, &mut c_write).await;
          let _ = c_write.shutdown().await;
        };

        tokio::join!(c_to_u, u_to_c);
      });
    }
  });

  let infisical_https_endpoint =
    format!("https://127.0.0.1:{inf_tls_port}");
  let raw_inf_http = format!("http://{inf_upstream_addr}");
  let http = Client::new();

  // Wait for Infisical readiness
  let inf_deadline = Instant::now() + Duration::from_secs(90);
  loop {
    if let Ok(resp) =
      http.get(format!("{raw_inf_http}/api/status")).send().await
      && resp.status().is_success()
    {
      break;
    }
    assert!(
      Instant::now() < inf_deadline,
      "Infisical container did not become healthy in time"
    );
    tokio::time::sleep(Duration::from_millis(500)).await;
  }

  // 3. Bootstrap Infisical Organization, Project, `/app` Folder, Universal Auth Machine Identity & Secrets
  let bootstrap_resp: Value = http
    .post(format!("{raw_inf_http}/api/v1/admin/bootstrap"))
    .json(&json!({
      "email": "admin@komodo-e2e.test",
      "password": "KomodoE2eBootstrapPassword#2026!",
      "organization": "komodo-e2e-org"
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let org_id = bootstrap_resp["organization"]["id"]
    .as_str()
    .unwrap()
    .to_string();
  let org_slug = bootstrap_resp["organization"]["slug"]
    .as_str()
    .unwrap()
    .to_string();
  let inf_admin_token =
    bootstrap_resp["identity"]["credentials"]["token"]
      .as_str()
      .unwrap()
      .to_string();

  let proj_resp: Value = http
    .post(format!("{raw_inf_http}/api/v1/projects"))
    .bearer_auth(&inf_admin_token)
    .json(&json!({ "projectName": "komodo-e2e-project" }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let inf_project_id =
    proj_resp["project"]["id"].as_str().unwrap().to_string();

  http
    .post(format!("{raw_inf_http}/api/v2/folders"))
    .bearer_auth(&inf_admin_token)
    .json(&json!({
      "projectId": inf_project_id,
      "environment": "prod",
      "name": "app",
      "path": "/"
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  let id_resp: Value = http
    .post(format!("{raw_inf_http}/api/v1/identities"))
    .bearer_auth(&inf_admin_token)
    .json(&json!({
      "name": "komodo-core-e2e-mi",
      "organizationId": org_id,
      "role": "no-access"
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let identity_id =
    id_resp["identity"]["id"].as_str().unwrap().to_string();

  let ua_resp: Value = http
    .post(format!(
      "{raw_inf_http}/api/v1/auth/universal-auth/identities/{identity_id}"
    ))
    .bearer_auth(&inf_admin_token)
    .json(&json!({
      "accessTokenTTL": 120,
      "accessTokenMaxTTL": 3600
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let reader_client_id = ua_resp["identityUniversalAuth"]["clientId"]
    .as_str()
    .unwrap()
    .to_string();

  let cs_resp: Value = http
    .post(format!(
      "{raw_inf_http}/api/v1/auth/universal-auth/identities/{identity_id}/client-secrets"
    ))
    .bearer_auth(&inf_admin_token)
    .json(&json!({
      "description": "komodo-e2e-secret",
      "ttl": 0,
      "numUsesLimit": 0
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let reader_client_secret =
    cs_resp["clientSecret"].as_str().unwrap().to_string();

  http
    .post(format!(
      "{raw_inf_http}/api/v1/projects/{inf_project_id}/memberships/identities/{identity_id}"
    ))
    .bearer_auth(&inf_admin_token)
    .json(&json!({
      "roles": [{ "role": "viewer", "isTemporary": false }]
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  // High-entropy canaries + exact Section 4.1 contract values
  let contract_database_url =
    "postgres://user:p%40ss%23word%2F123@db:5432/app";
  let contract_db_password = "p@ss#word/123";
  // Note: Infisical v0.162.19 `POST /api/v4/secrets` applies `z.string().transform(val => val.trim())` on stored values;
  // internal multiple spaces and all shell/Compose/Komodo metacharacters survive verbatim.
  let contract_special_literal = "inner  spaces $DOLLAR ${UNSET_SHELL} [[DB_USER]] 'single' \"double\" \\backslash #hash &amp =eq +plus";
  let canary_dot_key =
    format!("CANARY_EXT_DOT_KEY_{}", Uuid::new_v4().simple());
  let canary_svc_sec =
    format!("CANARY_EXT_SVC_SEC_{}", Uuid::new_v4().simple());
  let canary_svc_sec_rotated =
    format!("CANARY_EXT_SVC_ROTATED_{}", Uuid::new_v4().simple());
  let canary_file_sec = format!(
    "CANARY_EXT_FILE_SEC_{}\n-----END TEST SECRET-----\n",
    Uuid::new_v4().simple()
  );
  let canary_deploy_sec =
    format!("CANARY_EXT_DEPLOY_SEC_{}", Uuid::new_v4().simple());
  let canary_build_sec =
    format!("CANARY_EXT_BUILD_SEC_{}", Uuid::new_v4().simple());

  let all_canaries = vec![
    reader_client_secret.clone(),
    String::from("p%40ss%23word%2F123"),
    String::from(contract_db_password),
    canary_dot_key.clone(),
    canary_svc_sec.clone(),
    canary_svc_sec_rotated.clone(),
    canary_file_sec.trim().to_string(),
    canary_deploy_sec.clone(),
    canary_build_sec.clone(),
  ];

  for (key, value) in [
    ("DATABASE_URL", contract_database_url),
    ("DB_PASSWORD", contract_db_password),
    ("SPECIAL_LITERAL", contract_special_literal),
    ("service.jwt.secret", canary_dot_key.as_str()),
    ("API_ONLY_SECRET", canary_svc_sec.as_str()),
    ("FILE_CERT_PEM", canary_file_sec.as_str()),
    ("DEPLOY_TOKEN", canary_deploy_sec.as_str()),
    ("BUILD_REGISTRY_TOKEN", canary_build_sec.as_str()),
  ] {
    let encoded_key = urlencoding::encode(key);
    http
      .post(format!("{raw_inf_http}/api/v4/secrets/{encoded_key}"))
      .bearer_auth(&inf_admin_token)
      .json(&json!({
        "projectId": inf_project_id,
        "environment": "prod",
        "secretPath": "/app",
        "secretValue": value,
        "secretComment": "e2e-seeded"
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
  }

  // 4. Start Real Periphery (with distinct Root CA TLS + mutual X25519 Noise keys) and Real Core
  let periphery_pki = generate_pki(
    &temp_root.join("pki-periphery"),
    "Periphery E2E Root CA",
  );
  let keys_dir = temp_root.join("noise-keys");
  std::fs::create_dir_all(&keys_dir).unwrap();
  let core_noise_key_path = keys_dir.join("core.key");
  let periphery_noise_key_path = keys_dir.join("periphery.key");

  let core_kp = RotatableKeyPair::from_private_key_spec(
    PkiKind::Mutual,
    &format!("file:{}", core_noise_key_path.display()),
  )
  .unwrap();
  let periphery_kp = RotatableKeyPair::from_private_key_spec(
    PkiKind::Mutual,
    &format!("file:{}", periphery_noise_key_path.display()),
  )
  .unwrap();
  let core_public_key = core_kp.load().public.as_str().to_string();
  let periphery_public_key =
    periphery_kp.load().public.as_str().to_string();

  let periphery_root = temp_root.join("periphery-root");
  std::fs::create_dir_all(&periphery_root).unwrap();
  let periphery_root =
    std::fs::canonicalize(&periphery_root).unwrap();
  let periphery_port = pick_unused_port();

  let periphery_log_path = temp_root.join("periphery.log");
  let periphery_log_file =
    std::fs::File::create(&periphery_log_path).unwrap();
  let periphery_err_file = periphery_log_file.try_clone().unwrap();

  let periphery_child = Command::new(&periphery_bin)
    .env("PERIPHERY_PORT", periphery_port.to_string())
    .env("PERIPHERY_BIND_IP", "127.0.0.1")
    .env("PERIPHERY_ROOT_DIRECTORY", &periphery_root)
    .env(
      "PERIPHERY_PRIVATE_KEY",
      format!("file:{}", periphery_noise_key_path.display()),
    )
    .env("PERIPHERY_CORE_PUBLIC_KEYS", &core_public_key)
    .env("PERIPHERY_SSL_ENABLED", "true")
    .env("PERIPHERY_SSL_CERT_FILE", &periphery_pki.server_cert_path)
    .env("PERIPHERY_SSL_KEY_FILE", &periphery_pki.server_key_path)
    .env("PERIPHERY_STATS_POLLING_RATE", "1-hr")
    .env("PERIPHERY_CONTAINER_STATS_POLLING_RATE", "1-hr")
    .stdout(Stdio::from(periphery_log_file))
    .stderr(Stdio::from(periphery_err_file))
    .spawn()
    .expect("spawn periphery process");
  harness.periphery_child = Some(periphery_child);

  let core_root = temp_root.join("core-root");
  std::fs::create_dir_all(&core_root).unwrap();
  let core_port = pick_unused_port();
  let db_name = format!("komodo_e2e_{suffix}");

  let key_k0_b64 = BASE64.encode(&[0x11u8; 32]);
  let key_k1_b64 = BASE64.encode(&[0x22u8; 32]);
  let admin_password = "KomodoE2eAdminPassword#2026!";

  let core_log_path = temp_root.join("core.log");
  let core_log_file = std::fs::File::create(&core_log_path).unwrap();
  let core_err_file = core_log_file.try_clone().unwrap();

  let core_child = Command::new(&core_bin)
    .env("KOMODO_PORT", core_port.to_string())
    .env("KOMODO_BIND_IP", "127.0.0.1")
    .env("KOMODO_HOST", format!("http://127.0.0.1:{core_port}"))
    .env(
      "KOMODO_PRIVATE_KEY",
      format!("file:{}", core_noise_key_path.display()),
    )
    .env("KOMODO_DATABASE_ADDRESS", &mongo_addr)
    .env("KOMODO_DATABASE_DB_NAME", &db_name)
    .env("KOMODO_LOCAL_AUTH", "true")
    .env("KOMODO_ENABLE_NEW_USERS", "true")
    .env("KOMODO_DISABLE_INIT_RESOURCES", "true")
    .env("KOMODO_INIT_ADMIN_USERNAME", "admin")
    .env("KOMODO_INIT_ADMIN_PASSWORD", admin_password)
    .env(
      "KOMODO_JWT_SECRET",
      "komodo-e2e-distinct-jwt-secret-never-reused-for-aead",
    )
    .env("KOMODO_SECRET_PROVIDER_ALLOW_LOOPBACK", "true")
    .env(
      "KOMODO_SECRET_PROVIDER_ENCRYPTION_KEY",
      format!("k1:{key_k1_b64}"),
    )
    .env(
      "KOMODO_SECRET_PROVIDER_PREVIOUS_ENCRYPTION_KEYS",
      format!("k0:{key_k0_b64}"),
    )
    .env("KOMODO_REPO_DIRECTORY", core_root.join("repos"))
    .env("KOMODO_SYNC_DIRECTORY", core_root.join("syncs"))
    .env("KOMODO_ACTION_DIRECTORY", core_root.join("actions"))
    .env("KOMODO_MONITORING_INTERVAL", "1-hr")
    .env("KOMODO_RESOURCE_POLL_INTERVAL", "1-hr")
    .stdout(Stdio::from(core_log_file))
    .stderr(Stdio::from(core_err_file))
    .spawn()
    .expect("spawn core process");
  harness.core_child = Some(core_child);

  let core_url = format!("http://127.0.0.1:{core_port}");
  let proc_deadline = Instant::now() + Duration::from_secs(30);
  loop {
    let core_ok = http
      .get(format!("{core_url}/version"))
      .send()
      .await
      .map(|r| r.status().is_success())
      .unwrap_or(false);
    let periph_ok = TcpStream::connect(("127.0.0.1", periphery_port))
      .await
      .is_ok();
    if core_ok && periph_ok {
      break;
    }
    if Instant::now() >= proc_deadline {
      let c_logs =
        std::fs::read_to_string(&core_log_path).unwrap_or_default();
      let p_logs = std::fs::read_to_string(&periphery_log_path)
        .unwrap_or_default();
      panic!(
        "Core or Periphery did not become ready in time!\n--- CORE LOG ---\n{c_logs}\n--- PERIPHERY LOG ---\n{p_logs}"
      );
    }
    tokio::time::sleep(Duration::from_millis(250)).await;
  }

  // 5. Authenticate Admin & Non-Admin Developer Users via Real HTTP Auth Endpoints
  let (_admin_id, admin_client) = create_local_user_api_client(
    &http,
    &core_url,
    "admin",
    admin_password,
    false,
    None,
  )
  .await;

  let (dev_user_id, dev_client) = create_local_user_api_client(
    &http,
    &core_url,
    "developer",
    "KomodoDevUserPass#2026!",
    true,
    Some(&admin_client),
  )
  .await;

  // Non-admin must be denied on admin Secret Provider management endpoints
  assert!(
    dev_client
      .read(ListSecretProviders::default())
      .await
      .is_err(),
    "non-admin must not list full SecretProviders"
  );

  // 6. Create Infisical SecretProvider, Verify Read Sanitization, 3-Stage Test, Target-Change Guard & Keyring Rotation
  let default_selector = InfisicalSecretSelector {
    project_id: inf_project_id.clone(),
    environment: String::from("prod"),
    secret_path: String::from("/app"),
    include_imports: false,
    expand_references: false,
  };
  let infisical_config = InfisicalProviderConfig {
    endpoint: infisical_https_endpoint.clone(),
    client_id: reader_client_id.clone(),
    organization_slug: org_slug.clone(),
    ca_cert_pem: infisical_pki.ca_cert_pem.clone(),
    allow_private_ips: false,
    default_selector: Some(default_selector.clone()),
  };

  let created_provider = admin_client
    .write(CreateSecretProvider {
      slug: String::from("infisical-prod"),
      name: String::from("Infisical Production"),
      description: String::from("Live E2E Infisical provider"),
      enabled: Some(true),
      config: SecretProviderConfig::Infisical(
        infisical_config.clone(),
      ),
      credential: Some(SecretProviderCredentialInput::Infisical {
        client_secret: reader_client_secret.clone(),
      }),
    })
    .await
    .expect("CreateSecretProvider failed");

  assert_eq!(created_provider.slug, "infisical-prod");
  assert!(created_provider.has_credentials);
  assert_eq!(created_provider.revision, 1);
  assert_eq!(created_provider.credential_revision, 1);

  let fetched_provider = admin_client
    .read(GetSecretProvider {
      id_or_slug: String::from("infisical-prod"),
    })
    .await
    .unwrap();
  assert_no_canaries(
    "GetSecretProvider",
    &serde_json::to_string(&fetched_provider).unwrap(),
    &all_canaries,
  );

  // 3-stage connection test using stored credentials (`CredentialUpdate::Keep`)
  let conn_test = admin_client
    .write(TestSecretProviderConnection {
      id: Some(created_provider.id.clone()),
      config: None,
      credential_update: CredentialUpdate::Keep,
      selector: Some(SecretSelector::Infisical(
        default_selector.clone(),
      )),
      sample_key: Some(String::from("DATABASE_URL")),
    })
    .await
    .expect("TestSecretProviderConnection failed");
  assert!(conn_test.ready_for_deploy, "conn_test: {conn_test:?}");
  assert_eq!(
    conn_test.authentication.status,
    SecretCheckStageStatus::Passed
  );
  assert_eq!(
    conn_test.metadata_access.status,
    SecretCheckStageStatus::Passed
  );
  assert_eq!(
    conn_test.value_read_access.status,
    SecretCheckStageStatus::Passed
  );
  assert_no_canaries(
    "TestSecretProviderConnection",
    &serde_json::to_string(&conn_test).unwrap(),
    &all_canaries,
  );

  // Changing endpoint or `allow_private_ips` with `CredentialUpdate::Keep` MUST fail closed!
  let mut altered_endpoint_cfg = infisical_config.clone();
  altered_endpoint_cfg.endpoint =
    String::from("https://evil.example.com");
  assert!(
    admin_client
      .write(UpdateSecretProvider {
        id: created_provider.id.clone(),
        expected_revision: created_provider.revision,
        name: None,
        description: None,
        enabled: None,
        config: Some(SecretProviderConfig::Infisical(
          altered_endpoint_cfg.clone()
        )),
        credential_update: CredentialUpdate::Keep,
      })
      .await
      .is_err(),
    "Changing endpoint with CredentialUpdate::Keep must be rejected"
  );
  assert!(
    admin_client
      .write(TestSecretProviderConnection {
        id: Some(created_provider.id.clone()),
        config: Some(SecretProviderConfig::Infisical(
          altered_endpoint_cfg
        )),
        credential_update: CredentialUpdate::Keep,
        selector: None,
        sample_key: None,
      })
      .await
      .is_err(),
    "Testing altered endpoint with stored credentials must be rejected"
  );

  // Stale `expected_revision` MUST fail closed (409 Conflict)
  assert!(
    admin_client
      .write(UpdateSecretProvider {
        id: created_provider.id.clone(),
        expected_revision: 999,
        name: Some(String::from("Should Fail")),
        description: None,
        enabled: None,
        config: None,
        credential_update: CredentialUpdate::Keep,
      })
      .await
      .is_err(),
    "Stale expected_revision must be rejected"
  );

  // Keyring rotation endpoint check
  let rot_res = admin_client
    .write(RotateSecretProviderEncryptionKeys {})
    .await
    .expect("RotateSecretProviderEncryptionKeys failed");
  assert_eq!(rot_res.active_key_id, "k1");
  assert_eq!(rot_res.rotated_count, 0);

  // 7. Create Verified Confidential Server, Builder & Komodo Variable
  let periphery_https_addr =
    format!("https://127.0.0.1:{periphery_port}");
  let server = admin_client
    .write(CreateServer {
      name: String::from("e2e-confidential-server"),
      config: PartialServerConfig {
        address: Some(periphery_https_addr),
        enabled: Some(true),
        insecure_tls: Some(false),
        ca_cert_pem: Some(periphery_pki.ca_cert_pem.clone()),
        ..Default::default()
      },
      public_key: Some(periphery_public_key),
    })
    .await
    .expect("CreateServer failed");

  // Wait for verified WSS + Noise connection (`ServerState::Ok`)
  let srv_deadline = Instant::now() + Duration::from_secs(20);
  loop {
    let st = admin_client
      .read(GetServerState {
        server: server.id.clone(),
      })
      .await
      .unwrap();
    if st.status == ServerState::Ok {
      break;
    }
    if Instant::now() >= srv_deadline {
      let c_logs =
        std::fs::read_to_string(&core_log_path).unwrap_or_default();
      let p_logs = std::fs::read_to_string(&periphery_log_path)
        .unwrap_or_default();
      panic!(
        "Verified Server did not reach ServerState::Ok (got {:?})\n--- CORE ---\n{c_logs}\n--- PERIPHERY ---\n{p_logs}",
        st.status
      );
    }
    tokio::time::sleep(Duration::from_millis(250)).await;
  }

  let builder = admin_client
    .write(CreateBuilder {
      name: String::from("e2e-server-builder"),
      config: PartialBuilderConfig::Server(
        _PartialServerBuilderConfig {
          server_ids: Some(vec![server.id.clone()]),
        },
      ),
    })
    .await
    .expect("CreateBuilder failed");

  admin_client
    .write(CreateVariable {
      name: String::from("DB_USER"),
      value: String::from("komodo_app"),
      description: String::from("Non-secret DB username"),
      is_secret: false,
    })
    .await
    .expect("CreateVariable failed");

  // 8. Negative Security Gates: Insecure TLS Transport & `compose_cmd_wrapper` Conflict
  let compose_yaml = r#"services:
  api:
    image: debian:bookworm-slim
    command: ["sh", "-c", "echo boot_leak_test=\"$$API_ONLY_SECRET\" && sleep 300"]
    environment:
      DATABASE_URL: ${DATABASE_URL}
      MIXED_URL: ${MIXED_URL}
      SPECIAL_LITERAL: ${SPECIAL_LITERAL}
      DOT_KEY_SECRET: ${DOT_KEY_SECRET}
  worker:
    image: debian:bookworm-slim
    command: ["sleep", "300"]
    environment:
      DATABASE_URL: ${DATABASE_URL}
"#;

  // Create grant first for admin / ResourceAuthorized on AllStacks, AllDeployments, AllBuilds
  //Wait: test default-deny BEFORE creating grant first!
  let stack = admin_client
    .write(CreateStack {
      name: String::from("e2e-secret-stack"),
      config: PartialStackConfig {
        server_id: Some(server.id.clone()),
        project_name: Some(harness.compose_project.clone()),
        auto_pull: Some(false),
        file_contents: Some(String::from(compose_yaml)),
        environment: Some(String::from(
          "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]\n\
           MIXED_URL=postgres://[[DB_USER]]:[[vault.infisical-prod.DB_PASSWORD]]@db:5432/app?dup=[[vault.infisical-prod.DB_PASSWORD]]\n\
           SPECIAL_LITERAL=[[vault.infisical-prod.SPECIAL_LITERAL]]\n\
           DOT_KEY_SECRET=[[vault.infisical-prod.service.jwt.secret]]",
        )),
        secret_bindings: Some(vec![
          StackSecretBinding {
            source: String::from("infisical-prod"),
            remote_key: String::from("API_ONLY_SECRET"),
            target: StackSecretTarget::ServiceEnv {
              variable: String::from("API_ONLY_SECRET"),
              services: vec![String::from("api")],
            },
            format: SecretValueFormat::Text,
            json_field: String::new(),
            required: true,
            allow_empty: false,
            conflict_policy: SecretConflictPolicy::Fail,
          },
          StackSecretBinding {
            source: String::from("infisical-prod"),
            remote_key: String::from("FILE_CERT_PEM"),
            target: StackSecretTarget::ComposeFileSecret {
              secret_name: String::from("api_file_secret"),
              services: vec![String::from("api")],
              target_path: String::new(),
              uid: String::new(),
              gid: String::new(),
              mode: Some(0o400),
            },
            format: SecretValueFormat::Text,
            json_field: String::new(),
            required: true,
            allow_empty: false,
            conflict_policy: SecretConflictPolicy::Fail,
          },
        ]),
        ..Default::default()
      },
    })
    .await
    .expect("CreateStack failed");

  // Grant developer Komodo Write+Execute+Inspect+Logs on `stack`, WITHOUT a `SecretProviderGrant` yet
  let mut dev_specifics = IndexSet::new();
  dev_specifics.insert(SpecificPermission::Inspect);
  dev_specifics.insert(SpecificPermission::Logs);
  admin_client
    .write(UpdatePermissionOnTarget {
      user_target: UserTarget::User(dev_user_id.clone()),
      resource_target: ResourceTarget::Stack(stack.id.clone()),
      permission: PermissionLevelAndSpecifics {
        level: PermissionLevel::Write,
        specific: dev_specifics,
      },
    })
    .await
    .unwrap();

  // Default-deny check: BEFORE `SecretProviderGrant` exists, both `ListExternalSecretKeys` and `DeployStack` MUST fail closed!
  assert!(
    dev_client
      .read(ListExternalSecretKeys {
        target: ResourceTarget::Stack(stack.id.clone()),
        provider: String::from("infisical-prod"),
        selector: None,
      })
      .await
      .is_err(),
    "Default-deny must block ListExternalSecretKeys without SecretProviderGrant"
  );

  let denied_deploy = execute_and_wait(
    &dev_client,
    DeployStack {
      stack: stack.id.clone(),
      services: Vec::new(),
      stop_time: None,
    },
  )
  .await
  .unwrap();
  assert!(
    !denied_deploy.success,
    "Default-deny must block DeployStack without SecretProviderGrant: {}",
    format_update_logs(&denied_deploy)
  );

  // Now create scoped `SecretProviderGrant`
  let all_usages: IndexSet<SecretUsage> = [
    SecretUsage::MetadataList,
    SecretUsage::Preview,
    SecretUsage::StackEnv,
    SecretUsage::StackComposeFileSecret,
    SecretUsage::DeploymentEnv,
    SecretUsage::BuildSecret,
  ]
  .into_iter()
  .collect();

  admin_client
    .write(CreateSecretProviderGrant {
      provider_id: created_provider.id.clone(),
      name: String::from("e2e-prod-app-grant"),
      description: String::from("Scoped grant for /app in prod"),
      enabled: Some(true),
      subject: SecretGrantSubject::ResourceAuthorized,
      allowed_resources: vec![
        SecretResourceScope::AllStacks,
        SecretResourceScope::AllDeployments,
        SecretResourceScope::AllBuilds,
      ],
      selector_scope: SecretSelectorScope::Infisical(
        InfisicalSelectorScope {
          project_ids: vec![inf_project_id.clone()],
          environments: vec![String::from("prod")],
          path_prefixes: vec![String::from("/app")],
          allow_imports: false,
          allow_expand_references: false,
        },
      ),
      allowed_usages: all_usages,
    })
    .await
    .expect("CreateSecretProviderGrant failed");

  // Verify non-admin picker & preview endpoints return metadata ONLY (zero secret values!)
  let avail = dev_client
    .read(ListAvailableSecretProviders {
      target: Some(ResourceTarget::Stack(stack.id.clone())),
    })
    .await
    .unwrap();
  assert_eq!(avail.len(), 1);
  assert_eq!(avail[0].slug, "infisical-prod");
  assert!(avail[0].granted);
  assert_no_canaries(
    "ListAvailableSecretProviders",
    &serde_json::to_string(&avail).unwrap(),
    &all_canaries,
  );

  let listed_keys = dev_client
    .read(ListExternalSecretKeys {
      target: ResourceTarget::Stack(stack.id.clone()),
      provider: String::from("infisical-prod"),
      selector: None,
    })
    .await
    .expect("ListExternalSecretKeys should succeed after grant");
  assert!(
    v4_metadata_fetches.load(Ordering::SeqCst) >= 1,
    "Metadata listing / connection test must use viewSecretValue=false metadata query"
  );
  assert!(listed_keys.iter().any(|k| k.key == "DATABASE_URL"));
  assert!(listed_keys.iter().any(|k| k.key == "service.jwt.secret"));
  assert_no_canaries(
    "ListExternalSecretKeys",
    &serde_json::to_string(&listed_keys).unwrap(),
    &all_canaries,
  );

  let preview = dev_client
    .read(PreviewResourceSecrets {
      target: ResourceTarget::Stack(stack.id.clone()),
    })
    .await
    .expect("PreviewResourceSecrets should succeed");
  assert!(
    preview.valid && preview.errors.is_empty(),
    "preview: {preview:?}"
  );
  assert_no_canaries(
    "PreviewResourceSecrets",
    &serde_json::to_string(&preview).unwrap(),
    &all_canaries,
  );

  // Verify usages tracking & DeleteSecretProvider rejection while referenced
  let usages = admin_client
    .read(ListSecretProviderUsages {
      provider_id: created_provider.id.clone(),
    })
    .await
    .unwrap();
  assert!(
    usages.iter().any(|u| u.resource_name == "e2e-secret-stack")
  );
  assert!(
    admin_client
      .write(DeleteSecretProvider {
        id: created_provider.id.clone(),
      })
      .await
      .is_err(),
    "DeleteSecretProvider must fail while referenced by Stack"
  );

  // Negative check #1: Insecure TLS Server MUST be rejected before sending secrets to Periphery
  admin_client
    .write(UpdateServer {
      id: server.id.clone(),
      config: PartialServerConfig {
        insecure_tls: Some(true),
        ..Default::default()
      },
    })
    .await
    .unwrap();
  let insecure_srv_deadline =
    Instant::now() + Duration::from_secs(15);
  loop {
    let st = admin_client
      .read(GetServerState {
        server: server.id.clone(),
      })
      .await
      .unwrap();
    if st.status == ServerState::Ok {
      break;
    }
    assert!(
      Instant::now() < insecure_srv_deadline,
      "Server did not reconnect after toggling insecure_tls=true: {:?}",
      st.status
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
  }

  let insecure_stack = admin_client
    .write(CreateStack {
      name: String::from("e2e-insecure-tls-stack"),
      config: PartialStackConfig {
        server_id: Some(server.id.clone()),
        auto_pull: Some(false),
        file_contents: Some(String::from(
          "services:\n  s:\n    image: debian:bookworm-slim\n    command: [\"sleep\", \"10\"]\n",
        )),
        environment: Some(String::from(
          "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
        )),
        ..Default::default()
      },
    })
    .await
    .unwrap();
  let insecure_upd = execute_and_wait(
    &admin_client,
    DeployStack {
      stack: insecure_stack.id.clone(),
      services: Vec::new(),
      stop_time: None,
    },
  )
  .await
  .unwrap();
  assert!(
    !insecure_upd.success
      && format_update_logs(&insecure_upd).contains("insecure_tls"),
    "Insecure TLS server must be rejected: {}",
    format_update_logs(&insecure_upd)
  );
  admin_client
    .write(DeleteStack {
      id: insecure_stack.id,
    })
    .await
    .unwrap();

  admin_client
    .write(UpdateServer {
      id: server.id.clone(),
      config: PartialServerConfig {
        insecure_tls: Some(false),
        ca_cert_pem: Some(periphery_pki.ca_cert_pem.clone()),
        ..Default::default()
      },
    })
    .await
    .unwrap();
  let restore_srv_deadline = Instant::now() + Duration::from_secs(15);
  loop {
    let st = admin_client
      .read(GetServerState {
        server: server.id.clone(),
      })
      .await
      .unwrap();
    if st.status == ServerState::Ok {
      break;
    }
    assert!(
      Instant::now() < restore_srv_deadline,
      "Server did not reconnect after restoring insecure_tls=false: {:?}",
      st.status
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
  }

  // Negative check #2: `compose_cmd_wrapper` + external secrets MUST fail closed with explicit conflict
  let wrapper_stack = admin_client
    .write(CreateStack {
      name: String::from("e2e-wrapper-conflict-stack"),
      config: PartialStackConfig {
        server_id: Some(server.id.clone()),
        auto_pull: Some(false),
        compose_cmd_wrapper: Some(String::from(
          "echo [[COMPOSE_COMMAND]]",
        )),
        file_contents: Some(String::from(
          "services:\n  s:\n    image: debian:bookworm-slim\n    command: [\"sleep\", \"10\"]\n",
        )),
        environment: Some(String::from(
          "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
        )),
        ..Default::default()
      },
    })
    .await
    .unwrap();
  let wrapper_upd = execute_and_wait(
    &admin_client,
    DeployStack {
      stack: wrapper_stack.id.clone(),
      services: Vec::new(),
      stop_time: None,
    },
  )
  .await
  .unwrap();
  assert!(
    !wrapper_upd.success
      && format_update_logs(&wrapper_upd)
        .contains("compose_cmd_wrapper"),
    "compose_cmd_wrapper conflict must fail closed: {}",
    format_update_logs(&wrapper_upd)
  );
  admin_client
    .write(DeleteStack {
      id: wrapper_stack.id,
    })
    .await
    .unwrap();

  // 9. Live Stack Deploy on Docker Compose v5.5.0: Verify Single-Batch Deduplication & Byte-for-Byte Container Values
  let val_fetches_before_deploy =
    v4_value_fetches.load(Ordering::SeqCst);
  let deploy_upd = execute_and_wait(
    &dev_client,
    DeployStack {
      stack: stack.id.clone(),
      services: Vec::new(),
      stop_time: None,
    },
  )
  .await
  .unwrap();
  assert!(
    deploy_upd.success,
    "DeployStack failed:\n{}",
    format_update_logs(&deploy_upd)
  );
  assert_eq!(
    v4_value_fetches.load(Ordering::SeqCst)
      - val_fetches_before_deploy,
    1,
    "All 6 external secret references/bindings (including duplicate DB_PASSWORD) must resolve in exactly 1 Infisical v4 request!"
  );

  // Locate running `api` and `worker` container IDs
  let api_cid = run_docker(&[
    "compose",
    "-p",
    &harness.compose_project,
    "ps",
    "-q",
    "api",
  ]);
  let worker_cid = run_docker(&[
    "compose",
    "-p",
    &harness.compose_project,
    "ps",
    "-q",
    "worker",
  ]);
  assert!(!api_cid.is_empty() && !worker_cid.is_empty());

  let exec_printenv = |cid: &str, var: &str| -> Option<String> {
    let out = Command::new("docker")
      .args(["exec", cid, "printenv", var])
      .output()
      .unwrap();
    if out.status.success() {
      let mut s = String::from_utf8(out.stdout).unwrap();
      if s.ends_with('\n') {
        s.pop();
      }
      Some(s)
    } else {
      None
    }
  };

  // Byte-for-byte literal verification inside `api` container
  assert_eq!(
    exec_printenv(&api_cid, "DATABASE_URL").as_deref(),
    Some(contract_database_url),
    "Pre-encoded DATABASE_URL must NOT be double-encoded!"
  );
  assert_eq!(
    exec_printenv(&api_cid, "MIXED_URL").as_deref(),
    Some(
      "postgres://komodo_app:p@ss#word/123@db:5432/app?dup=p@ss#word/123"
    ),
    "Mixed Komodo variable + duplicate vault ref must substitute literally without URL-encoding!"
  );
  assert_eq!(
    exec_printenv(&api_cid, "SPECIAL_LITERAL").as_deref(),
    Some(contract_special_literal),
    "Special literal ($DOLLAR, ${{UNSET_SHELL}}, [[DB_USER]], quotes, backslashes, #hash, spaces) must survive byte-for-byte!"
  );
  assert_eq!(
    exec_printenv(&api_cid, "DOT_KEY_SECRET").as_deref(),
    Some(canary_dot_key.as_str())
  );
  assert_eq!(
    exec_printenv(&api_cid, "API_ONLY_SECRET").as_deref(),
    Some(canary_svc_sec.as_str())
  );

  // Verify `worker` isolation: `worker` has `DATABASE_URL`, but does NOT have `API_ONLY_SECRET` or `/run/secrets/api_file_secret`!
  assert_eq!(
    exec_printenv(&worker_cid, "DATABASE_URL").as_deref(),
    Some(contract_database_url)
  );
  assert_eq!(
    exec_printenv(&worker_cid, "API_ONLY_SECRET"),
    None,
    "ServiceEnv scoped to 'api' must NOT leak into 'worker'!"
  );
  assert!(
    !Command::new("docker")
      .args([
        "exec",
        &worker_cid,
        "test",
        "-e",
        "/run/secrets/api_file_secret"
      ])
      .status()
      .unwrap()
      .success(),
    "ComposeFileSecret scoped to 'api' must NOT be mounted in 'worker'!"
  );

  // Verify `/run/secrets/api_file_secret` bytes & `0400` permissions inside `api`
  let mounted_bytes = Command::new("docker")
    .args(["exec", &api_cid, "cat", "/run/secrets/api_file_secret"])
    .output()
    .unwrap();
  assert!(mounted_bytes.status.success());
  assert_eq!(
    String::from_utf8(mounted_bytes.stdout).unwrap(),
    canary_file_sec
  );
  let mounted_mode = run_docker(&[
    "exec",
    &api_cid,
    "stat",
    "-c",
    "%a",
    "/run/secrets/api_file_secret",
  ]);
  assert_eq!(mounted_mode, "400");

  // Verify host `.env` file permissions on disk are `0600`
  let host_env_path = periphery_root
    .join("stacks")
    .join("e2e-secret-stack")
    .join(".env");
  let env_mode = std::fs::metadata(&host_env_path)
    .unwrap()
    .permissions()
    .mode()
    & 0o777;
  assert_eq!(
    env_mode, 0o600,
    "Host Stack .env file must have 0600 permissions, got {:o}",
    env_mode
  );
  assert!(
    periphery_root
      .join("secret-runtime")
      .join("stacks")
      .join(&harness.compose_project)
      .exists(),
    "Runtime secret generation directory must exist while Stack is deployed"
  );

  // 10. Verify Structured Redaction Across `GetStack`, `InspectStackContainer`, and `GetStackLog`
  let got_stack = dev_client
    .read(GetStack {
      stack: stack.id.clone(),
    })
    .await
    .unwrap();
  let got_stack_json = serde_json::to_string(&got_stack).unwrap();
  assert_no_canaries("GetStack", &got_stack_json, &all_canaries);

  let inspected_api = dev_client
    .read(InspectStackContainer {
      stack: stack.id.clone(),
      service: String::from("api"),
    })
    .await
    .unwrap();
  let inspected_api_json =
    serde_json::to_string(&inspected_api).unwrap();
  assert_no_canaries(
    "InspectStackContainer",
    &inspected_api_json,
    &all_canaries,
  );
  assert!(
    inspected_api_json.contains("API_ONLY_SECRET=<REDACTED>"),
    "InspectStackContainer must mask secret env keys as <REDACTED>: {inspected_api_json}"
  );

  // Wait briefly for `echo boot_leak_test=...` to flush to Docker container stdout, then verify `GetStackLog` masks it!
  tokio::time::sleep(Duration::from_millis(400)).await;
  let stack_log = dev_client
    .read(GetStackLog {
      stack: stack.id.clone(),
      services: vec![String::from("api")],
      tail: 50,
      timestamps: false,
    })
    .await
    .unwrap();
  let stack_log_json = serde_json::to_string(&stack_log).unwrap();
  assert_no_canaries("GetStackLog", &stack_log_json, &all_canaries);
  assert!(
    stack_log.stdout.contains("boot_leak_test=<REDACTED>"),
    "GetStackLog must redact runtime secret values printed to container stdout! Got: {}",
    stack_log.stdout
  );

  // 11. Verify `StopStack` -> `StartStack` -> `RestartStack` Lifecycle Survives WITHOUT Re-fetching Infisical
  let fetches_before_lifecycle =
    v4_value_fetches.load(Ordering::SeqCst);
  for step_name in ["StopStack", "StartStack", "RestartStack"] {
    let upd = match step_name {
      "StopStack" => execute_and_wait(
        &dev_client,
        StopStack {
          stack: stack.id.clone(),
          stop_time: Some(2),
          services: Vec::new(),
        },
      )
      .await
      .unwrap(),
      "StartStack" => execute_and_wait(
        &dev_client,
        StartStack {
          stack: stack.id.clone(),
          services: Vec::new(),
        },
      )
      .await
      .unwrap(),
      _ => execute_and_wait(
        &dev_client,
        RestartStack {
          stack: stack.id.clone(),
          services: Vec::new(),
        },
      )
      .await
      .unwrap(),
    };
    assert!(
      upd.success,
      "{step_name} failed:\n{}",
      format_update_logs(&upd)
    );
  }
  assert_eq!(
    v4_value_fetches.load(Ordering::SeqCst),
    fetches_before_lifecycle,
    "Stop/Start/Restart must reuse persisted runtime manifest without re-fetching Infisical!"
  );
  let mounted_after_restart = Command::new("docker")
    .args(["exec", &api_cid, "cat", "/run/secrets/api_file_secret"])
    .output()
    .unwrap();
  assert_eq!(
    String::from_utf8(mounted_after_restart.stdout).unwrap(),
    canary_file_sec
  );

  // 12. Verify `DeployStackIfChanged`: Unchanged No-Op vs Remote Secret Rotation Single-Fetch Redeploy
  let updates_before_noop = admin_client
    .read(ListUpdates {
      query: None,
      page: 0,
    })
    .await
    .unwrap();
  let last_upd_id_before_noop = updates_before_noop
    .updates
    .first()
    .map(|u| u.id.clone())
    .unwrap_or_default();
  let fetches_before_noop = v4_value_fetches.load(Ordering::SeqCst);

  dev_client
    .execute(DeployStackIfChanged {
      stack: stack.id.clone(),
      stop_time: None,
    })
    .await
    .unwrap();

  // Wait for background `DeployStackIfChanged` preflight check to query Infisical once
  let noop_deadline = Instant::now() + Duration::from_secs(15);
  while v4_value_fetches.load(Ordering::SeqCst) == fetches_before_noop
  {
    assert!(
      Instant::now() < noop_deadline,
      "Timed out waiting for no-op DeployStackIfChanged to check Infisical"
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
  }
  tokio::time::sleep(Duration::from_millis(400)).await;

  let updates_after_noop = admin_client
    .read(ListUpdates {
      query: None,
      page: 0,
    })
    .await
    .unwrap();
  assert_eq!(
    updates_after_noop
      .updates
      .first()
      .map(|u| u.id.as_str())
      .unwrap_or(""),
    last_upd_id_before_noop,
    "Unchanged DeployStackIfChanged must be a no-op without creating a deploy Update"
  );
  let api_cid_after_noop = run_docker(&[
    "compose",
    "-p",
    &harness.compose_project,
    "ps",
    "-q",
    "api",
  ]);
  assert_eq!(
    api_cid_after_noop, api_cid,
    "DeployStackIfChanged must not recreate containers when secrets & files are unchanged"
  );

  // Rotate `API_ONLY_SECRET` in Infisical
  http
    .patch(format!("{raw_inf_http}/api/v4/secrets/API_ONLY_SECRET"))
    .bearer_auth(&inf_admin_token)
    .json(&json!({
      "projectId": inf_project_id,
      "environment": "prod",
      "secretPath": "/app",
      "secretValue": canary_svc_sec_rotated,
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  let fetches_before_if_changed =
    v4_value_fetches.load(Ordering::SeqCst);
  dev_client
    .execute(DeployStackIfChanged {
      stack: stack.id.clone(),
      stop_time: Some(2),
    })
    .await
    .unwrap();

  // Wait for background `DeployStackIfChanged` to create its `FullDeploy` Update in MongoDB, then poll to completion
  let rotated_upd_id = {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
      let list = admin_client
        .read(ListUpdates {
          query: None,
          page: 0,
        })
        .await
        .unwrap();
      if let Some(first) = list.updates.first()
        && first.id != last_upd_id_before_noop
      {
        break first.id.clone();
      }
      assert!(
        Instant::now() < deadline,
        "Timed out waiting for DeployStackIfChanged to create FullDeploy Update after secret rotation"
      );
      tokio::time::sleep(Duration::from_millis(150)).await;
    }
  };
  let rotated_upd = dev_client
    .poll_update_until_complete(&rotated_upd_id)
    .await
    .unwrap();
  assert!(
    rotated_upd.success,
    "DeployStackIfChanged on secret rotation failed:\n{}",
    format_update_logs(&rotated_upd)
  );
  assert_eq!(
    v4_value_fetches.load(Ordering::SeqCst)
      - fetches_before_if_changed,
    1,
    "DeployStackIfChanged must detect secret rotation AND deploy using a single Infisical fetch!"
  );

  let new_api_cid = run_docker(&[
    "compose",
    "-p",
    &harness.compose_project,
    "ps",
    "-q",
    "api",
  ]);
  assert_eq!(
    exec_printenv(&new_api_cid, "API_ONLY_SECRET").as_deref(),
    Some(canary_svc_sec_rotated.as_str())
  );

  // 13. Preflight Failure Must Preserve Running Workload
  let latest_prov = admin_client
    .read(GetSecretProvider {
      id_or_slug: created_provider.id.clone(),
    })
    .await
    .unwrap();
  let disabled_prov = admin_client
    .write(UpdateSecretProvider {
      id: created_provider.id.clone(),
      expected_revision: latest_prov.revision,
      name: None,
      description: None,
      enabled: Some(false),
      config: None,
      credential_update: CredentialUpdate::Keep,
    })
    .await
    .unwrap();

  let failed_redeploy = execute_and_wait(
    &dev_client,
    DeployStack {
      stack: stack.id.clone(),
      services: Vec::new(),
      stop_time: None,
    },
  )
  .await
  .unwrap();
  assert!(
    !failed_redeploy.success,
    "DeployStack must fail closed when provider is disabled"
  );
  // Running `api` container must still be alive with its prior configuration!
  assert_eq!(
    exec_printenv(&new_api_cid, "API_ONLY_SECRET").as_deref(),
    Some(canary_svc_sec_rotated.as_str()),
    "Preflight failure must NOT tear down or corrupt the running Stack!"
  );

  // Re-enable provider
  admin_client
    .write(UpdateSecretProvider {
      id: created_provider.id.clone(),
      expected_revision: disabled_prov.revision,
      name: None,
      description: None,
      enabled: Some(true),
      config: None,
      credential_update: CredentialUpdate::Keep,
    })
    .await
    .unwrap();

  // Destroy Stack & verify `secret-runtime/stacks/<project_name>` directory is removed
  let destroy_stack_upd = execute_and_wait(
    &dev_client,
    DestroyStack {
      stack: stack.id.clone(),
      services: Vec::new(),
      remove_orphans: true,
      stop_time: Some(2),
    },
  )
  .await
  .unwrap();
  assert!(destroy_stack_upd.success);
  assert!(
    !periphery_root
      .join("secret-runtime")
      .join("stacks")
      .join(&harness.compose_project)
      .exists(),
    "DestroyStack must clean up secret-runtime/stacks/<project> runtime files"
  );

  // 14. Live Deployment E2E: Inline + Structured Container Env + Redaction
  let deployment = admin_client
    .write(CreateDeployment {
      name: harness.deployment_container.clone(),
      config: PartialDeploymentConfig {
        server_id: Some(server.id.clone()),
        image: Some(DeploymentImage::Image {
          image: String::from("debian:bookworm-slim"),
        }),
        network: Some(String::from("bridge")),
        command: Some(String::from(
          "sh -c 'echo deploy_log_leak=$DEPLOY_TOKEN && sleep 300'",
        )),
        environment: Some(String::from(
          "DEPLOY_INLINE=prefix:[[vault.infisical-prod.DB_PASSWORD]]:suffix",
        )),
        secret_bindings: Some(vec![DeploymentSecretBinding {
          source: String::from("infisical-prod"),
          remote_key: String::from("DEPLOY_TOKEN"),
          variable: String::from("DEPLOY_TOKEN"),
          json_field: String::new(),
          required: true,
          allow_empty: false,
          conflict_policy: SecretConflictPolicy::Fail,
        }]),
        ..Default::default()
      },
    })
    .await
    .expect("CreateDeployment failed");

  let deploy_cont_upd = execute_and_wait(
    &admin_client,
    Deploy {
      deployment: deployment.id.clone(),
      stop_signal: None,
      stop_time: Some(2),
    },
  )
  .await
  .unwrap();
  assert!(
    deploy_cont_upd.success,
    "Deployment failed:\n{}",
    format_update_logs(&deploy_cont_upd)
  );

  assert_eq!(
    exec_printenv(&harness.deployment_container, "DEPLOY_INLINE")
      .as_deref(),
    Some("prefix:p@ss#word/123:suffix")
  );
  assert_eq!(
    exec_printenv(&harness.deployment_container, "DEPLOY_TOKEN")
      .as_deref(),
    Some(canary_deploy_sec.as_str())
  );

  let inspected_dep = admin_client
    .read(InspectDeploymentContainer {
      deployment: deployment.id.clone(),
    })
    .await
    .unwrap();
  let inspected_dep_json =
    serde_json::to_string(&inspected_dep).unwrap();
  assert_no_canaries(
    "InspectDeploymentContainer",
    &inspected_dep_json,
    &all_canaries,
  );
  assert!(inspected_dep_json.contains("DEPLOY_TOKEN=<REDACTED>"));

  tokio::time::sleep(Duration::from_millis(400)).await;
  let dep_log = admin_client
    .read(GetDeploymentLog {
      deployment: deployment.id.clone(),
      tail: 50,
      timestamps: false,
    })
    .await
    .unwrap();
  assert_no_canaries(
    "GetDeploymentLog",
    &serde_json::to_string(&dep_log).unwrap(),
    &all_canaries,
  );
  assert!(dep_log.stdout.contains("deploy_log_leak=<REDACTED>"));

  let destroy_dep_upd = execute_and_wait(
    &admin_client,
    DestroyDeployment {
      deployment: deployment.id.clone(),
      signal: None,
      time: Some(2),
    },
  )
  .await
  .unwrap();
  assert!(destroy_dep_upd.success);

  // 15. Live BuildKit `--secret` E2E: Verify Secret Mount in `RUN`, Image Layer Non-Leakage & Temp Cleanup
  let expected_build_sec_sha256 =
    hex::encode(Sha256::digest(canary_build_sec.as_bytes()));
  let dockerfile = format!(
    "FROM debian:bookworm-slim\n\
     ARG KOMODO_SECRET_CACHE_NONCE\n\
     RUN --mount=type=secret,id=INLINE_BUILD_SEC --mount=type=secret,id=bound_build_sec \\\n\
         test -s /run/secrets/INLINE_BUILD_SEC && \\\n\
         actual_sha=$(sha256sum /run/secrets/bound_build_sec | cut -d' ' -f1) && \\\n\
         test \"$actual_sha\" = \"{expected_build_sec_sha256}\" && \\\n\
         echo \"verified:$KOMODO_SECRET_CACHE_NONCE\" > /verified.txt\n"
  );

  let build = admin_client
    .write(CreateBuild {
      name: harness.built_image_name.clone(),
      config: PartialBuildConfig {
        builder_id: Some(builder.id.clone()),
        dockerfile: Some(dockerfile),
        secret_cache_bust: Some(true),
        secret_args: Some(String::from(
          "INLINE_BUILD_SEC=[[vault.infisical-prod.DB_PASSWORD]]",
        )),
        secret_bindings: Some(vec![BuildSecretBinding {
          source: String::from("infisical-prod"),
          remote_key: String::from("BUILD_REGISTRY_TOKEN"),
          secret_id: String::from("bound_build_sec"),
          format: SecretValueFormat::Text,
          json_field: String::new(),
          required: true,
          allow_empty: false,
          conflict_policy: SecretConflictPolicy::Fail,
        }]),
        ..Default::default()
      },
    })
    .await
    .expect("CreateBuild failed");

  let build_upd = execute_and_wait(
    &admin_client,
    RunBuild {
      build: build.id.clone(),
    },
  )
  .await
  .unwrap();
  assert!(
    build_upd.success,
    "RunBuild failed:\n{}",
    format_update_logs(&build_upd)
  );

  let built_image_ref =
    format!("{}:latest", harness.built_image_name);
  let verified_marker = run_docker(&[
    "run",
    "--rm",
    &built_image_ref,
    "cat",
    "/verified.txt",
  ]);
  assert!(
    verified_marker.starts_with("verified:")
      && verified_marker.len() > 20,
    "Expected verified marker with cache bust nonce, got: {verified_marker}"
  );

  let image_inspect =
    run_docker(&["image", "inspect", &built_image_ref]);
  let image_history =
    run_docker(&["history", "--no-trunc", &built_image_ref]);
  assert_no_canaries(
    "docker image inspect",
    &image_inspect,
    &all_canaries,
  );
  assert_no_canaries("docker history", &image_history, &all_canaries);

  // Verify isolated build secret temp directory was completely cleaned up
  let builds_secret_root =
    periphery_root.join("secret-runtime").join("builds");
  if builds_secret_root.exists() {
    let leftover: Vec<_> =
      std::fs::read_dir(&builds_secret_root).unwrap().collect();
    assert!(
      leftover.is_empty(),
      "Isolated BuildKit secret directory must be empty after RunBuild, found: {leftover:?}"
    );
  }

  // 16. Comprehensive High-Entropy Canary Leak Audit Across ResourceSync TOML, Updates API, Process Logs & Every MongoDB Collection
  let exported_toml = admin_client
    .read(ExportAllResourcesToToml {
      include_resources: true,
      tags: Vec::new(),
      include_variables: true,
      include_user_groups: true,
      existing: None,
    })
    .await
    .unwrap();
  assert!(
    exported_toml
      .toml
      .contains("[[vault.infisical-prod.DATABASE_URL]]"),
    "ExportAllResourcesToToml must preserve external vault references"
  );
  assert_no_canaries(
    "ExportAllResourcesToToml",
    &exported_toml.toml,
    &all_canaries,
  );

  let updates_list = admin_client
    .read(ListUpdates {
      query: None,
      page: 0,
    })
    .await
    .unwrap();
  assert!(!updates_list.updates.is_empty());
  for item in &updates_list.updates {
    let full_upd = admin_client
      .read(GetUpdate {
        id: item.id.clone(),
      })
      .await
      .unwrap();
    assert_no_canaries(
      &format!("GetUpdate({})", item.id),
      &serde_json::to_string(&full_upd).unwrap(),
      &all_canaries,
    );
  }

  let core_stdout_logs =
    std::fs::read_to_string(&core_log_path).unwrap_or_default();
  let periphery_stdout_logs =
    std::fs::read_to_string(&periphery_log_path).unwrap_or_default();
  assert_no_canaries(
    "Core process logs",
    &core_stdout_logs,
    &all_canaries,
  );
  assert_no_canaries(
    "Periphery process logs",
    &periphery_stdout_logs,
    &all_canaries,
  );

  // Scan EVERY collection in MongoDB directly!
  let mongo_client =
    mongodb::Client::with_uri_str(format!("mongodb://{mongo_addr}"))
      .await
      .unwrap();
  let mongo_db = mongo_client.database(&db_name);
  let collections = mongo_db.list_collection_names().await.unwrap();
  assert!(
    collections.contains(&String::from("SecretProvider"))
      && collections
        .contains(&String::from("StoredSecretProviderCredential"))
      && collections.contains(&String::from("SecretProviderGrant"))
      && collections
        .contains(&String::from("SecretDeploymentMetadata")),
    "Expected secret provider collections in MongoDB, found: {collections:?}"
  );

  for coll_name in &collections {
    let coll = mongo_db.collection::<Document>(coll_name);
    let docs: Vec<Document> = coll
      .find(Document::new())
      .await
      .unwrap()
      .try_collect()
      .await
      .unwrap();
    for doc in &docs {
      let serialized = serde_json::to_string(doc).unwrap();
      assert_no_canaries(
        &format!("MongoDB collection '{coll_name}'"),
        &serialized,
        &all_canaries,
      );
    }
  }

  // 17. Cleanly Delete Referencing Resources & Verify `DeleteSecretProvider` Now Succeeds
  admin_client
    .write(DeleteStack { id: stack.id })
    .await
    .unwrap();
  admin_client
    .write(DeleteDeployment { id: deployment.id })
    .await
    .unwrap();
  admin_client
    .write(DeleteBuild { id: build.id })
    .await
    .unwrap();

  let remaining_usages = admin_client
    .read(ListSecretProviderUsages {
      provider_id: created_provider.id.clone(),
    })
    .await
    .unwrap();
  assert!(remaining_usages.is_empty());

  admin_client
    .write(DeleteSecretProvider {
      id: created_provider.id.clone(),
    })
    .await
    .expect("DeleteSecretProvider should succeed once unreferenced");
}
