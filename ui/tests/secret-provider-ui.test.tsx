import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { MantineProvider } from "@mantine/core";
import { Types } from "komodo_client";
import { renderToStaticMarkup } from "react-dom/server";
import {
  ConnectionTestResultView,
  StackSecretWarningsBanner,
} from "@/components/secret-provider/views";
import {
  buildCredentialUpdatePayload,
  buildInlineVaultToken,
  buildWholeValueEnvSnippet,
  extractInlineVaultRefs,
  hasComposeWrapperConflict,
  hasInfisicalTargetChanged,
  INFISICAL_CLOUD_EU_URL,
  INFISICAL_CLOUD_US_URL,
  resolveEffectiveCredentialMode,
  validateProviderSlug,
  validateStackSecretBinding,
} from "@/components/secret-provider/utils";

function renderWithMantine(node: React.ReactNode): string {
  return renderToStaticMarkup(<MantineProvider>{node}</MantineProvider>);
}

describe("SecretProvider slug & inline reference helpers", () => {
  it("accepts canonical lowercase slugs and rejects dots/uppercase/invalid hyphens", () => {
    assert.equal(validateProviderSlug("infisical-prod"), null);
    assert.equal(validateProviderSlug("vault1"), null);
    assert.match(validateProviderSlug("") ?? "", /required/i);
    assert.match(validateProviderSlug("infisical.prod") ?? "", /cannot contain '\.'/);
    assert.match(validateProviderSlug("Infisical-Prod") ?? "", /lowercase/i);
    assert.match(validateProviderSlug("-infisical") ?? "", /start or end/i);
    assert.match(validateProviderSlug("infisical-") ?? "", /start or end/i);
    assert.match(validateProviderSlug("infisical--prod") ?? "", /consecutive/i);
    assert.match(validateProviderSlug("a".repeat(65)) ?? "", /64/);
  });

  it("preserves dots in remote secret keys literally and builds whole-value snippets", () => {
    assert.equal(
      buildInlineVaultToken("infisical-prod", "app.db.password"),
      "[[vault.infisical-prod.app.db.password]]",
    );
    assert.equal(
      buildWholeValueEnvSnippet(
        "DATABASE_URL",
        "infisical-prod",
        "DATABASE_URL",
      ),
      "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
    );
  });

  it("extracts deduplicated inline vault refs across whole-value and mixed lines while skipping comments", () => {
    const env = [
      "  # COMMENTED=[[vault.infisical-prod.IGNORED_SECRET]]",
      "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
      "MIXED=postgres://[[DB_USER]]:[[vault.infisical-prod.DB_PASSWORD]]@db:5432/app?ro=[[vault.infisical-eu.RO_PASS]]&dup=[[vault.infisical-prod.DB_PASSWORD]]",
    ].join("\n");

    const refs = extractInlineVaultRefs(env);
    assert.deepEqual(
      refs.map((r) => ({ slug: r.slug, key: r.key })),
      [
        { slug: "infisical-prod", key: "DATABASE_URL" },
        { slug: "infisical-prod", key: "DB_PASSWORD" },
        { slug: "infisical-eu", key: "RO_PASS" },
      ],
    );
  });
});

describe("Credential Keep/Replace/Clear & target-change exfiltration guard", () => {
  const baseConfig: Types.InfisicalProviderConfig = {
    endpoint: INFISICAL_CLOUD_US_URL,
    client_id: "client-123",
    organization_slug: "acme",
    allow_private_ips: false,
    ca_cert_pem: undefined,
  };

  it("treats trailing slash or host casing differences as unchanged target", () => {
    assert.equal(
      hasInfisicalTargetChanged(baseConfig, {
        ...baseConfig,
        endpoint: "HTTPS://APP.INFISICAL.COM/",
      }),
      false,
    );
  });

  it("detects endpoint, client_id, org, CA, and allow_private_ips changes and blocks silent Keep", () => {
    assert.equal(
      hasInfisicalTargetChanged(baseConfig, {
        ...baseConfig,
        endpoint: INFISICAL_CLOUD_EU_URL,
      }),
      true,
    );
    assert.equal(
      hasInfisicalTargetChanged(baseConfig, {
        ...baseConfig,
        allow_private_ips: true,
      }),
      true,
    );

    const modeState = resolveEffectiveCredentialMode({
      isEdit: true,
      hasStoredCredentials: true,
      targetChanged: true,
      selectedMode: "Keep",
    });
    assert.equal(modeState.keepBlockedByTargetChange, true);
    assert.equal(modeState.effectiveMode, "Replace");

    const clearState = resolveEffectiveCredentialMode({
      isEdit: true,
      hasStoredCredentials: true,
      targetChanged: true,
      selectedMode: "Clear",
    });
    assert.equal(clearState.keepBlockedByTargetChange, true);
    assert.equal(clearState.effectiveMode, "Clear");
  });

  it("rejects ******** sentinel masks and empty secrets when mode is Replace", () => {
    const masked = buildCredentialUpdatePayload("Replace", "********");
    assert.equal(masked.payload, undefined);
    assert.match(masked.error ?? "", /cannot be submitted/i);

    const empty = buildCredentialUpdatePayload("Replace", "   ");
    assert.equal(empty.payload, undefined);
    assert.match(empty.error ?? "", /required/i);

    const valid = buildCredentialUpdatePayload(
      "Replace",
      "real-infisical-secret",
    );
    assert.equal(valid.error, undefined);
    assert.deepEqual(valid.payload, {
      type: "Replace",
      params: {
        type: "Infisical",
        params: { client_secret: "real-infisical-secret" },
      },
    });
  });
});

describe("Stack wrapper conflict & structured binding validation", () => {
  it("flags compose_cmd_wrapper conflict only when active non-comment wrapper and external secrets coexist", () => {
    assert.equal(
      hasComposeWrapperConflict({
        environment: "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
        compose_cmd_wrapper: "# sops exec-env .encrypted.env '[[COMPOSE_COMMAND]]'\n",
      }),
      false,
    );

    assert.equal(
      hasComposeWrapperConflict({
        environment: "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
        compose_cmd_wrapper: "sops exec-env .encrypted.env '[[COMPOSE_COMMAND]]'",
      }),
      true,
    );
  });

  it("requires explicit target services for ServiceEnv and ComposeFileSecret bindings", () => {
    assert.match(
      validateStackSecretBinding({
        source: "infisical-prod",
        remote_key: "DB_PASSWORD",
        target: {
          type: "ServiceEnv",
          params: { variable: "DB_PASSWORD", services: [] },
        },
        required: true,
        allow_empty: false,
      }) ?? "",
      /at least one target Compose service/i,
    );

    assert.equal(
      validateStackSecretBinding({
        source: "infisical-prod",
        remote_key: "DB_PASSWORD",
        target: {
          type: "ComposeFileSecret",
          params: {
            secret_name: "db_password",
            services: ["api"],
            mode: 0o400,
          },
        },
        required: true,
        allow_empty: false,
      }),
      null,
    );
  });
});

describe("Rendered React UI verification components", () => {
  it("renders 3-stage ConnectionTestResultView distinguishing metadata success from value-read failure", () => {
    const html = renderWithMantine(
      <ConnectionTestResultView
        result={{
          ready_for_deploy: false,
          key_count: 4,
          authentication: {
            status: Types.SecretCheckStageStatus.Passed,
            message: "Universal Auth succeeded",
          },
          metadata_access: {
            status: Types.SecretCheckStageStatus.Passed,
            message: "Listed v4 secret metadata",
          },
          value_read_access: {
            status: Types.SecretCheckStageStatus.Failed,
            message: "secretValueHidden=true (ReadValue permission missing)",
          },
        }}
      />,
    );

    assert.match(html, /Not Verified for Deploy/);
    assert.match(html, /4 key\(s\) visible/);
    assert.match(html, /secretValueHidden=true/);
  });

  it("renders StackSecretWarningsBanner with Swarm and compose_cmd_wrapper conflict alerts plus .env 0600 notice", () => {
    const html = renderWithMantine(
      <StackSecretWarningsBanner
        swarmId="swarm-1"
        config={{
          environment: "DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]",
          compose_cmd_wrapper: "infisical run -- [[COMPOSE_COMMAND]]",
        }}
      />,
    );

    assert.match(html, /Docker Swarm native external secrets are not supported in V1/);
    assert.match(html, /Wrapper Conflict/);
    assert.match(html, /0600/);
  });
});
