use std::{process::Command, time::Duration};

use komodo_client::entities::secret_provider::{
  InfisicalProviderConfig, InfisicalSecretSelector,
  SecretCheckStageStatus, SecretProvider, SecretProviderConfig,
  SecretProviderCredentialInput, SecretSelector, SecretValueFormat,
};
use reqwest::Client;
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

use super::{
  adapter::{AdapterResolveItemRequest, SecretProviderAdapter},
  http::set_test_allow_loopback,
  infisical::{
    INFISICAL_ADAPTER, invalidate_infisical_provider_cache,
    tests::{build_tls_acceptor, generate_test_pki},
  },
};

struct EphemeralInfisicalStack {
  network_name: String,
  pg_container: String,
  redis_container: String,
  infisical_container: String,
}

impl EphemeralInfisicalStack {
  fn cleanup(&self) {
    for c in [
      &self.infisical_container,
      &self.redis_container,
      &self.pg_container,
    ] {
      let _ = Command::new("docker").args(["rm", "-f", c]).output();
    }
    let _ = Command::new("docker")
      .args(["network", "rm", &self.network_name])
      .output();
  }
}

impl Drop for EphemeralInfisicalStack {
  fn drop(&mut self) {
    self.cleanup();
  }
}

fn run_docker(args: &[&str]) -> String {
  let out = Command::new("docker")
    .args(args)
    .output()
    .expect("failed to execute docker CLI");
  assert!(
    out.status.success(),
    "docker {:?} failed:\nstdout: {}\nstderr: {}",
    args,
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Live contract test against real `infisical/infisical:v0.162.19` + `postgres:18.1-alpine` + `valkey/valkey:9.0.1-alpine`
/// fronted by a real TLS 1.3 X.509 certificate signed by an isolated test Root CA.
#[tokio::test]
async fn live_infisical_v0_162_19_universal_auth_and_v4_secrets_contract()
 {
  set_test_allow_loopback(true);

  let suffix =
    uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
  let stack = EphemeralInfisicalStack {
    network_name: format!("komodo-inf-test-net-{suffix}"),
    pg_container: format!("komodo-inf-test-pg-{suffix}"),
    redis_container: format!("komodo-inf-test-valkey-{suffix}"),
    infisical_container: format!("komodo-inf-test-srv-{suffix}"),
  };

  run_docker(&["network", "create", &stack.network_name]);

  run_docker(&[
    "run",
    "-d",
    "--name",
    &stack.pg_container,
    "--network",
    &stack.network_name,
    "--network-alias",
    "infisical-db",
    "-e",
    "POSTGRES_USER=infisical",
    "-e",
    "POSTGRES_PASSWORD=infisical_test_pw",
    "-e",
    "POSTGRES_DB=infisical",
    "postgres:18.1-alpine",
  ]);

  run_docker(&[
    "run",
    "-d",
    "--name",
    &stack.redis_container,
    "--network",
    &stack.network_name,
    "--network-alias",
    "infisical-redis",
    "valkey/valkey:9.0.1-alpine",
  ]);

  // Wait for Postgres readiness inside container
  let pg_deadline =
    std::time::Instant::now() + Duration::from_secs(30);
  loop {
    let out = Command::new("docker")
      .args([
        "exec",
        &stack.pg_container,
        "pg_isready",
        "-U",
        "infisical",
      ])
      .output()
      .unwrap();
    if out.status.success() {
      break;
    }
    assert!(
      std::time::Instant::now() < pg_deadline,
      "Postgres container did not become ready in time"
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
  }

  run_docker(&[
    "run",
    "-d",
    "--name",
    &stack.infisical_container,
    "--network",
    &stack.network_name,
    "-p",
    "127.0.0.1::8080",
    "-e",
    "ENCRYPTION_KEY=6c1fe4e407b8911c104518103505b218",
    "-e",
    "AUTH_SECRET=55f9ce8f3e3b46b29a2e9a2a8f7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b",
    "-e",
    "DB_CONNECTION_URI=postgres://infisical:infisical_test_pw@infisical-db:5432/infisical",
    "-e",
    "REDIS_URL=redis://infisical-redis:6379",
    "-e",
    "TELEMETRY_ENABLED=false",
    // Infisical `/api/v1/admin/bootstrap` signs a legacy setup token without `exp` whose default
    // 90-day grace window from 2026-05-04 elapsed; Universal Auth tokens tested below carry explicit `exp`.
    "-e",
    "LEGACY_IDENTITY_ACCESS_TOKEN_EXPIRATION_ENFORCED_AT=2099-01-01T00:00:00.000Z",
    "infisical/infisical:v0.162.19",
  ]);

  let port_out =
    run_docker(&["port", &stack.infisical_container, "8080/tcp"]);
  let upstream_addr = port_out
    .lines()
    .next()
    .expect("missing mapped port for infisical")
    .trim()
    .to_string();

  // Front Infisical with a real TLS 1.3 X.509 acceptor signed by our test Root CA
  let pki = generate_test_pki();
  let acceptor = build_tls_acceptor(&pki);
  let tls_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let tls_port = tls_listener.local_addr().unwrap().port();
  let upstream_for_proxy = upstream_addr.clone();

  let proxy_handle = tokio::spawn(async move {
    while let Ok((client_tcp, _)) = tls_listener.accept().await {
      let acceptor = acceptor.clone();
      let upstream_target = upstream_for_proxy.clone();
      tokio::spawn(async move {
        let Ok(mut tls_stream) = acceptor.accept(client_tcp).await
        else {
          return;
        };
        let Ok(mut upstream_tcp) =
          TcpStream::connect(&upstream_target).await
        else {
          return;
        };
        let _ = tokio::io::copy_bidirectional(
          &mut tls_stream,
          &mut upstream_tcp,
        )
        .await;
      });
    }
  });

  let https_endpoint = format!("https://127.0.0.1:{tls_port}");
  let admin_http = Client::new();
  let raw_http_base = format!("http://{upstream_addr}");

  // Wait for Infisical `/api/status` health check
  let boot_deadline =
    std::time::Instant::now() + Duration::from_secs(90);
  loop {
    if let Ok(resp) = admin_http
      .get(format!("{raw_http_base}/api/status"))
      .send()
      .await
      && resp.status().is_success()
    {
      break;
    }
    if std::time::Instant::now() >= boot_deadline {
      let logs = Command::new("docker")
        .args(["logs", "--tail", "80", &stack.infisical_container])
        .output()
        .map(|o| {
          format!(
            "{}\n{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
          )
        })
        .unwrap_or_default();
      panic!("Infisical container did not become healthy:\n{logs}");
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
  }

  // 1. Bootstrap Infisical organization + admin token
  let bootstrap_resp: Value = admin_http
    .post(format!("{raw_http_base}/api/v1/admin/bootstrap"))
    .json(&json!({
      "email": "admin@komodo.test",
      "password": "KomodoStrongBootstrapPassword#2026!",
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
  let admin_token =
    bootstrap_resp["identity"]["credentials"]["token"]
      .as_str()
      .unwrap()
      .to_string();

  // 2. Create Project
  let proj_resp: Value = admin_http
    .post(format!("{raw_http_base}/api/v1/projects"))
    .bearer_auth(&admin_token)
    .json(&json!({
      "projectName": "komodo-secrets-prod"
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json()
    .await
    .unwrap();
  let project_id =
    proj_resp["project"]["id"].as_str().unwrap().to_string();

  // 3. Create folders `/shared` and `/app` in `prod` environment
  for folder_name in ["shared", "app"] {
    admin_http
      .post(format!("{raw_http_base}/api/v2/folders"))
      .bearer_auth(&admin_token)
      .json(&json!({
        "projectId": project_id,
        "environment": "prod",
        "name": folder_name,
        "path": "/"
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
  }

  // 4. Seed secrets in `/shared` and `/app` (including pre-encoded DATABASE_URL, special-char DB_PASSWORD, dot key, import override, and secret reference!)
  let secrets_to_seed = [
    (
      "/shared",
      "SHARED_API_KEY",
      "shared-val-987",
      "Imported from /shared",
    ),
    (
      "/shared",
      "OVERRIDE_KEY",
      "from-shared-should-be-overridden",
      "",
    ),
    (
      "/app",
      "OVERRIDE_KEY",
      "from-app-local-wins",
      "Local overrides import",
    ),
    (
      "/app",
      "DATABASE_URL",
      "postgres://user:p%40ss%23word%2F123@db:5432/app",
      "Recommended whole URL",
    ),
    (
      "/app",
      "DB_PASSWORD",
      "p@ss#word/123",
      "Raw special-char password",
    ),
    (
      "/app",
      "service.jwt.secret",
      "literal-[[EXISTING_VAR]]-${UNSET}-#hash",
      "Dot key with literal brackets",
    ),
    (
      "/app",
      "REF_EXPANDED_URL",
      "https://api.internal/${SHARED_API_KEY}/v1",
      "Infisical reference syntax",
    ),
  ];

  for (path, key, value, comment) in secrets_to_seed {
    let encoded_key = urlencoding::encode(key);
    admin_http
      .post(format!("{raw_http_base}/api/v4/secrets/{encoded_key}"))
      .bearer_auth(&admin_token)
      .json(&json!({
        "projectId": project_id,
        "environment": "prod",
        "secretPath": path,
        "secretValue": value,
        "secretComment": comment
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
  }

  // 5. Configure secret import in `/app` importing `/shared`
  admin_http
    .post(format!("{raw_http_base}/api/v2/secret-imports"))
    .bearer_auth(&admin_token)
    .json(&json!({
      "projectId": project_id,
      "environment": "prod",
      "path": "/app",
      "import": {
        "environment": "prod",
        "path": "/shared"
      }
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  // 6. Create Machine Identity #1 (Viewer / full ReadValue) + Universal Auth
  let create_ua_identity = |name: &str| {
    let admin_http = admin_http.clone();
    let raw_http_base = raw_http_base.clone();
    let admin_token = admin_token.clone();
    let org_id = org_id.clone();
    let name = name.to_string();
    async move {
      let id_resp: Value = admin_http
        .post(format!("{raw_http_base}/api/v1/identities"))
        .bearer_auth(&admin_token)
        .json(&json!({
          "name": name,
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

      let ua_resp: Value = admin_http
        .post(format!(
          "{raw_http_base}/api/v1/auth/universal-auth/identities/{identity_id}"
        ))
        .bearer_auth(&admin_token)
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
      let client_id = ua_resp["identityUniversalAuth"]["clientId"]
        .as_str()
        .unwrap()
        .to_string();

      let cs_resp: Value = admin_http
        .post(format!(
          "{raw_http_base}/api/v1/auth/universal-auth/identities/{identity_id}/client-secrets"
        ))
        .bearer_auth(&admin_token)
        .json(&json!({
          "description": "komodo-test",
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
      let client_secret =
        cs_resp["clientSecret"].as_str().unwrap().to_string();

      (identity_id, client_id, client_secret)
    }
  };

  let (reader_identity_id, reader_client_id, reader_client_secret) =
    create_ua_identity("komodo-reader-mi").await;

  admin_http
    .post(format!(
      "{raw_http_base}/api/v1/projects/{project_id}/memberships/identities/{reader_identity_id}"
    ))
    .bearer_auth(&admin_token)
    .json(&json!({
      "roles": [{ "role": "viewer", "isTemporary": false }]
    }))
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  // 7. Create Custom Role with `describeSecret` ONLY (no `readValue`) to test real Infisical `secretValueHidden: true`!
  let role_create_res = admin_http
    .post(format!("{raw_http_base}/api/v1/projects/{project_id}/roles"))
    .bearer_auth(&admin_token)
    .json(&json!({
      "slug": "describe-only",
      "name": "Describe Only",
      "description": "Can list metadata without reading secret values",
      "permissions": [
        { "subject": "secrets", "action": ["describeSecret"] },
        { "subject": "secret-folders", "action": ["read"] },
        { "subject": "secret-imports", "action": ["read"] }
      ]
    }))
    .send()
    .await
    .unwrap();

  let custom_role_created = role_create_res.status().is_success();

  // 8. Exercise `INFISICAL_ADAPTER` over REAL HTTPS TLS with custom `ca_cert_pem`!
  let provider = SecretProvider {
    id: "live-prov-1".to_string(),
    slug: "infisical-live".to_string(),
    name: "Infisical Live".to_string(),
    description: String::new(),
    enabled: true,
    revision: 1,
    credential_revision: 1,
    has_credentials: true,
    updated_at: 0,
    config: SecretProviderConfig::Infisical(
      InfisicalProviderConfig {
        endpoint: https_endpoint.clone(),
        client_id: reader_client_id,
        organization_slug: org_slug.clone(),
        ca_cert_pem: pki.ca_cert_pem.clone(),
        allow_private_ips: false,
        default_selector: Some(InfisicalSecretSelector {
          project_id: project_id.clone(),
          environment: "prod".to_string(),
          secret_path: "/app".to_string(),
          include_imports: true,
          expand_references: true,
        }),
      },
    ),
  };
  let cred = SecretProviderCredentialInput::Infisical {
    client_secret: reader_client_secret,
  };
  let cancel = CancellationToken::new();

  // 8a. 3-stage test_connection MUST pass all 3 stages!
  let diag = INFISICAL_ADAPTER
    .test_connection(
      &provider,
      &cred,
      None,
      Some("DATABASE_URL"),
      &cancel,
    )
    .await
    .unwrap();
  assert!(
    diag.ready_for_deploy,
    "expected ready_for_deploy=true, got: {diag:?}"
  );
  assert_eq!(
    diag.authentication.status,
    SecretCheckStageStatus::Passed
  );
  assert_eq!(
    diag.metadata_access.status,
    SecretCheckStageStatus::Passed
  );
  assert_eq!(
    diag.value_read_access.status,
    SecretCheckStageStatus::Passed
  );

  // 8b. Metadata listing (`viewSecretValue=false`) returns keys + comments + imported flags, zero values!
  let sel_with_imports =
    SecretSelector::Infisical(InfisicalSecretSelector {
      project_id: project_id.clone(),
      environment: "prod".to_string(),
      secret_path: "/app".to_string(),
      include_imports: true,
      expand_references: true,
    });
  let metadata = INFISICAL_ADAPTER
    .list_metadata(&provider, &cred, &sel_with_imports, &cancel)
    .await
    .unwrap();
  let meta_keys: Vec<&str> =
    metadata.iter().map(|m| m.key.as_str()).collect();
  assert!(meta_keys.contains(&"DATABASE_URL"));
  assert!(meta_keys.contains(&"DB_PASSWORD"));
  assert!(meta_keys.contains(&"SHARED_API_KEY"));
  assert!(meta_keys.contains(&"service.jwt.secret"));
  let shared_meta =
    metadata.iter().find(|m| m.key == "SHARED_API_KEY").unwrap();
  assert!(shared_meta.imported);
  assert_eq!(shared_meta.source_path, "/shared");

  // 8c. Batch resolution with imports + reference expansion enabled
  let reqs = vec![
    AdapterResolveItemRequest {
      remote_key: "DATABASE_URL".into(),
      format: SecretValueFormat::Text,
      json_field: String::new(),
      required: true,
      allow_empty: false,
    },
    AdapterResolveItemRequest {
      remote_key: "DB_PASSWORD".into(),
      format: SecretValueFormat::Text,
      json_field: String::new(),
      required: true,
      allow_empty: false,
    },
    AdapterResolveItemRequest {
      remote_key: "service.jwt.secret".into(),
      format: SecretValueFormat::Text,
      json_field: String::new(),
      required: true,
      allow_empty: false,
    },
    AdapterResolveItemRequest {
      remote_key: "SHARED_API_KEY".into(),
      format: SecretValueFormat::Text,
      json_field: String::new(),
      required: true,
      allow_empty: false,
    },
    AdapterResolveItemRequest {
      remote_key: "OVERRIDE_KEY".into(),
      format: SecretValueFormat::Text,
      json_field: String::new(),
      required: true,
      allow_empty: false,
    },
    AdapterResolveItemRequest {
      remote_key: "REF_EXPANDED_URL".into(),
      format: SecretValueFormat::Text,
      json_field: String::new(),
      required: true,
      allow_empty: false,
    },
  ];

  let resolved = INFISICAL_ADAPTER
    .resolve_batch(
      &provider,
      &cred,
      &sel_with_imports,
      &reqs,
      &cancel,
    )
    .await
    .unwrap();

  assert_eq!(
    resolved["DATABASE_URL"]
      .as_ref()
      .unwrap()
      .as_text()
      .unwrap(),
    "postgres://user:p%40ss%23word%2F123@db:5432/app"
  );
  assert_eq!(
    resolved["DB_PASSWORD"].as_ref().unwrap().as_text().unwrap(),
    "p@ss#word/123"
  );
  assert_eq!(
    resolved["service.jwt.secret"]
      .as_ref()
      .unwrap()
      .as_text()
      .unwrap(),
    "literal-[[EXISTING_VAR]]-${UNSET}-#hash"
  );
  assert_eq!(
    resolved["SHARED_API_KEY"]
      .as_ref()
      .unwrap()
      .as_text()
      .unwrap(),
    "shared-val-987"
  );
  assert_eq!(
    resolved["OVERRIDE_KEY"]
      .as_ref()
      .unwrap()
      .as_text()
      .unwrap(),
    "from-app-local-wins"
  );
  assert_eq!(
    resolved["REF_EXPANDED_URL"]
      .as_ref()
      .unwrap()
      .as_text()
      .unwrap(),
    "https://api.internal/shared-val-987/v1"
  );

  // 8d. With `include_imports=false` & `expand_references=false`, `SHARED_API_KEY` is absent and `REF_EXPANDED_URL` stays unexpanded!
  let sel_no_imports_no_expand =
    SecretSelector::Infisical(InfisicalSecretSelector {
      project_id: project_id.clone(),
      environment: "prod".to_string(),
      secret_path: "/app".to_string(),
      include_imports: false,
      expand_references: false,
    });
  let resolved_raw = INFISICAL_ADAPTER
    .resolve_batch(
      &provider,
      &cred,
      &sel_no_imports_no_expand,
      &[
        AdapterResolveItemRequest {
          remote_key: "SHARED_API_KEY".into(),
          format: SecretValueFormat::Text,
          json_field: String::new(),
          required: false,
          allow_empty: true,
        },
        AdapterResolveItemRequest {
          remote_key: "REF_EXPANDED_URL".into(),
          format: SecretValueFormat::Text,
          json_field: String::new(),
          required: true,
          allow_empty: false,
        },
      ],
      &cancel,
    )
    .await
    .unwrap();
  assert!(resolved_raw["SHARED_API_KEY"].is_none());
  assert_eq!(
    resolved_raw["REF_EXPANDED_URL"]
      .as_ref()
      .unwrap()
      .as_text()
      .unwrap(),
    "https://api.internal/${SHARED_API_KEY}/v1"
  );

  // 8e. Verify real DescribeSecret-only (or ungranted project) Machine Identity fails closed on value read!
  let (limited_id, limited_cid, limited_csec) =
    create_ua_identity("komodo-limited-mi").await;
  if custom_role_created {
    admin_http
      .post(format!(
        "{raw_http_base}/api/v1/projects/{project_id}/memberships/identities/{limited_id}"
      ))
      .bearer_auth(&admin_token)
      .json(&json!({
        "roles": [{ "role": "describe-only", "isTemporary": false }]
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
  } else {
    admin_http
      .post(format!(
        "{raw_http_base}/api/v1/projects/{project_id}/memberships/identities/{limited_id}"
      ))
      .bearer_auth(&admin_token)
      .json(&json!({
        "roles": [{ "role": "no-access", "isTemporary": false }]
      }))
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
  }

  let limited_provider = SecretProvider {
    id: "live-prov-limited".to_string(),
    slug: "infisical-limited".to_string(),
    name: "Infisical Limited".to_string(),
    description: String::new(),
    enabled: true,
    revision: 1,
    credential_revision: 1,
    has_credentials: true,
    updated_at: 0,
    config: SecretProviderConfig::Infisical(
      InfisicalProviderConfig {
        endpoint: https_endpoint,
        client_id: limited_cid,
        organization_slug: org_slug,
        ca_cert_pem: pki.ca_cert_pem.clone(),
        allow_private_ips: false,
        default_selector: Some(InfisicalSecretSelector {
          project_id: project_id.clone(),
          environment: "prod".to_string(),
          secret_path: "/app".to_string(),
          include_imports: false,
          expand_references: false,
        }),
      },
    ),
  };
  let limited_cred = SecretProviderCredentialInput::Infisical {
    client_secret: limited_csec,
  };

  let limited_diag = INFISICAL_ADAPTER
    .test_connection(
      &limited_provider,
      &limited_cred,
      None,
      None,
      &cancel,
    )
    .await
    .unwrap();
  assert!(!limited_diag.ready_for_deploy);
  assert_eq!(
    limited_diag.authentication.status,
    SecretCheckStageStatus::Passed
  );
  if custom_role_created {
    assert_eq!(
      limited_diag.metadata_access.status,
      SecretCheckStageStatus::Passed
    );
    assert_eq!(
      limited_diag.value_read_access.status,
      SecretCheckStageStatus::Failed
    );
  }

  // `resolve_batch` MUST fail closed and NEVER return "<hidden-by-infisical>"!
  assert!(
    INFISICAL_ADAPTER
      .resolve_batch(
        &limited_provider,
        &limited_cred,
        &sel_no_imports_no_expand,
        &reqs[..1],
        &cancel,
      )
      .await
      .is_err()
  );

  invalidate_infisical_provider_cache("live-prov-1");
  invalidate_infisical_provider_cache("live-prov-limited");
  proxy_handle.abort();
}
