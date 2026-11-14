import { Types } from "komodo_client";

export type CredentialMode = "Keep" | "Replace" | "Clear";

export const INFISICAL_CLOUD_US_URL = "https://app.infisical.com";
export const INFISICAL_CLOUD_EU_URL = "https://eu.infisical.com";

export const INFISICAL_ENDPOINT_PRESETS = [
  { label: "Infisical Cloud (US)", value: INFISICAL_CLOUD_US_URL },
  { label: "Infisical Cloud (EU)", value: INFISICAL_CLOUD_EU_URL },
  { label: "Self-Hosted (HTTPS)", value: "custom" },
] as const;

export function getInfisicalPresetForUrl(
  endpoint: string,
): (typeof INFISICAL_ENDPOINT_PRESETS)[number]["value"] {
  const canonical = canonicalizeEndpointForComparison(endpoint);
  if (canonical === INFISICAL_CLOUD_US_URL) return INFISICAL_CLOUD_US_URL;
  if (canonical === INFISICAL_CLOUD_EU_URL) return INFISICAL_CLOUD_EU_URL;
  return "custom";
}

export function canonicalizeEndpointForComparison(endpoint: string): string {
  const trimmed = endpoint.trim().replace(/\/+$/, "");
  if (!trimmed) return "";
  try {
    const parsed = new URL(trimmed);
    const pathname = parsed.pathname.replace(/\/+$/, "");
    return `${parsed.protocol.toLowerCase()}//${parsed.host.toLowerCase()}${pathname}`;
  } catch {
    return trimmed;
  }
}

/**
 * Validates an immutable SecretProvider slug (`[a-z0-9-]+`, 1..=64 chars, no dots,
 * no leading/trailing/consecutive hyphens).
 */
export function validateProviderSlug(slug: string): string | null {
  if (!slug) {
    return "Provider slug is required.";
  }
  if (slug.length > 64) {
    return "Provider slug cannot exceed 64 characters.";
  }
  if (slug.includes(".")) {
    return "Provider slug cannot contain '.' (dots are reserved for remote secret keys in [[vault.<slug>.<key>]]).";
  }
  if (slug.startsWith("-") || slug.endsWith("-")) {
    return "Provider slug cannot start or end with '-'.";
  }
  if (slug.includes("--")) {
    return "Provider slug cannot contain consecutive '--'.";
  }
  if (!/^[a-z0-9-]+$/.test(slug)) {
    return "Provider slug may only contain lowercase ASCII letters (a-z), digits (0-9), and single hyphens (-).";
  }
  return null;
}

/**
 * Builds an inline external secret token (`[[vault.<slug>.<key>]]`),
 * preserving dots and characters in `key` literally.
 */
export function buildInlineVaultToken(slug: string, key: string): string {
  return `[[vault.${slug}.${key}]]`;
}

/**
 * Builds a recommended whole-value environment line (`ENV_KEY=[[vault.<slug>.<key>]]`).
 */
export function buildWholeValueEnvSnippet(
  envVar: string,
  slug: string,
  key: string,
): string {
  const cleanVar = envVar.trim() || key.trim();
  return `${cleanVar}=${buildInlineVaultToken(slug, key)}`;
}

export interface InlineVaultRef {
  slug: string;
  key: string;
  token: string;
}

/**
 * Extracts `[[vault.<slug>.<key>]]` tokens from a multiline key-value string,
 * ignoring full-line `#` comments (matching Komodo's environment parser).
 */
export function extractInlineVaultRefs(input?: string | null): InlineVaultRef[] {
  if (!input) return [];
  const results: InlineVaultRef[] = [];
  const seen = new Set<string>();
  const lines = input.split(/\r?\n/);
  const pattern = /\[\[vault\.([a-z0-9-]+)\.([^\[\]\r\n]+)\]\]/g;

  for (const line of lines) {
    if (line.trimStart().startsWith("#")) continue;
    let match: RegExpExecArray | null;
    pattern.lastIndex = 0;
    while ((match = pattern.exec(line)) !== null) {
      const token = match[0];
      if (!seen.has(token)) {
        seen.add(token);
        results.push({
          slug: match[1],
          key: match[2],
          token,
        });
      }
    }
  }
  return results;
}

/**
 * Checks whether a shell command wrapper string contains active non-comment commands.
 */
export function hasActiveCommandWrapper(wrapper?: string | null): boolean {
  if (!wrapper) return false;
  return wrapper
    .split(/\r?\n/)
    .some((line) => {
      const trimmed = line.trim();
      return trimmed.length > 0 && !trimmed.startsWith("#");
    });
}

export function hasExternalSecretsInStack(
  config: Partial<Types.StackConfig>,
): boolean {
  return (
    (config.secret_sources?.length ?? 0) > 0 ||
    (config.secret_bindings?.length ?? 0) > 0 ||
    extractInlineVaultRefs(config.environment).length > 0
  );
}

export function hasExternalSecretsInDeployment(
  config: Partial<Types.DeploymentConfig>,
): boolean {
  return (
    (config.secret_sources?.length ?? 0) > 0 ||
    (config.secret_bindings?.length ?? 0) > 0 ||
    extractInlineVaultRefs(config.environment).length > 0
  );
}

export function hasExternalSecretsInBuild(
  config: Partial<Types.BuildConfig>,
): boolean {
  return (
    (config.secret_sources?.length ?? 0) > 0 ||
    (config.secret_bindings?.length ?? 0) > 0 ||
    extractInlineVaultRefs(config.secret_args).length > 0
  );
}

/**
 * V1 fails closed when native external secrets and `compose_cmd_wrapper` are configured together.
 */
export function hasComposeWrapperConflict(
  config: Partial<Types.StackConfig>,
): boolean {
  return (
    hasExternalSecretsInStack(config) &&
    hasActiveCommandWrapper(config.compose_cmd_wrapper)
  );
}

/**
 * Detects whether any target/authentication/TLS boundary field changed on an Infisical provider.
 * When true and stored credentials exist, silent `Keep` is rejected to prevent credential exfiltration.
 */
export function hasInfisicalTargetChanged(
  original: Types.InfisicalProviderConfig,
  draft: Types.InfisicalProviderConfig,
): boolean {
  return (
    canonicalizeEndpointForComparison(original.endpoint) !==
      canonicalizeEndpointForComparison(draft.endpoint) ||
    (original.client_id ?? "").trim() !== (draft.client_id ?? "").trim() ||
    (original.organization_slug ?? "").trim() !==
      (draft.organization_slug ?? "").trim() ||
    (original.ca_cert_pem ?? "").trim() !== (draft.ca_cert_pem ?? "").trim() ||
    Boolean(original.allow_private_ips) !== Boolean(draft.allow_private_ips)
  );
}

export function resolveEffectiveCredentialMode(params: {
  isEdit: boolean;
  hasStoredCredentials: boolean;
  targetChanged: boolean;
  selectedMode: CredentialMode;
}): {
  effectiveMode: CredentialMode;
  keepBlockedByTargetChange: boolean;
} {
  const { isEdit, hasStoredCredentials, targetChanged, selectedMode } = params;
  if (!isEdit) {
    return {
      effectiveMode: "Replace",
      keepBlockedByTargetChange: false,
    };
  }
  const keepBlockedByTargetChange = hasStoredCredentials && targetChanged;
  if (keepBlockedByTargetChange && selectedMode === "Keep") {
    return {
      effectiveMode: "Replace",
      keepBlockedByTargetChange: true,
    };
  }
  return {
    effectiveMode: selectedMode,
    keepBlockedByTargetChange,
  };
}

const MASK_SENTINEL_REGEX = /^\*{4,}$/;

export function isMaskSentinel(value: string): boolean {
  return MASK_SENTINEL_REGEX.test(value.trim());
}

/**
 * Builds an explicit `CredentialUpdate` payload for `UpdateSecretProvider` / `TestSecretProviderConnection`.
 * Rejects `********` mask sentinels and empty replacement values.
 */
export function buildCredentialUpdatePayload(
  mode: CredentialMode,
  clientSecretInput: string,
): { payload?: Types.CredentialUpdate; error?: string } {
  if (mode === "Keep") {
    return { payload: { type: "Keep" } };
  }
  if (mode === "Clear") {
    return { payload: { type: "Clear" } };
  }
  const trimmed = clientSecretInput.trim();
  if (!trimmed) {
    return {
      error: "Client Secret is required when Credential Action is Replace.",
    };
  }
  if (isMaskSentinel(trimmed)) {
    return {
      error:
        "Placeholder mask '********' cannot be submitted as a real Client Secret. Choose 'Keep' or paste a real secret.",
    };
  }
  return {
    payload: {
      type: "Replace",
      params: {
        type: "Infisical",
        params: { client_secret: clientSecretInput },
      },
    },
  };
}

export function validateStackSecretBinding(
  binding: Types.StackSecretBinding,
): string | null {
  if (!binding.source.trim()) {
    return "Source alias or provider slug is required.";
  }
  if (!binding.remote_key.trim()) {
    return "Remote secret key is required.";
  }
  const { target } = binding;
  if (target.type === "ComposeEnv") {
    if (!target.params.variable.trim()) {
      return "Target .env variable name is required.";
    }
    return null;
  }
  if (target.type === "ServiceEnv") {
    if (!target.params.variable.trim()) {
      return "Target service environment variable name is required.";
    }
    if (!target.params.services.some((s) => s.trim().length > 0)) {
      return "ServiceEnv binding requires at least one target Compose service.";
    }
    return null;
  }
  if (target.type === "ComposeFileSecret") {
    if (!target.params.secret_name.trim()) {
      return "Compose secret name is required.";
    }
    if (!target.params.services.some((s) => s.trim().length > 0)) {
      return "ComposeFileSecret binding requires at least one target Compose service.";
    }
    if (
      target.params.mode !== undefined &&
      (target.params.mode < 0 || target.params.mode > 0o777)
    ) {
      return "File mode must be between 0000 and 0777 (decimal 0..511, e.g. 0400 = 256).";
    }
    return null;
  }
  return null;
}

export function stageStatusBadgeColor(
  status: Types.SecretCheckStageStatus,
): string {
  switch (status) {
    case Types.SecretCheckStageStatus.Passed:
      return "green";
    case Types.SecretCheckStageStatus.Failed:
      return "red";
    case Types.SecretCheckStageStatus.Skipped:
      return "gray";
    default:
      return "gray";
  }
}
