import {
  useInvalidate,
  useRead,
  useUser,
  useWrite,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import ResourceLink from "@/resources/link";
import { UsableResource } from "@/resources";
import {
  buildCredentialUpdatePayload,
  buildInlineVaultToken,
  CredentialMode,
  getInfisicalPresetForUrl,
  hasInfisicalTargetChanged,
  INFISICAL_CLOUD_US_URL,
  INFISICAL_ENDPOINT_PRESETS,
  resolveEffectiveCredentialMode,
  validateProviderSlug,
} from "@/components/secret-provider/utils";
import { ConnectionTestResultView } from "@/components/secret-provider/views";
import {
  Alert,
  Badge,
  Button,
  Code,
  Divider,
  Group,
  Modal,
  MultiSelect,
  PasswordInput,
  SegmentedControl,
  Select,
  SimpleGrid,
  Stack,
  Switch,
  Table,
  TagsInput,
  Text,
  Textarea,
  TextInput,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import {
  ConfirmButton,
  CopyButton,
  DataTable,
  SearchInput,
  Section,
  SortableHeader,
} from "mogh_ui";
import { useEffect, useMemo, useState } from "react";

const ALL_SECRET_USAGES: Types.SecretUsage[] = [
  Types.SecretUsage.StackEnv,
  Types.SecretUsage.StackComposeFileSecret,
  Types.SecretUsage.DeploymentEnv,
  Types.SecretUsage.BuildSecret,
  Types.SecretUsage.MetadataList,
  Types.SecretUsage.Preview,
];

const ALL_EXECUTION_ACTORS: Types.SecretExecutionActor[] = [
  Types.SecretExecutionActor.Procedure,
  Types.SecretExecutionActor.Webhook,
  Types.SecretExecutionActor.Schedule,
  Types.SecretExecutionActor.AutoRedeploy,
  Types.SecretExecutionActor.ResourceSync,
  Types.SecretExecutionActor.Action,
];

export default function SecretProvidersSection() {
  const user = useUser().data;
  const disabled = !user?.admin;
  const [search, setSearch] = useState("");
  const [modalState, setModalState] = useState<{
    opened: boolean;
    provider?: Types.SecretProvider;
  }>({ opened: false });
  const [grantsProvider, setGrantsProvider] =
    useState<Types.SecretProvider | null>(null);
  const [usagesProvider, setUsagesProvider] =
    useState<Types.SecretProvider | null>(null);

  const providers =
    useRead("ListSecretProviders", {}, { enabled: !!user?.admin }).data ?? [];

  const searchTerms = search
    .toLowerCase()
    .split(" ")
    .map((t) => t.trim())
    .filter(Boolean);

  const filtered = providers.filter((p) => {
    if (searchTerms.length === 0) return true;
    const haystack = [
      p.slug ?? "",
      p.name ?? "",
      p.description ?? "",
      p.config.params.endpoint ?? "",
    ]
      .join(" ")
      .toLowerCase();
    return searchTerms.every((t) => haystack.includes(t));
  });

  const inv = useInvalidate();
  const invalidateProviders = () => {
    inv(
      ["ListSecretProviders"],
      ["GetSecretProvider"],
      ["ListAvailableSecretProviders"],
      ["ListSecretProviderUsages"],
    );
  };

  const { mutate: updateProvider } = useWrite("UpdateSecretProvider", {
    onSuccess: () => {
      invalidateProviders();
      notifications.show({ message: "Updated secret provider" });
    },
  });

  const { mutate: rotateKeys, isPending: rotatingKeys } = useWrite(
    "RotateSecretProviderEncryptionKeys",
    {
      onSuccess: (res) => {
        invalidateProviders();
        notifications.show({
          title: "Encryption keyring rotated",
          message: `Re-encrypted ${res.rotated_count} credential(s) onto active key '${res.active_key_id}'.`,
        });
      },
    },
  );

  return (
    <>
      <Section
        title="Secret Providers"
        icon={<ICONS.Key size="1.3rem" />}
      >
        <Alert color="blue" variant="light" icon={<ICONS.Info size="1rem" />}>
          External Secret Providers authenticate from Komodo Core via Infisical
          Machine Identity Universal Auth (<Code>/api/v1/auth/universal-auth/login</Code>)
          and Secrets API v4 (<Code>/api/v4/secrets</Code>). Client secrets are
          encrypted at rest with AES-256-GCM and are never returned by Read APIs
          or sent to Periphery agents. Access is default-deny until granted under{" "}
          <b>Grants</b>.
        </Alert>

        <Group justify="space-between">
          <Group>
            <Button
              leftSection={<ICONS.Create size="1rem" />}
              disabled={disabled}
              onClick={() => setModalState({ opened: true })}
            >
              New Secret Provider
            </Button>
            <ConfirmButton
              icon={<ICONS.RotateKey size="1rem" />}
              disabled={disabled || providers.length === 0}
              loading={rotatingKeys}
              onClick={() => rotateKeys({})}
            >
              Rotate Encryption Keys
            </ConfirmButton>
          </Group>
          <SearchInput value={search} onSearch={setSearch} />
        </Group>

        <DataTable
          tableKey="secret-providers"
          data={filtered}
          columns={[
            {
              accessorKey: "name",
              header: ({ column }) => (
                <SortableHeader column={column} title="Provider" />
              ),
              cell: ({ row }) => {
                const p = row.original;
                return (
                  <Stack gap={2}>
                    <Group gap="xs">
                      <Text fw={600}>{p.name || p.slug}</Text>
                      <Badge size="xs" variant="light">
                        {p.config.type}
                      </Badge>
                    </Group>
                    {p.description && (
                      <Text fz="xs" c="dimmed">
                        {p.description}
                      </Text>
                    )}
                  </Stack>
                );
              },
            },
            {
              accessorKey: "slug",
              header: ({ column }) => (
                <SortableHeader column={column} title="Slug & Inline Token" />
              ),
              cell: ({ row }) => {
                const slug = row.original.slug ?? "";
                const sampleToken = buildInlineVaultToken(slug, "SECRET_KEY");
                return (
                  <Group gap="xs" wrap="nowrap">
                    <Code>{slug}</Code>
                    <CopyButton content={sampleToken} />
                  </Group>
                );
              },
            },
            {
              id: "endpoint",
              header: "Endpoint & Default Scope",
              cell: ({ row }) => {
                const cfg = row.original.config.params;
                const def = cfg.default_selector;
                return (
                  <Stack gap={2}>
                    <Group gap="xs">
                      <Text fz="sm">{cfg.endpoint}</Text>
                      {cfg.allow_private_ips && (
                        <Badge size="xs" color="orange" variant="outline">
                          Private IPs
                        </Badge>
                      )}
                      {cfg.ca_cert_pem && (
                        <Badge size="xs" color="cyan" variant="outline">
                          Custom CA
                        </Badge>
                      )}
                    </Group>
                    {def && (
                      <Text fz="xs" c="dimmed">
                        Default: {def.project_id} / {def.environment} (
                        {def.secret_path || "/"})
                      </Text>
                    )}
                  </Stack>
                );
              },
            },
            {
              id: "credentials",
              header: "Credentials",
              cell: ({ row }) => {
                const p = row.original;
                return p.has_credentials ? (
                  <Badge color="green" variant="light">
                    Stored (rev {p.credential_revision ?? 1})
                  </Badge>
                ) : (
                  <Badge color="yellow" variant="light">
                    No Credentials
                  </Badge>
                );
              },
            },
            {
              accessorKey: "enabled",
              header: "Enabled",
              cell: ({ row }) => {
                const p = row.original;
                const id = p._id?.$oid;
                return (
                  <Switch
                    checked={p.enabled}
                    disabled={disabled || !id}
                    onChange={(e) => {
                      if (!id) return;
                      updateProvider({
                        id,
                        expected_revision: p.revision,
                        enabled: e.currentTarget.checked,
                        credential_update: { type: "Keep" },
                      });
                    }}
                  />
                );
              },
            },
            {
              id: "actions",
              header: "Actions",
              cell: ({ row }) => {
                const p = row.original;
                return (
                  <Group gap="xs" wrap="nowrap">
                    <Button
                      size="xs"
                      variant="default"
                      leftSection={<ICONS.Edit size="0.9rem" />}
                      disabled={disabled}
                      onClick={() =>
                        setModalState({ opened: true, provider: p })
                      }
                    >
                      Edit / Test
                    </Button>
                    <Button
                      size="xs"
                      variant="light"
                      leftSection={<ICONS.Key size="0.9rem" />}
                      disabled={disabled}
                      onClick={() => setGrantsProvider(p)}
                    >
                      Grants
                    </Button>
                    <Button
                      size="xs"
                      variant="subtle"
                      color="red"
                      leftSection={<ICONS.Link size="0.9rem" />}
                      disabled={disabled}
                      onClick={() => setUsagesProvider(p)}
                    >
                      Usages / Delete
                    </Button>
                  </Group>
                );
              },
            },
          ]}
        />
      </Section>

      <SecretProviderModal
        opened={modalState.opened}
        provider={modalState.provider}
        onClose={() => setModalState({ opened: false })}
        onSaved={invalidateProviders}
      />

      <SecretProviderGrantsModal
        provider={grantsProvider}
        onClose={() => setGrantsProvider(null)}
      />

      <SecretProviderUsagesModal
        provider={usagesProvider}
        onClose={() => setUsagesProvider(null)}
        onDeleted={() => {
          setUsagesProvider(null);
          invalidateProviders();
        }}
      />
    </>
  );
}

function SecretProviderModal({
  opened,
  provider,
  onClose,
  onSaved,
}: {
  opened: boolean;
  provider?: Types.SecretProvider;
  onClose: () => void;
  onSaved: () => void;
}) {
  const isEdit = !!provider;
  const providerId = provider?._id?.$oid;
  const hasStoredCredentials = !!provider?.has_credentials;

  const [slug, setSlug] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [enabled, setEnabled] = useState(true);

  const [preset, setPreset] = useState<string>(INFISICAL_CLOUD_US_URL);
  const [endpoint, setEndpoint] = useState(INFISICAL_CLOUD_US_URL);
  const [clientId, setClientId] = useState("");
  const [organizationSlug, setOrganizationSlug] = useState("");
  const [allowPrivateIps, setAllowPrivateIps] = useState(false);
  const [caCertPem, setCaCertPem] = useState("");

  const [useDefaultSelector, setUseDefaultSelector] = useState(false);
  const [defProjectId, setDefProjectId] = useState("");
  const [defEnvironment, setDefEnvironment] = useState("prod");
  const [defSecretPath, setDefSecretPath] = useState("/");
  const [defIncludeImports, setDefIncludeImports] = useState(false);
  const [defExpandRefs, setDefExpandRefs] = useState(false);

  const [selectedCredentialMode, setSelectedCredentialMode] =
    useState<CredentialMode>("Replace");
  const [clientSecret, setClientSecret] = useState("");

  const [testProjectId, setTestProjectId] = useState("");
  const [testEnvironment, setTestEnvironment] = useState("");
  const [testSecretPath, setTestSecretPath] = useState("/");
  const [testSampleKey, setTestSampleKey] = useState("");
  const [testResult, setTestResult] =
    useState<Types.TestSecretProviderResponse | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  useEffect(() => {
    if (!opened) return;
    setFormError(null);
    setTestResult(null);
    setClientSecret("");
    if (provider) {
      const cfg = provider.config.params;
      setSlug(provider.slug ?? "");
      setName(provider.name ?? "");
      setDescription(provider.description ?? "");
      setEnabled(provider.enabled);
      setEndpoint(cfg.endpoint);
      setPreset(getInfisicalPresetForUrl(cfg.endpoint));
      setClientId(cfg.client_id ?? "");
      setOrganizationSlug(cfg.organization_slug ?? "");
      setAllowPrivateIps(!!cfg.allow_private_ips);
      setCaCertPem(cfg.ca_cert_pem ?? "");
      if (cfg.default_selector) {
        setUseDefaultSelector(true);
        setDefProjectId(cfg.default_selector.project_id);
        setDefEnvironment(cfg.default_selector.environment);
        setDefSecretPath(cfg.default_selector.secret_path || "/");
        setDefIncludeImports(!!cfg.default_selector.include_imports);
        setDefExpandRefs(!!cfg.default_selector.expand_references);
        setTestProjectId(cfg.default_selector.project_id);
        setTestEnvironment(cfg.default_selector.environment);
        setTestSecretPath(cfg.default_selector.secret_path || "/");
      } else {
        setUseDefaultSelector(false);
        setDefProjectId("");
        setDefEnvironment("prod");
        setDefSecretPath("/");
        setDefIncludeImports(false);
        setDefExpandRefs(false);
        setTestProjectId("");
        setTestEnvironment("");
        setTestSecretPath("/");
      }
      setTestSampleKey("");
      setSelectedCredentialMode(
        provider.has_credentials ? "Keep" : "Replace",
      );
    } else {
      setSlug("");
      setName("");
      setDescription("");
      setEnabled(true);
      setPreset(INFISICAL_CLOUD_US_URL);
      setEndpoint(INFISICAL_CLOUD_US_URL);
      setClientId("");
      setOrganizationSlug("");
      setAllowPrivateIps(false);
      setCaCertPem("");
      setUseDefaultSelector(false);
      setDefProjectId("");
      setDefEnvironment("prod");
      setDefSecretPath("/");
      setDefIncludeImports(false);
      setDefExpandRefs(false);
      setSelectedCredentialMode("Replace");
      setTestProjectId("");
      setTestEnvironment("");
      setTestSecretPath("/");
      setTestSampleKey("");
    }
  }, [opened, provider]);

  const draftInfisicalConfig: Types.InfisicalProviderConfig = useMemo(
    () => ({
      endpoint: endpoint.trim(),
      client_id: clientId.trim() || undefined,
      organization_slug: organizationSlug.trim() || undefined,
      ca_cert_pem: caCertPem.trim() || undefined,
      allow_private_ips: allowPrivateIps,
      default_selector: useDefaultSelector
        ? {
            project_id: defProjectId.trim(),
            environment: defEnvironment.trim(),
            secret_path: defSecretPath.trim() || "/",
            include_imports: defIncludeImports,
            expand_references: defExpandRefs,
          }
        : undefined,
    }),
    [
      endpoint,
      clientId,
      organizationSlug,
      caCertPem,
      allowPrivateIps,
      useDefaultSelector,
      defProjectId,
      defEnvironment,
      defSecretPath,
      defIncludeImports,
      defExpandRefs,
    ],
  );

  const targetChanged = useMemo(() => {
    if (!provider) return false;
    return hasInfisicalTargetChanged(
      provider.config.params,
      draftInfisicalConfig,
    );
  }, [provider, draftInfisicalConfig]);

  const { effectiveMode, keepBlockedByTargetChange } = useMemo(
    () =>
      resolveEffectiveCredentialMode({
        isEdit,
        hasStoredCredentials,
        targetChanged,
        selectedMode: selectedCredentialMode,
      }),
    [isEdit, hasStoredCredentials, targetChanged, selectedCredentialMode],
  );

  const slugError = !isEdit && slug ? validateProviderSlug(slug) : null;

  const { mutate: createProvider, isPending: creating } = useWrite(
    "CreateSecretProvider",
    {
      onSuccess: () => {
        onSaved();
        notifications.show({ message: "Secret provider created" });
        onClose();
      },
    },
  );

  const { mutate: updateProvider, isPending: updating } = useWrite(
    "UpdateSecretProvider",
    {
      onSuccess: () => {
        onSaved();
        notifications.show({ message: "Secret provider updated" });
        onClose();
      },
    },
  );

  const { mutate: testConnection, isPending: testing } = useWrite(
    "TestSecretProviderConnection",
    {
      onSuccess: (res) => {
        setTestResult(res);
      },
    },
  );

  const handleTest = () => {
    setFormError(null);
    if (!draftInfisicalConfig.endpoint.startsWith("https://")) {
      setFormError("Endpoint must start with https://.");
      return;
    }
    const credBuild = buildCredentialUpdatePayload(effectiveMode, clientSecret);
    if (credBuild.error || !credBuild.payload) {
      setFormError(credBuild.error ?? "Invalid credential configuration.");
      return;
    }
    const selectorProject = testProjectId.trim() || defProjectId.trim();
    const selectorEnv = testEnvironment.trim() || defEnvironment.trim();
    const selectorPath =
      testSecretPath.trim() || defSecretPath.trim() || "/";
    const selector: Types.SecretSelector | undefined =
      selectorProject && selectorEnv
        ? {
            type: "Infisical",
            params: {
              project_id: selectorProject,
              environment: selectorEnv,
              secret_path: selectorPath,
              include_imports: defIncludeImports,
              expand_references: defExpandRefs,
            },
          }
        : undefined;

    testConnection({
      id: providerId,
      config: { type: "Infisical", params: draftInfisicalConfig },
      credential_update: credBuild.payload,
      selector,
      sample_key: testSampleKey.trim() || undefined,
    });
  };

  const handleSave = () => {
    setFormError(null);
    if (!isEdit) {
      const sErr = validateProviderSlug(slug.trim());
      if (sErr) {
        setFormError(sErr);
        return;
      }
    }
    if (!name.trim()) {
      setFormError("Display name is required.");
      return;
    }
    if (!draftInfisicalConfig.endpoint.startsWith("https://")) {
      setFormError("Endpoint must start with https://.");
      return;
    }
    if (
      useDefaultSelector &&
      (!defProjectId.trim() || !defEnvironment.trim())
    ) {
      setFormError(
        "Default Selector requires both Project ID and Environment slug.",
      );
      return;
    }

    if (!isEdit) {
      let credential: Types.SecretProviderCredentialInput | undefined;
      if (clientSecret.trim()) {
        const credBuild = buildCredentialUpdatePayload("Replace", clientSecret);
        if (credBuild.error || !credBuild.payload) {
          setFormError(credBuild.error ?? "Invalid Client Secret.");
          return;
        }
        if (credBuild.payload.type === "Replace") {
          credential = credBuild.payload.params;
        }
      }
      createProvider({
        slug: slug.trim(),
        name: name.trim(),
        description: description.trim(),
        enabled,
        config: { type: "Infisical", params: draftInfisicalConfig },
        credential,
      });
    } else if (provider && providerId) {
      const credBuild = buildCredentialUpdatePayload(
        effectiveMode,
        clientSecret,
      );
      if (credBuild.error || !credBuild.payload) {
        setFormError(credBuild.error ?? "Invalid credential update.");
        return;
      }
      updateProvider({
        id: providerId,
        expected_revision: provider.revision,
        name: name.trim(),
        description: description.trim(),
        enabled,
        config: { type: "Infisical", params: draftInfisicalConfig },
        credential_update: credBuild.payload,
      });
    }
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Text fw={600} size="lg">
          {isEdit
            ? `Edit Secret Provider: ${provider?.slug}`
            : "Create Infisical Secret Provider"}
        </Text>
      }
      size="xl"
    >
      <Stack gap="md">
        {formError && (
          <Alert color="red" icon={<ICONS.Alert size="1rem" />}>
            {formError}
          </Alert>
        )}

        <SimpleGrid cols={{ base: 1, sm: 2 }}>
          <TextInput
            label="Immutable Slug"
            description="Used in [[vault.<slug>.<key>]]. Lowercase a-z, 0-9, single hyphens; no dots."
            placeholder="infisical-prod"
            value={slug}
            disabled={isEdit}
            error={slugError}
            onChange={(e) => setSlug(e.currentTarget.value)}
            required
          />
          <TextInput
            label="Display Name"
            description="Human-readable label shown in Settings and pickers."
            placeholder="Infisical Production"
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
            required
          />
        </SimpleGrid>

        <Group justify="space-between" align="flex-end">
          <TextInput
            label="Description"
            placeholder="Production secrets for Compose & Deployments"
            value={description}
            onChange={(e) => setDescription(e.currentTarget.value)}
            style={{ flex: 1 }}
          />
          <Switch
            label="Provider Enabled"
            checked={enabled}
            onChange={(e) => setEnabled(e.currentTarget.checked)}
            mb={8}
          />
        </Group>

        <Divider label="Infisical Endpoint & Machine Identity" />

        <SegmentedControl
          data={[...INFISICAL_ENDPOINT_PRESETS]}
          value={preset}
          onChange={(val) => {
            setPreset(val);
            if (val !== "custom") {
              setEndpoint(val);
            }
          }}
        />

        <TextInput
          label="HTTPS Endpoint URL"
          description="Infisical Cloud or self-hosted HTTPS base URL (redirects are rejected)."
          placeholder="https://app.infisical.com"
          value={endpoint}
          onChange={(e) => {
            const next = e.currentTarget.value;
            setEndpoint(next);
            setPreset(getInfisicalPresetForUrl(next));
          }}
          required
        />

        <SimpleGrid cols={{ base: 1, sm: 2 }}>
          <TextInput
            label="Universal Auth Client ID"
            description="Machine Identity Client ID (least-privilege ReadValue + DescribeSecret)."
            placeholder="00000000-0000-0000-0000-000000000000"
            value={clientId}
            onChange={(e) => setClientId(e.currentTarget.value)}
          />
          <TextInput
            label="Organization Slug (Optional)"
            description="Optional organization scoping passed during Universal Auth login."
            placeholder="my-org"
            value={organizationSlug}
            onChange={(e) => setOrganizationSlug(e.currentTarget.value)}
          />
        </SimpleGrid>

        {isEdit && (
          <Stack gap="xs">
            <Group justify="space-between">
              <Text fw={500} fz="sm">
                Client Secret Action
              </Text>
              <SegmentedControl
                size="xs"
                value={effectiveMode}
                onChange={(val) =>
                  setSelectedCredentialMode(val as CredentialMode)
                }
                data={[
                  {
                    label: hasStoredCredentials
                      ? "Keep Stored"
                      : "Keep Empty",
                    value: "Keep",
                    disabled: keepBlockedByTargetChange,
                  },
                  { label: "Replace Secret", value: "Replace" },
                  {
                    label: "Clear Stored",
                    value: "Clear",
                    disabled: !hasStoredCredentials,
                  },
                ]}
              />
            </Group>
            {keepBlockedByTargetChange && (
              <Alert color="orange" icon={<ICONS.Alert size="1rem" />}>
                Endpoint, Client ID, Organization, Private IP policy, or CA
                certificate changed. To prevent sending existing stored
                credentials to a modified endpoint, you must explicitly{" "}
                <b>Replace Secret</b> or <b>Clear Stored</b>.
              </Alert>
            )}
          </Stack>
        )}

        {(!isEdit || effectiveMode === "Replace") && (
          <PasswordInput
            label="Universal Auth Client Secret"
            description="Encrypted at rest using AES-256-GCM; never returned by Read APIs."
            placeholder="Paste Infisical Universal Auth Client Secret"
            value={clientSecret}
            onChange={(e) => setClientSecret(e.currentTarget.value)}
            required={isEdit && effectiveMode === "Replace"}
          />
        )}

        <Divider label="TLS & Network Controls (Self-Hosted)" />

        <Switch
          label="Allow Private / Internal IP Ranges"
          description="Enable only for trusted self-hosted Infisical instances on RFC1918/internal networks. Cloud metadata and link-local ranges remain unconditionally blocked."
          checked={allowPrivateIps}
          onChange={(e) => setAllowPrivateIps(e.currentTarget.checked)}
        />

        <Textarea
          label="Custom Root CA Certificate PEM (Optional)"
          description="PEM-encoded CA certificate(s) trusted when verifying a self-hosted Infisical TLS server. Never disables certificate verification."
          placeholder="-----BEGIN CERTIFICATE-----..."
          minRows={2}
          autosize
          value={caCertPem}
          onChange={(e) => setCaCertPem(e.currentTarget.value)}
        />

        <Divider label="Optional Default Scope Selector" />

        <Switch
          label="Configure Default Infisical Project / Environment / Path"
          description="Allows inline [[vault.<slug>.<key>]] references on Stacks, Deployments, and Builds without repeating a SecretSource on each resource."
          checked={useDefaultSelector}
          onChange={(e) => setUseDefaultSelector(e.currentTarget.checked)}
        />

        {useDefaultSelector && (
          <Stack gap="xs">
            <SimpleGrid cols={{ base: 1, sm: 3 }}>
              <TextInput
                label="Project ID"
                placeholder="proj_..."
                value={defProjectId}
                onChange={(e) => setDefProjectId(e.currentTarget.value)}
                required
              />
              <TextInput
                label="Environment Slug"
                placeholder="prod"
                value={defEnvironment}
                onChange={(e) => setDefEnvironment(e.currentTarget.value)}
                required
              />
              <TextInput
                label="Secret Path"
                placeholder="/"
                value={defSecretPath}
                onChange={(e) => setDefSecretPath(e.currentTarget.value)}
              />
            </SimpleGrid>
            <Group gap="xl">
              <Switch
                label="Include Imports"
                checked={defIncludeImports}
                onChange={(e) => setDefIncludeImports(e.currentTarget.checked)}
              />
              <Switch
                label="Expand Secret References"
                checked={defExpandRefs}
                onChange={(e) => setDefExpandRefs(e.currentTarget.checked)}
              />
            </Group>
            {(defIncludeImports || defExpandRefs) && (
              <Alert color="yellow" icon={<ICONS.Alert size="1rem" />}>
                Enabling <b>Include Imports</b> or{" "}
                <b>Expand Secret References</b> allows Infisical to read
                imported folders or referenced secrets outside the initial{" "}
                <Code>{defSecretPath || "/"}</Code> path. Restrict the Infisical
                Machine Identity role strictly to permitted paths/environments,
                and explicitly enable matching flags on Secret Provider Grants.
              </Alert>
            )}
          </Stack>
        )}

        <Divider label="3-Stage Connection Verification" />

        <SimpleGrid cols={{ base: 1, sm: 4 }}>
          <TextInput
            label="Test Project ID"
            placeholder={defProjectId || "Optional projectId"}
            value={testProjectId}
            onChange={(e) => setTestProjectId(e.currentTarget.value)}
          />
          <TextInput
            label="Test Environment"
            placeholder={defEnvironment || "prod"}
            value={testEnvironment}
            onChange={(e) => setTestEnvironment(e.currentTarget.value)}
          />
          <TextInput
            label="Test Path"
            placeholder={defSecretPath || "/"}
            value={testSecretPath}
            onChange={(e) => setTestSecretPath(e.currentTarget.value)}
          />
          <TextInput
            label="Sample Key (Optional)"
            placeholder="DATABASE_URL"
            value={testSampleKey}
            onChange={(e) => setTestSampleKey(e.currentTarget.value)}
          />
        </SimpleGrid>

        {testResult && <ConnectionTestResultView result={testResult} />}

        <Group justify="space-between" mt="sm">
          <Button
            variant="light"
            leftSection={<ICONS.Test size="1rem" />}
            loading={testing}
            onClick={handleTest}
          >
            Test Connection
          </Button>
          <Group>
            <Button variant="subtle" onClick={onClose}>
              Cancel
            </Button>
            <Button
              leftSection={<ICONS.Save size="1rem" />}
              loading={creating || updating}
              onClick={handleSave}
            >
              {isEdit ? "Save Changes" : "Create Provider"}
            </Button>
          </Group>
        </Group>
      </Stack>
    </Modal>
  );
}

function SecretProviderGrantsModal({
  provider,
  onClose,
}: {
  provider: Types.SecretProvider | null;
  onClose: () => void;
}) {
  const providerId = provider?._id?.$oid ?? "";
  const opened = !!provider && !!providerId;
  const inv = useInvalidate();

  const grants =
    useRead(
      "ListSecretProviderGrants",
      { provider_id: providerId },
      { enabled: opened },
    ).data ?? [];

  const users = useRead("ListUsers", {}, { enabled: opened }).data ?? [];
  const groups = useRead("ListUserGroups", {}, { enabled: opened }).data ?? [];
  const stacks =
    useRead("ListStacks", { limit: 0 }, { enabled: opened }).data ?? [];
  const deployments =
    useRead("ListDeployments", { limit: 0 }, { enabled: opened }).data ?? [];
  const builds =
    useRead("ListBuilds", { limit: 0 }, { enabled: opened }).data ?? [];

  const [editingGrant, setEditingGrant] =
    useState<Types.SecretProviderGrant | null>(null);
  const [grantName, setGrantName] = useState("");
  const [grantDescription, setGrantDescription] = useState("");
  const [grantEnabled, setGrantEnabled] = useState(true);

  const [subjectType, setSubjectType] = useState<
    Types.SecretGrantSubject["type"]
  >("ResourceAuthorized");
  const [subjectUserId, setSubjectUserId] = useState("");
  const [subjectGroupId, setSubjectGroupId] = useState("");
  const [subjectActor, setSubjectActor] = useState<Types.SecretExecutionActor>(
    Types.SecretExecutionActor.Procedure,
  );
  const [subjectActorResourceId, setSubjectActorResourceId] = useState("");

  const [allStacks, setAllStacks] = useState(true);
  const [allDeployments, setAllDeployments] = useState(true);
  const [allBuilds, setAllBuilds] = useState(true);
  const [selectedStacks, setSelectedStacks] = useState<string[]>([]);
  const [selectedDeployments, setSelectedDeployments] = useState<string[]>([]);
  const [selectedBuilds, setSelectedBuilds] = useState<string[]>([]);

  const [projectIds, setProjectIds] = useState<string[]>([]);
  const [environments, setEnvironments] = useState<string[]>(["prod"]);
  const [pathPrefixes, setPathPrefixes] = useState<string[]>(["/"]);
  const [allowImports, setAllowImports] = useState(false);
  const [allowExpandRefs, setAllowExpandRefs] = useState(false);
  const [allowedUsages, setAllowedUsages] =
    useState<Types.SecretUsage[]>(ALL_SECRET_USAGES);
  const [grantError, setGrantError] = useState<string | null>(null);

  const resetGrantForm = (g?: Types.SecretProviderGrant | null) => {
    setGrantError(null);
    if (!g) {
      setEditingGrant(null);
      setGrantName("");
      setGrantDescription("");
      setGrantEnabled(true);
      setSubjectType("ResourceAuthorized");
      setSubjectUserId("");
      setSubjectGroupId("");
      setSubjectActor(Types.SecretExecutionActor.Procedure);
      setSubjectActorResourceId("");
      setAllStacks(true);
      setAllDeployments(true);
      setAllBuilds(true);
      setSelectedStacks([]);
      setSelectedDeployments([]);
      setSelectedBuilds([]);
      const def = provider?.config.params.default_selector;
      setProjectIds(def?.project_id ? [def.project_id] : []);
      setEnvironments(def?.environment ? [def.environment] : ["prod"]);
      setPathPrefixes([def?.secret_path || "/"]);
      setAllowImports(!!def?.include_imports);
      setAllowExpandRefs(!!def?.expand_references);
      setAllowedUsages(ALL_SECRET_USAGES);
      return;
    }

    setEditingGrant(g);
    setGrantName(g.name ?? "");
    setGrantDescription(g.description ?? "");
    setGrantEnabled(g.enabled);
    setSubjectType(g.subject.type);
    if (g.subject.type === "User") setSubjectUserId(g.subject.params.user_id);
    if (g.subject.type === "UserGroup")
      setSubjectGroupId(g.subject.params.group_id);
    if (g.subject.type === "SystemActor") {
      setSubjectActor(g.subject.params.actor);
      setSubjectActorResourceId(g.subject.params.actor_resource_id ?? "");
    }
    const scopes = g.allowed_resources ?? [];
    setAllStacks(scopes.some((s) => s.type === "AllStacks"));
    setAllDeployments(scopes.some((s) => s.type === "AllDeployments"));
    setAllBuilds(scopes.some((s) => s.type === "AllBuilds"));
    setSelectedStacks(
      scopes.flatMap((s) => (s.type === "Stack" ? [s.params.id] : [])),
    );
    setSelectedDeployments(
      scopes.flatMap((s) => (s.type === "Deployment" ? [s.params.id] : [])),
    );
    setSelectedBuilds(
      scopes.flatMap((s) => (s.type === "Build" ? [s.params.id] : [])),
    );
    setProjectIds(g.selector_scope.params.project_ids ?? []);
    setEnvironments(g.selector_scope.params.environments ?? []);
    setPathPrefixes(g.selector_scope.params.path_prefixes ?? ["/"]);
    setAllowImports(!!g.selector_scope.params.allow_imports);
    setAllowExpandRefs(!!g.selector_scope.params.allow_expand_references);
    setAllowedUsages(g.allowed_usages ?? []);
  };

  useEffect(() => {
    if (opened) resetGrantForm(null);
  }, [opened, providerId]);

  const invalidateGrants = () => {
    inv(["ListSecretProviderGrants"], ["ListAvailableSecretProviders"]);
  };

  const { mutate: createGrant, isPending: creatingGrant } = useWrite(
    "CreateSecretProviderGrant",
    {
      onSuccess: () => {
        invalidateGrants();
        resetGrantForm(null);
        notifications.show({ message: "Grant created" });
      },
    },
  );

  const { mutate: updateGrant, isPending: updatingGrant } = useWrite(
    "UpdateSecretProviderGrant",
    {
      onSuccess: () => {
        invalidateGrants();
        resetGrantForm(null);
        notifications.show({ message: "Grant updated" });
      },
    },
  );

  const { mutate: deleteGrant } = useWrite("DeleteSecretProviderGrant", {
    onSuccess: () => {
      invalidateGrants();
      notifications.show({ message: "Grant deleted" });
    },
  });

  const submitGrant = () => {
    setGrantError(null);
    let subject: Types.SecretGrantSubject;
    if (subjectType === "ResourceAuthorized") {
      subject = { type: "ResourceAuthorized" };
    } else if (subjectType === "User") {
      if (!subjectUserId.trim()) {
        setGrantError("Select a User for the grant subject.");
        return;
      }
      subject = { type: "User", params: { user_id: subjectUserId.trim() } };
    } else if (subjectType === "UserGroup") {
      if (!subjectGroupId.trim()) {
        setGrantError("Select a User Group for the grant subject.");
        return;
      }
      subject = {
        type: "UserGroup",
        params: { group_id: subjectGroupId.trim() },
      };
    } else {
      subject = {
        type: "SystemActor",
        params: {
          actor: subjectActor,
          actor_resource_id: subjectActorResourceId.trim() || undefined,
        },
      };
    }

    const allowed_resources: Types.SecretResourceScope[] = [];
    if (allStacks) allowed_resources.push({ type: "AllStacks" });
    if (allDeployments) allowed_resources.push({ type: "AllDeployments" });
    if (allBuilds) allowed_resources.push({ type: "AllBuilds" });
    selectedStacks.forEach((id) =>
      allowed_resources.push({ type: "Stack", params: { id } }),
    );
    selectedDeployments.forEach((id) =>
      allowed_resources.push({ type: "Deployment", params: { id } }),
    );
    selectedBuilds.forEach((id) =>
      allowed_resources.push({ type: "Build", params: { id } }),
    );

    if (allowed_resources.length === 0) {
      setGrantError("Select at least one allowed resource scope.");
      return;
    }
    if (
      projectIds.length === 0 ||
      environments.length === 0 ||
      pathPrefixes.length === 0
    ) {
      setGrantError(
        "Infisical Selector Scope requires at least one Project ID, Environment, and Path Prefix.",
      );
      return;
    }
    if (allowedUsages.length === 0) {
      setGrantError("Select at least one allowed usage.");
      return;
    }

    const selector_scope: Types.SecretSelectorScope = {
      type: "Infisical",
      params: {
        project_ids: projectIds,
        environments,
        path_prefixes: pathPrefixes,
        allow_imports: allowImports,
        allow_expand_references: allowExpandRefs,
      },
    };

    if (editingGrant && editingGrant._id?.$oid) {
      updateGrant({
        id: editingGrant._id.$oid,
        expected_revision: editingGrant.revision,
        name: grantName.trim(),
        description: grantDescription.trim(),
        enabled: grantEnabled,
        subject,
        allowed_resources,
        selector_scope,
        allowed_usages: allowedUsages,
      });
    } else {
      createGrant({
        provider_id: providerId,
        name: grantName.trim(),
        description: grantDescription.trim(),
        enabled: grantEnabled,
        subject,
        allowed_resources,
        selector_scope,
        allowed_usages: allowedUsages,
      });
    }
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Text fw={600} size="lg">
          Authorization Grants — {provider?.name || provider?.slug}
        </Text>
      }
      size="xl"
    >
      <Stack gap="md">
        <Alert color="blue" variant="light" icon={<ICONS.Info size="1rem" />}>
          External Secret Providers enforce default-deny RBAC. Interactive users
          and API keys match <Code>ResourceAuthorized</Code>,{" "}
          <Code>User</Code>, or <Code>UserGroup</Code> grants; automated
          Procedures, Webhooks, Schedules, AutoRedeploy, ResourceSync, and
          Actions require an explicit <Code>SystemActor</Code> grant.
        </Alert>

        {grants.length > 0 && (
          <Table withTableBorder withRowBorders>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Name / Subject</Table.Th>
                <Table.Th>Infisical Scope</Table.Th>
                <Table.Th>Usages</Table.Th>
                <Table.Th>Enabled</Table.Th>
                <Table.Th>Actions</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {grants.map((g) => {
                const gid = g._id?.$oid ?? "";
                const scope = g.selector_scope.params;
                return (
                  <Table.Tr key={gid}>
                    <Table.Td>
                      <Stack gap={2}>
                        <Text fw={600} fz="sm">
                          {g.name || "(Unnamed Grant)"}
                        </Text>
                        <Code fz="xs">
                          {g.subject.type}
                          {g.subject.type === "SystemActor"
                            ? `:${g.subject.params.actor}`
                            : ""}
                        </Code>
                      </Stack>
                    </Table.Td>
                    <Table.Td fz="xs">
                      <div>Projects: {(scope.project_ids ?? []).join(", ")}</div>
                      <div>Envs: {(scope.environments ?? []).join(", ")}</div>
                      <div>Paths: {(scope.path_prefixes ?? []).join(", ")}</div>
                    </Table.Td>
                    <Table.Td>
                      <Group gap={4}>
                        {(g.allowed_usages ?? []).map((u) => (
                          <Badge key={u} size="xs" variant="outline">
                            {u}
                          </Badge>
                        ))}
                      </Group>
                    </Table.Td>
                    <Table.Td>
                      <Switch
                        size="xs"
                        checked={g.enabled}
                        onChange={(e) =>
                          gid &&
                          updateGrant({
                            id: gid,
                            expected_revision: g.revision,
                            enabled: e.currentTarget.checked,
                          })
                        }
                      />
                    </Table.Td>
                    <Table.Td>
                      <Group gap="xs" wrap="nowrap">
                        <Button
                          size="compact-xs"
                          variant="default"
                          onClick={() => resetGrantForm(g)}
                        >
                          Edit
                        </Button>
                        <ConfirmButton
                          size="compact-xs"
                          icon={<ICONS.Delete size="0.8rem" />}
                          confirmProps={{ color: "red", variant: "filled" }}
                          onClick={() => gid && deleteGrant({ id: gid })}
                        >
                          Delete
                        </ConfirmButton>
                      </Group>
                    </Table.Td>
                  </Table.Tr>
                );
              })}
            </Table.Tbody>
          </Table>
        )}

        <Divider
          label={editingGrant ? "Edit Selected Grant" : "Create New Grant"}
        />

        {grantError && (
          <Alert color="red" icon={<ICONS.Alert size="1rem" />}>
            {grantError}
          </Alert>
        )}

        <SimpleGrid cols={{ base: 1, sm: 2 }}>
          <TextInput
            label="Grant Name"
            placeholder="Prod Compose Deploys"
            value={grantName}
            onChange={(e) => setGrantName(e.currentTarget.value)}
          />
          <Select
            label="Subject Type"
            value={subjectType}
            onChange={(v) =>
              v && setSubjectType(v as Types.SecretGrantSubject["type"])
            }
            data={[
              {
                label:
                  "ResourceAuthorized (Any user with resource permission)",
                value: "ResourceAuthorized",
              },
              { label: "Specific User / Service User", value: "User" },
              { label: "User Group", value: "UserGroup" },
              {
                label:
                  "System Actor (Procedure / Webhook / Schedule / AutoRedeploy / Sync)",
                value: "SystemActor",
              },
            ]}
          />
        </SimpleGrid>

        {subjectType === "User" && (
          <Select
            label="User"
            placeholder="Select user"
            searchable
            value={subjectUserId}
            onChange={(v) => setSubjectUserId(v ?? "")}
            data={users
              .filter((u) => !!u._id?.$oid)
              .map((u) => ({
                label: `${u.username} (${u._id?.$oid})`,
                value: u._id!.$oid,
              }))}
          />
        )}

        {subjectType === "UserGroup" && (
          <Select
            label="User Group"
            placeholder="Select user group or type 'everyone'"
            searchable
            value={subjectGroupId}
            onChange={(v) => setSubjectGroupId(v ?? "")}
            data={[
              { label: "everyone (Built-in group)", value: "everyone" },
              ...groups
                .filter((g) => !!g._id?.$oid)
                .map((g) => ({
                  label: `${g.name} (${g._id?.$oid})`,
                  value: g._id!.$oid,
                })),
            ]}
          />
        )}

        {subjectType === "SystemActor" && (
          <SimpleGrid cols={{ base: 1, sm: 2 }}>
            <Select
              label="Automated Execution Actor"
              value={subjectActor}
              onChange={(v) =>
                v && setSubjectActor(v as Types.SecretExecutionActor)
              }
              data={ALL_EXECUTION_ACTORS}
            />
            <TextInput
              label="Restrict to Trigger Resource ID (Optional)"
              placeholder="Optional Procedure / ResourceSync / Action ID"
              value={subjectActorResourceId}
              onChange={(e) => setSubjectActorResourceId(e.currentTarget.value)}
            />
          </SimpleGrid>
        )}

        <Text fw={500} fz="sm">
          Allowed Komodo Resources
        </Text>
        <Group gap="lg">
          <Switch
            label="All Stacks"
            checked={allStacks}
            onChange={(e) => setAllStacks(e.currentTarget.checked)}
          />
          <Switch
            label="All Deployments"
            checked={allDeployments}
            onChange={(e) => setAllDeployments(e.currentTarget.checked)}
          />
          <Switch
            label="All Builds"
            checked={allBuilds}
            onChange={(e) => setAllBuilds(e.currentTarget.checked)}
          />
        </Group>

        {(!allStacks || !allDeployments || !allBuilds) && (
          <SimpleGrid cols={{ base: 1, sm: 3 }}>
            {!allStacks && (
              <MultiSelect
                label="Specific Stacks"
                searchable
                value={selectedStacks}
                onChange={setSelectedStacks}
                data={stacks.map((s) => ({ label: s.name, value: s.id }))}
              />
            )}
            {!allDeployments && (
              <MultiSelect
                label="Specific Deployments"
                searchable
                value={selectedDeployments}
                onChange={setSelectedDeployments}
                data={deployments.map((d) => ({ label: d.name, value: d.id }))}
              />
            )}
            {!allBuilds && (
              <MultiSelect
                label="Specific Builds"
                searchable
                value={selectedBuilds}
                onChange={setSelectedBuilds}
                data={builds.map((b) => ({ label: b.name, value: b.id }))}
              />
            )}
          </SimpleGrid>
        )}

        <SimpleGrid cols={{ base: 1, sm: 3 }}>
          <TagsInput
            label="Allowed Infisical Project IDs"
            placeholder="Press Enter to add projectId"
            value={projectIds}
            onChange={setProjectIds}
          />
          <TagsInput
            label="Allowed Environment Slugs"
            placeholder="prod, staging"
            value={environments}
            onChange={setEnvironments}
          />
          <TagsInput
            label="Allowed Path Prefixes"
            placeholder="/, /backend"
            value={pathPrefixes}
            onChange={setPathPrefixes}
          />
        </SimpleGrid>

        <Group gap="xl">
          <Switch
            label="Allow Imports"
            checked={allowImports}
            onChange={(e) => setAllowImports(e.currentTarget.checked)}
          />
          <Switch
            label="Allow Expand Secret References"
            checked={allowExpandRefs}
            onChange={(e) => setAllowExpandRefs(e.currentTarget.checked)}
          />
          <Switch
            label="Grant Enabled"
            checked={grantEnabled}
            onChange={(e) => setGrantEnabled(e.currentTarget.checked)}
          />
        </Group>

        {(allowImports || allowExpandRefs) && (
          <Alert color="yellow" icon={<ICONS.Alert size="1rem" />}>
            Imports and secret reference expansion may resolve upstream secrets
            outside the listed path prefixes. Ensure the Infisical Machine
            Identity itself enforces least-privilege folder/environment access.
          </Alert>
        )}

        <MultiSelect
          label="Allowed Usages"
          value={allowedUsages}
          onChange={(vals) => setAllowedUsages(vals as Types.SecretUsage[])}
          data={ALL_SECRET_USAGES}
        />

        <Group justify="end">
          {editingGrant && (
            <Button variant="subtle" onClick={() => resetGrantForm(null)}>
              Cancel Edit
            </Button>
          )}
          <Button
            leftSection={<ICONS.Save size="1rem" />}
            loading={creatingGrant || updatingGrant}
            onClick={submitGrant}
          >
            {editingGrant ? "Update Grant" : "Create Grant"}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}

function SecretProviderUsagesModal({
  provider,
  onClose,
  onDeleted,
}: {
  provider: Types.SecretProvider | null;
  onClose: () => void;
  onDeleted: () => void;
}) {
  const providerId = provider?._id?.$oid ?? "";
  const opened = !!provider && !!providerId;
  const usages =
    useRead(
      "ListSecretProviderUsages",
      { provider_id: providerId },
      { enabled: opened },
    ).data ?? [];

  const { mutate: deleteProvider, isPending: deleting } = useWrite(
    "DeleteSecretProvider",
    {
      onSuccess: () => {
        notifications.show({ message: "Secret provider deleted" });
        onDeleted();
      },
    },
  );

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Text fw={600} size="lg">
          Resource Usages & Delete — {provider?.name || provider?.slug}
        </Text>
      }
      size="lg"
    >
      <Stack gap="md">
        {usages.length === 0 ? (
          <Alert color="green" icon={<ICONS.Check size="1rem" />}>
            No Stacks, Deployments, or Builds currently reference{" "}
            <Code>{provider?.slug}</Code>. It can be safely deleted.
          </Alert>
        ) : (
          <>
            <Alert color="orange" icon={<ICONS.Alert size="1rem" />}>
              This provider is referenced by <b>{usages.length}</b> resource(s).
              Remove all references before deleting the provider, or disable the
              provider temporarily.
            </Alert>
            <Table withTableBorder withRowBorders>
              <Table.Thead>
                <Table.Tr>
                  <Table.Th>Resource</Table.Th>
                  <Table.Th>Reference Types</Table.Th>
                  <Table.Th>Referenced Keys</Table.Th>
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {usages.map((u) => (
                  <Table.Tr key={`${u.resource_type}:${u.resource_id}`}>
                    <Table.Td>
                      <ResourceLink
                        type={u.resource_type as UsableResource}
                        id={u.resource_id}
                      />
                    </Table.Td>
                    <Table.Td>
                      <Group gap={4}>
                        {u.reference_kinds.map((k) => (
                          <Badge key={k} size="xs" variant="light">
                            {k}
                          </Badge>
                        ))}
                      </Group>
                    </Table.Td>
                    <Table.Td>
                      <Group gap={4}>
                        {u.keys.map((k) => (
                          <Code key={k} fz="xs">
                            {k}
                          </Code>
                        ))}
                      </Group>
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </>
        )}

        <Group justify="space-between">
          <Button variant="subtle" onClick={onClose}>
            Close
          </Button>
          <ConfirmButton
            icon={<ICONS.Delete size="1rem" />}
            disabled={usages.length > 0 || !providerId}
            loading={deleting}
            confirmProps={{ variant: "filled", color: "red" }}
            onClick={() => providerId && deleteProvider({ id: providerId })}
          >
            Delete Secret Provider
          </ConfirmButton>
        </Group>
      </Stack>
    </Modal>
  );
}
