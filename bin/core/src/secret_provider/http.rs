use std::{
  collections::HashMap,
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
  sync::{Arc, LazyLock, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Context, anyhow, bail};
use bytes::BytesMut;
use reqwest::{
  Certificate, Client, Method, Response, StatusCode, Url,
  dns::{Addrs, Name, Resolve, Resolving},
  header::HeaderMap,
  redirect,
};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::config::core_config;

/// Hard upper bound on any HTTP response body from an external secret provider (4 MiB).
pub const MAX_SECRET_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
/// Default TCP + TLS connect timeout per attempt.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Default per-attempt request timeout.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Default total deadline for a provider operation (including login + retries).
pub const DEFAULT_OPERATION_DEADLINE: Duration =
  Duration::from_secs(25);
/// Maximum concurrent outbound HTTP requests per provider endpoint.
pub const MAX_CONCURRENT_REQUESTS_PER_PROVIDER: usize = 4;

/// Normalize any IPv6-embedded IPv4 representation (IPv4-mapped `::ffff:a.b.c.d`,
/// IPv4-compatible, 6to4 `2002::/16`, Teredo `2001:0000::/32`, NAT64 `64:ff9b::/96` & `64:ff9b:1::/48`)
/// so embedded private/link-local/metadata IPv4 targets cannot bypass IPv4 SSRF checks.
fn extract_embedded_ipv4(v6: Ipv6Addr) -> Option<Ipv4Addr> {
  if let Some(mapped) = v6.to_ipv4_mapped() {
    return Some(mapped);
  }
  let segs = v6.segments();
  // Legacy IPv4-compatible `::a.b.c.d` (excluding `::` and `::1`)
  if segs[0..6] == [0, 0, 0, 0, 0, 0]
    && !(segs[6] == 0 && segs[7] <= 1)
  {
    let octets = v6.octets();
    return Some(Ipv4Addr::new(
      octets[12], octets[13], octets[14], octets[15],
    ));
  }
  // 6to4 `2002::/16` embeds IPv4 in bits 16..48
  if segs[0] == 0x2002 {
    return Some(Ipv4Addr::new(
      (segs[1] >> 8) as u8,
      (segs[1] & 0xff) as u8,
      (segs[2] >> 8) as u8,
      (segs[2] & 0xff) as u8,
    ));
  }
  // Teredo `2001:0000::/32` embeds obfuscated (~client_ipv4) in last 32 bits
  if segs[0] == 0x2001 && segs[1] == 0x0000 {
    let octets = v6.octets();
    return Some(Ipv4Addr::new(
      !octets[12],
      !octets[13],
      !octets[14],
      !octets[15],
    ));
  }
  // RFC 6052 / RFC 8215 NAT64 well-known prefixes `64:ff9b::/96` and `64:ff9b:1::/48`
  if segs[0] == 0x0064
    && segs[1] == 0xff9b
    && ((segs[2] == 0
      && segs[3] == 0
      && segs[4] == 0
      && segs[5] == 0)
      || segs[2] == 1)
  {
    let octets = v6.octets();
    return Some(Ipv4Addr::new(
      octets[12], octets[13], octets[14], octets[15],
    ));
  }
  None
}

fn validate_ipv4_addr(
  ip: Ipv4Addr,
  allow_private_ips: bool,
  allow_loopback: bool,
) -> anyhow::Result<()> {
  let octets = ip.octets();

  // `0.0.0.0/8` ("this network" / unspecified)
  if octets[0] == 0 {
    bail!(
      "Outbound secret provider IP {ip} is forbidden (unspecified/0.0.0.0/8)"
    );
  }
  // Broadcast `255.255.255.255` and reserved Class E `240.0.0.0/4`
  if ip.is_broadcast() || octets[0] >= 240 {
    bail!(
      "Outbound secret provider IP {ip} is forbidden (broadcast/reserved range)"
    );
  }
  // Multicast `224.0.0.0/4`
  if ip.is_multicast() {
    bail!(
      "Outbound secret provider IP {ip} is forbidden (multicast range)"
    );
  }
  // Link-local & cloud instance metadata `169.254.0.0/16` (AWS/GCP/Azure 169.254.169.254, ECS 169.254.170.2)
  if ip.is_link_local() {
    bail!(
      "Outbound secret provider IP {ip} is unconditionally forbidden (link-local / cloud metadata 169.254.0.0/16)"
    );
  }
  // Alibaba Cloud metadata `100.100.100.200` & IETF protocol assignments `192.0.0.0/24` (incl. Oracle metadata `192.0.0.192`)
  if octets == [100, 100, 100, 200]
    || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
  {
    bail!(
      "Outbound secret provider IP {ip} is unconditionally forbidden (cloud metadata / special protocol address)"
    );
  }
  // Documentation & benchmark ranges (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`, `198.18.0.0/15`)
  if ip.is_documentation()
    || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
  {
    bail!(
      "Outbound secret provider IP {ip} is forbidden (documentation/benchmark range)"
    );
  }
  // Loopback `127.0.0.0/8`
  if ip.is_loopback() {
    if !allow_loopback {
      bail!(
        "Outbound secret provider IP {ip} is loopback (127.0.0.0/8), which is disabled by policy"
      );
    }
    return Ok(());
  }
  // RFC 1918 private (`10/8`, `172.16/12`, `192.168/16`) + CGNAT / Tailscale (`100.64.0.0/10`)
  let is_cgnat = octets[0] == 100 && (octets[1] & 0b1100_0000) == 64;
  if ip.is_private() || is_cgnat {
    if !allow_private_ips {
      bail!(
        "Outbound secret provider IP {ip} is in a private/internal network range; enable 'allow_private_ips' on the provider to permit self-hosted internal endpoints"
      );
    }
    return Ok(());
  }

  Ok(())
}

/// Validate a resolved or literal IP address against Komodo's outbound secret provider SSRF policy.
pub fn validate_ip_addr(
  ip: IpAddr,
  allow_private_ips: bool,
  allow_loopback: bool,
) -> anyhow::Result<()> {
  match ip {
    IpAddr::V4(v4) => {
      validate_ipv4_addr(v4, allow_private_ips, allow_loopback)
    }
    IpAddr::V6(v6) => {
      if let Some(embedded_v4) = extract_embedded_ipv4(v6) {
        return validate_ipv4_addr(
          embedded_v4,
          allow_private_ips,
          allow_loopback,
        )
        .with_context(|| {
          format!("IPv6 address {v6} embeds forbidden IPv4 target")
        });
      }
      if v6.is_unspecified() {
        bail!(
          "Outbound secret provider IP {v6} is forbidden (unspecified ::)"
        );
      }
      if v6.is_multicast() {
        bail!(
          "Outbound secret provider IP {v6} is forbidden (multicast ff00::/8)"
        );
      }
      // Link-local `fe80::/10` (includes AWS EC2 IPv6 IMDS `fd00:ec2::254` check separately below!)
      if v6.is_unicast_link_local() {
        bail!(
          "Outbound secret provider IP {v6} is unconditionally forbidden (IPv6 link-local fe80::/10)"
        );
      }
      let segs = v6.segments();
      // AWS EC2 IPv6 Instance Metadata Service `fd00:ec2::254`
      if segs == [0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254] {
        bail!(
          "Outbound secret provider IP {v6} is unconditionally forbidden (cloud metadata fd00:ec2::254)"
        );
      }
      // Documentation `2001:db8::/32`
      if segs[0] == 0x2001 && segs[1] == 0x0db8 {
        bail!(
          "Outbound secret provider IP {v6} is forbidden (documentation range 2001:db8::/32)"
        );
      }
      if v6.is_loopback() {
        if !allow_loopback {
          bail!(
            "Outbound secret provider IP {v6} is loopback (::1), which is disabled by policy"
          );
        }
        return Ok(());
      }
      // Unique Local `fc00::/7`
      if v6.is_unique_local() && !allow_private_ips {
        bail!(
          "Outbound secret provider IP {v6} is in IPv6 Unique Local range (fc00::/7); enable 'allow_private_ips' on the provider to permit self-hosted internal endpoints"
        );
      }
      Ok(())
    }
  }
}

/// Strictly canonicalize and validate an external secret provider HTTPS base URL.
/// Rejects non-HTTPS schemes, embedded credentials (`user:pass@`), query strings, fragments,
/// path traversal / backslashes / encoded slashes, cloud metadata hostnames, and forbidden IP literals.
pub fn canonicalize_https_endpoint(
  raw: &str,
  allow_private_ips: bool,
  allow_loopback: bool,
) -> anyhow::Result<Url> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    bail!("Secret provider endpoint URL cannot be empty");
  }
  if trimmed.bytes().any(|b| b <= 0x20 || b == 0x7f) {
    bail!(
      "Secret provider endpoint URL contains whitespace or control characters"
    );
  }
  if trimmed.contains('\\') {
    bail!("Secret provider endpoint URL must not contain '\\'");
  }

  // Inspect the raw URL path before `Url::parse` normalizes RFC-3986 dot-segments (`/a/../b` -> `/b`)
  if let Some(after_scheme) =
    trimmed.split_once("://").map(|(_, rest)| rest)
    && let Some(slash_idx) = after_scheme.find('/')
  {
    let raw_unparsed_path = &after_scheme[slash_idx..];
    let path_only = raw_unparsed_path
      .split(['?', '#'])
      .next()
      .unwrap_or(raw_unparsed_path);
    let lower_raw_path = path_only.to_ascii_lowercase();
    if lower_raw_path.contains("%2e")
      || lower_raw_path.contains("%2f")
      || lower_raw_path.contains("%5c")
      || lower_raw_path.contains("%00")
    {
      bail!(
        "Secret provider endpoint base path must not contain percent-encoded path or traversal sequences"
      );
    }
    for seg in path_only.split('/') {
      if seg == "." || seg == ".." {
        bail!(
          "Secret provider endpoint base path must not contain '.' or '..' segments"
        );
      }
    }
  }

  let mut url = Url::parse(trimmed).with_context(|| {
    format!("Invalid secret provider endpoint URL '{trimmed}'")
  })?;

  if url.scheme() != "https" {
    bail!(
      "Secret provider endpoint '{trimmed}' must use 'https://' (got '{}://')",
      url.scheme()
    );
  }
  if !url.username().is_empty() || url.password().is_some() {
    bail!(
      "Secret provider endpoint must not contain userinfo credentials in the URL"
    );
  }
  if url.query().is_some() || url.fragment().is_some() {
    bail!(
      "Secret provider endpoint must not contain query parameters or URL fragments"
    );
  }

  match url.host() {
    Some(url::Host::Ipv4(v4)) => {
      validate_ip_addr(
        IpAddr::V4(v4),
        allow_private_ips,
        allow_loopback,
      )?;
    }
    Some(url::Host::Ipv6(v6)) => {
      validate_ip_addr(
        IpAddr::V6(v6),
        allow_private_ips,
        allow_loopback,
      )?;
    }
    Some(url::Host::Domain(domain)) => {
      let lower = domain.trim_end_matches('.').to_ascii_lowercase();
      if lower.is_empty() {
        bail!("Secret provider endpoint hostname cannot be empty");
      }
      if matches!(
        lower.as_str(),
        "metadata"
          | "metadata.google.internal"
          | "instance-data"
          | "169.254.169.254.nip.io"
      ) || lower.ends_with(".metadata.google.internal")
      {
        bail!(
          "Secret provider endpoint hostname '{domain}' is forbidden (cloud metadata service)"
        );
      }
      if !allow_loopback
        && (lower == "localhost" || lower.ends_with(".localhost"))
      {
        bail!(
          "Secret provider endpoint hostname '{domain}' resolves to loopback, which is disabled by policy"
        );
      }
    }
    None => bail!("Secret provider endpoint URL is missing a host"),
  }

  let raw_path = url.path().to_string();
  let lower_path = raw_path.to_ascii_lowercase();
  if lower_path.contains("%2e")
    || lower_path.contains("%2f")
    || lower_path.contains("%5c")
    || lower_path.contains("%00")
  {
    bail!(
      "Secret provider endpoint base path must not contain percent-encoded path or traversal sequences"
    );
  }

  let mut clean_segments = Vec::new();
  for seg in raw_path.split('/') {
    if seg.is_empty() {
      continue;
    }
    if seg == "." || seg == ".." {
      bail!(
        "Secret provider endpoint base path must not contain '.' or '..' segments"
      );
    }
    clean_segments.push(seg);
  }

  let normalized_path = if clean_segments.is_empty() {
    String::new()
  } else {
    format!("/{}", clean_segments.join("/"))
  };
  url.set_path(&normalized_path);

  Ok(url)
}

/// Join a fixed API sub-path (such as `"/api/v1/auth/universal-auth/login"` or `"/api/v4/secrets"`)
/// onto a canonicalized base URL without losing reverse-proxy base paths.
pub fn build_api_url(
  canonical_base: &Url,
  api_subpath: &str,
) -> anyhow::Result<Url> {
  let mut url = canonical_base.clone();
  let base_path = url.path().trim_end_matches('/');
  let sub = api_subpath.trim_start_matches('/');
  let combined = if base_path.is_empty() {
    format!("/{sub}")
  } else {
    format!("{base_path}/{sub}")
  };
  url.set_path(&combined);
  Ok(url)
}

/// Parse and validate a PEM bundle of custom root CA certificates.
pub fn parse_ca_certificates(
  ca_cert_pem: &str,
) -> anyhow::Result<Vec<Certificate>> {
  let trimmed = ca_cert_pem.trim();
  if trimmed.is_empty() {
    return Ok(Vec::new());
  }
  let certs = Certificate::from_pem_bundle(trimmed.as_bytes())
    .context("Invalid PEM certificate bundle in 'ca_cert_pem'")?;
  if certs.is_empty() {
    bail!(
      "No valid X.509 certificates found in 'ca_cert_pem' PEM bundle"
    );
  }
  Ok(certs)
}

#[derive(Debug, Clone)]
struct SafeDnsResolver {
  allow_private_ips: bool,
  allow_loopback: bool,
}

impl Resolve for SafeDnsResolver {
  fn resolve(&self, name: Name) -> Resolving {
    let host = name.as_str().to_string();
    let allow_private_ips = self.allow_private_ips;
    let allow_loopback = self.allow_loopback;

    Box::pin(async move {
      let lower = host.trim_end_matches('.').to_ascii_lowercase();
      if matches!(
        lower.as_str(),
        "metadata" | "metadata.google.internal" | "instance-data"
      ) || lower.ends_with(".metadata.google.internal")
      {
        return Err(
          anyhow!(
            "DNS resolution blocked for forbidden metadata hostname '{host}'"
          )
          .into(),
        );
      }
      if !allow_loopback
        && (lower == "localhost" || lower.ends_with(".localhost"))
      {
        return Err(
          anyhow!(
            "DNS resolution blocked for loopback hostname '{host}'"
          )
          .into(),
        );
      }

      let addrs: Vec<SocketAddr> =
        tokio::net::lookup_host((host.as_str(), 443))
          .await
          .with_context(|| {
            format!(
              "DNS lookup failed for secret provider host '{host}'"
            )
          })?
          .collect();

      if addrs.is_empty() {
        return Err(
          anyhow!(
            "DNS lookup returned zero addresses for secret provider host '{host}'"
          )
          .into(),
        );
      }

      // Fail closed if ANY resolved A/AAAA record violates outbound IP policy (prevents DNS round-robin SSRF).
      for addr in &addrs {
        validate_ip_addr(
          addr.ip(),
          allow_private_ips,
          allow_loopback,
        )
        .with_context(|| {
          format!("Host '{host}' resolved to a forbidden IP address")
        })?;
      }

      let iter: Addrs = Box::new(addrs.into_iter());
      Ok(iter)
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClientCacheKey {
  provider_key: String,
  canonical_endpoint: String,
  ca_pem_sha256: String,
  allow_private_ips: bool,
  allow_loopback: bool,
}

/// Hardened HTTP client wrapper enforcing TLS verification, no redirects, SSRF-safe DNS,
/// per-provider concurrency caps, response body size caps, deadlines, and cancellation.
#[derive(Clone)]
pub struct HardenedHttpClient {
  canonical_base: Url,
  client: Client,
  semaphore: Arc<Semaphore>,
}

pub struct BoundedHttpResponse {
  pub status: StatusCode,
  pub headers: HeaderMap,
  pub body: Vec<u8>,
}

static HTTP_CLIENT_CACHE: LazyLock<
  Mutex<HashMap<ClientCacheKey, HardenedHttpClient>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(test)]
static TEST_ALLOW_LOOPBACK: std::sync::atomic::AtomicBool =
  std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
pub fn set_test_allow_loopback(allow: bool) {
  TEST_ALLOW_LOOPBACK
    .store(allow, std::sync::atomic::Ordering::SeqCst);
}

pub fn effective_allow_loopback() -> bool {
  #[cfg(test)]
  if TEST_ALLOW_LOOPBACK.load(std::sync::atomic::Ordering::SeqCst) {
    return true;
  }
  core_config().secret_provider_allow_loopback
}

pub fn evict_provider_http_clients(provider_key: &str) {
  if let Ok(mut guard) = HTTP_CLIENT_CACHE.lock() {
    guard.retain(|k, _| k.provider_key != provider_key);
  }
}

impl HardenedHttpClient {
  pub fn for_provider(
    provider_key: &str,
    raw_endpoint: &str,
    ca_cert_pem: &str,
    allow_private_ips: bool,
  ) -> anyhow::Result<Self> {
    Self::build_or_get(
      provider_key,
      raw_endpoint,
      ca_cert_pem,
      allow_private_ips,
      effective_allow_loopback(),
    )
  }

  pub fn build_or_get(
    provider_key: &str,
    raw_endpoint: &str,
    ca_cert_pem: &str,
    allow_private_ips: bool,
    allow_loopback: bool,
  ) -> anyhow::Result<Self> {
    let canonical_base = canonicalize_https_endpoint(
      raw_endpoint,
      allow_private_ips,
      allow_loopback,
    )?;
    let ca_certs = parse_ca_certificates(ca_cert_pem)?;
    let ca_pem_sha256 = if ca_certs.is_empty() {
      String::new()
    } else {
      hex::encode(Sha256::digest(ca_cert_pem.trim().as_bytes()))
    };

    let cache_key = ClientCacheKey {
      provider_key: provider_key.to_string(),
      canonical_endpoint: canonical_base.as_str().to_string(),
      ca_pem_sha256,
      allow_private_ips,
      allow_loopback,
    };

    if !provider_key.is_empty()
      && let Ok(guard) = HTTP_CLIENT_CACHE.lock()
      && let Some(existing) = guard.get(&cache_key)
    {
      return Ok(existing.clone());
    }

    let resolver = Arc::new(SafeDnsResolver {
      allow_private_ips,
      allow_loopback,
    });

    let mut builder = Client::builder()
      .https_only(true)
      .redirect(redirect::Policy::none())
      .tls_version_min(reqwest::tls::Version::TLS_1_2)
      .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
      .timeout(DEFAULT_REQUEST_TIMEOUT)
      .pool_idle_timeout(Duration::from_secs(30))
      .dns_resolver(resolver)
      .user_agent(format!(
        "komodo-core/{}",
        env!("CARGO_PKG_VERSION")
      ));

    if !ca_certs.is_empty() {
      // Pin TLS verification strictly to the admin-supplied private CA bundle (never disabling cert/hostname verification).
      builder = builder.tls_certs_only(ca_certs);
    }

    let client = builder.build().context(
      "Failed to construct hardened TLS HTTP client for secret provider",
    )?;

    let instance = Self {
      canonical_base,
      client,
      semaphore: Arc::new(Semaphore::new(
        MAX_CONCURRENT_REQUESTS_PER_PROVIDER,
      )),
    };

    if !provider_key.is_empty()
      && let Ok(mut guard) = HTTP_CLIENT_CACHE.lock()
    {
      // Evict stale entries for the same provider before inserting the new configuration.
      guard.retain(|k, _| k.provider_key != provider_key);
      guard.insert(cache_key, instance.clone());
    }

    Ok(instance)
  }

  pub fn canonical_base(&self) -> &Url {
    &self.canonical_base
  }

  /// Execute an HTTP request under the provider concurrency semaphore, overall `deadline`,
  /// `cancel` token, strict no-redirect check, and streamed `MAX_SECRET_RESPONSE_BYTES` body limit.
  pub async fn send_bounded<F>(
    &self,
    method: Method,
    subpath: &str,
    configure: F,
    deadline: Instant,
    cancel: &CancellationToken,
  ) -> anyhow::Result<BoundedHttpResponse>
  where
    F: FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
  {
    if cancel.is_cancelled() {
      bail!("Secret provider request cancelled");
    }
    let now = Instant::now();
    if now >= deadline {
      bail!("Secret provider operation deadline exceeded");
    }
    let remaining = deadline.duration_since(now);

    let url = build_api_url(&self.canonical_base, subpath)?;

    let permit = tokio::select! {
      biased;
      _ = cancel.cancelled() => {
        bail!("Secret provider request cancelled while waiting for concurrency slot");
      }
      res = tokio::time::timeout(remaining, self.semaphore.acquire()) => {
        res
          .context("Timed out waiting for secret provider concurrency slot")?
          .map_err(|_| anyhow!("Secret provider concurrency semaphore closed"))?
      }
    };

    let now = Instant::now();
    if now >= deadline {
      drop(permit);
      bail!("Secret provider operation deadline exceeded");
    }
    let req_timeout =
      deadline.duration_since(now).min(DEFAULT_REQUEST_TIMEOUT);

    let builder = configure(
      self.client.request(method, url).timeout(req_timeout),
    );

    let response = tokio::select! {
      biased;
      _ = cancel.cancelled() => {
        bail!("Secret provider HTTP request cancelled");
      }
      res = builder.send() => {
        res.map_err(sanitize_reqwest_error)?
      }
    };

    let bounded =
      read_bounded_response(response, deadline, cancel).await;
    drop(permit);
    bounded
  }
}

async fn read_bounded_response(
  mut response: Response,
  deadline: Instant,
  cancel: &CancellationToken,
) -> anyhow::Result<BoundedHttpResponse> {
  let status = response.status();
  if status.is_redirection() {
    bail!(
      "Upstream secret provider returned HTTP redirect ({status}), which is forbidden by security policy"
    );
  }

  if let Some(content_len) = response.content_length()
    && content_len > MAX_SECRET_RESPONSE_BYTES as u64
  {
    bail!(
      "Upstream secret provider response Content-Length ({content_len} bytes) exceeds maximum allowed size ({MAX_SECRET_RESPONSE_BYTES} bytes)"
    );
  }

  let headers = response.headers().clone();
  let mut body = BytesMut::new();

  loop {
    let now = Instant::now();
    if now >= deadline {
      bail!(
        "Secret provider operation deadline exceeded while reading response body"
      );
    }
    let remaining = deadline.duration_since(now);

    let next_chunk = tokio::select! {
      biased;
      _ = cancel.cancelled() => {
        bail!("Secret provider response read cancelled");
      }
      res = tokio::time::timeout(remaining, response.chunk()) => {
        res
          .context("Timed out reading secret provider response body")?
          .map_err(sanitize_reqwest_error)?
      }
    };

    let Some(chunk) = next_chunk else {
      break;
    };

    if body.len().saturating_add(chunk.len())
      > MAX_SECRET_RESPONSE_BYTES
    {
      bail!(
        "Upstream secret provider response body exceeds maximum allowed size ({MAX_SECRET_RESPONSE_BYTES} bytes)"
      );
    }
    body.extend_from_slice(&chunk);
  }

  Ok(BoundedHttpResponse {
    status,
    headers,
    body: body.to_vec(),
  })
}

/// Strip URL query parameters/userinfo from reqwest transport errors while preserving actionable TLS/connect causes.
fn sanitize_reqwest_error(err: reqwest::Error) -> anyhow::Error {
  let clean = err.without_url();
  if clean.is_timeout() {
    anyhow!("Upstream secret provider request timed out: {clean}")
  } else if clean.is_connect() {
    anyhow!(
      "Failed to connect to secret provider over TLS: {clean:#}"
    )
  } else {
    anyhow!("Secret provider HTTP transport error: {clean:#}")
  }
}

/// Extract a short, safe error summary from a non-2xx JSON/text response body without echoing large or sensitive payloads.
pub fn summarize_upstream_error_body(body: &[u8]) -> String {
  if let Ok(val) = serde_json::from_slice::<serde_json::Value>(body)
    && let Some(msg) = val
      .get("message")
      .and_then(|m| m.as_str())
      .or_else(|| val.get("error").and_then(|m| m.as_str()))
  {
    let cleaned: String =
      msg.chars().filter(|c| !c.is_control()).take(200).collect();
    if !cleaned.is_empty() {
      return cleaned;
    }
  }
  String::from("upstream returned an error response")
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  #[test]
  fn ssrf_ip_validation_blocks_metadata_link_local_mapped_and_gated_private()
   {
    // Public IPv4 & IPv6 allowed
    assert!(
      validate_ip_addr(
        IpAddr::from_str("93.184.216.34").unwrap(),
        false,
        false
      )
      .is_ok()
    );
    assert!(
      validate_ip_addr(
        IpAddr::from_str("2606:2800:220:1:248:1893:25c8:1946")
          .unwrap(),
        false,
        false
      )
      .is_ok()
    );

    // Cloud metadata & link-local ALWAYS blocked even when allow_private_ips=true & allow_loopback=true
    for always_blocked in [
      "169.254.169.254",
      "169.254.170.2",
      "100.100.100.200",
      "192.0.0.192",
      "0.0.0.0",
      "255.255.255.255",
      "224.0.0.1",
      "::ffff:169.254.169.254",
      "64:ff9b::169.254.169.254",
      "2002:a9fe:a9fe::1",
      "fe80::1",
      "fd00:ec2::254",
      "::",
    ] {
      let ip = IpAddr::from_str(always_blocked).unwrap();
      assert!(
        validate_ip_addr(ip, true, true).is_err(),
        "expected {always_blocked} to be unconditionally blocked"
      );
    }

    // Private & ULA & CGNAT blocked by default, allowed when allow_private_ips=true
    for private_ip in [
      "10.1.2.3",
      "172.16.5.4",
      "192.168.1.10",
      "100.64.1.2",
      "::ffff:10.0.0.5",
      "fd12:3456:789a::1",
    ] {
      let ip = IpAddr::from_str(private_ip).unwrap();
      assert!(validate_ip_addr(ip, false, false).is_err());
      assert!(validate_ip_addr(ip, true, false).is_ok());
    }

    // Loopback gated strictly on allow_loopback (NOT enabled merely by allow_private_ips)
    for lo in ["127.0.0.1", "::1", "::ffff:127.0.0.1"] {
      let ip = IpAddr::from_str(lo).unwrap();
      assert!(validate_ip_addr(ip, true, false).is_err());
      assert!(validate_ip_addr(ip, false, true).is_ok());
    }
  }

  #[test]
  fn endpoint_canonicalization_enforces_https_and_rejects_tricks() {
    let ok = canonicalize_https_endpoint(
      "https://app.infisical.com/",
      false,
      false,
    )
    .unwrap();
    assert_eq!(ok.as_str(), "https://app.infisical.com/");

    let subpath = canonicalize_https_endpoint(
      "https://vault.corp.example.com/infisical/",
      false,
      false,
    )
    .unwrap();
    let joined = build_api_url(&subpath, "/api/v4/secrets").unwrap();
    assert_eq!(
      joined.as_str(),
      "https://vault.corp.example.com/infisical/api/v4/secrets"
    );

    // Reject HTTP, userinfo, query, fragment, traversal, metadata host, loopback host
    for bad in [
      "http://app.infisical.com",
      "ws://app.infisical.com",
      "https://user:pass@app.infisical.com",
      "https://app.infisical.com?foo=bar",
      "https://app.infisical.com#frag",
      "https://app.infisical.com/a/../b",
      "https://app.infisical.com/%2e%2e/b",
      "https://metadata.google.internal",
      "https://169.254.169.254",
      "https://localhost:8443",
      "https://10.0.0.5:8443",
    ] {
      assert!(
        canonicalize_https_endpoint(bad, false, false).is_err(),
        "expected {bad} to be rejected"
      );
    }

    assert!(
      canonicalize_https_endpoint(
        "https://10.0.0.5:8443",
        true,
        false
      )
      .is_ok()
    );
  }
}
