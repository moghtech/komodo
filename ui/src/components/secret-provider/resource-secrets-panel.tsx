import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import {
  hasExternalSecretsInDeployment,
  validateStackSecretBinding,
} from "@/components/secret-provider/utils";
import { StackSecretWarningsBanner } from "@/components/secret-provider/views";
import {
  ActionIcon,
  Alert,
  Badge,
  Button,
  Card,
  Code,
  Group,
  Select,
  SimpleGrid,
  Stack,
  Switch,
  Table,
  TagsInput,
  Text,
  TextInput,
} from "@mantine/core";
import { Types } from "komodo_client";
import { useState } from "react";

function SecretSourcesEditor({
  target,
  sources,
  onChange,
  disabled,
}: {
  target: Types.ResourceTarget;
  sources: Types.SecretSource[];
  onChange: (sources: Types.SecretSource[]) => void;
  disabled?: boolean;
}) {
  const providers =
    useRead("ListAvailableSecretProviders", { target }).data ?? [];

  const addSource = () => {
    const first = providers[0];
    const def = first?.default_selector?.params;
    onChange([
      ...sources,
      {
        name: first ? `${first.slug}-src` : "default",
        provider: first?.slug ?? "",
        selector: {
          type: "Infisical",
          params: {
            project_id: def?.project_id ?? "",
            environment: def?.environment ?? "prod",
            secret_path: def?.secret_path || "/",
            include_imports: def?.include_imports ?? false,
            expand_references: def?.expand_references ?? false,
          },
        },
      },
    ]);
  };

  const updateAt = (idx: number, patch: Partial<Types.SecretSource>) => {
    onChange(sources.map((s, i) => (i === idx ? { ...s, ...patch } : s)));
  };

  const updateSelectorAt = (
    idx: number,
    patch: Partial<Types.InfisicalSecretSelector>,
  ) => {
    onChange(
      sources.map((s, i) =>
        i === idx
          ? {
              ...s,
              selector: {
                type: "Infisical",
                params: { ...s.selector.params, ...patch },
              },
            }
          : s,
      ),
    );
  };

  return (
    <Stack gap="xs">
      <Group justify="space-between">
        <div>
          <Text fw={600} fz="sm">
            Secret Sources (Optional Scope Overrides)
          </Text>
          <Text fz="xs" c="dimmed">
            Define named Infisical Project / Environment / Path scopes for
            bindings or to override a provider&apos;s default selector.
          </Text>
        </div>
        {!disabled && (
          <Button
            size="xs"
            variant="light"
            leftSection={<ICONS.Add size="0.9rem" />}
            onClick={addSource}
          >
            Add Secret Source
          </Button>
        )}
      </Group>

      {sources.map((src, idx) => {
        const sel = src.selector.params;
        return (
          <Card key={idx} withBorder padding="sm" radius="md">
            <Stack gap="xs">
              <Group justify="space-between">
                <Badge variant="light">Source #{idx + 1}</Badge>
                {!disabled && (
                  <ActionIcon
                    color="red"
                    variant="subtle"
                    onClick={() =>
                      onChange(sources.filter((_, i) => i !== idx))
                    }
                  >
                    <ICONS.Remove size="1rem" />
                  </ActionIcon>
                )}
              </Group>
              <SimpleGrid cols={{ base: 1, sm: 5 }}>
                <TextInput
                  label="Source Alias"
                  placeholder="Optional alias (defaults to provider slug)"
                  value={src.name ?? ""}
                  disabled={disabled}
                  onChange={(e) =>
                    updateAt(idx, { name: e.currentTarget.value })
                  }
                />
                {providers.length > 0 ? (
                  <Select
                    label="Provider Slug"
                    value={src.provider}
                    disabled={disabled}
                    searchable
                    onChange={(v) => v && updateAt(idx, { provider: v })}
                    data={providers.map((p) => ({
                      label: `${p.name} (${p.slug})`,
                      value: p.slug,
                    }))}
                  />
                ) : (
                  <TextInput
                    label="Provider Slug"
                    placeholder="infisical-prod"
                    value={src.provider}
                    disabled={disabled}
                    onChange={(e) =>
                      updateAt(idx, { provider: e.currentTarget.value })
                    }
                  />
                )}
                <TextInput
                  label="Infisical Project ID"
                  placeholder="proj_..."
                  value={sel.project_id}
                  disabled={disabled}
                  onChange={(e) =>
                    updateSelectorAt(idx, { project_id: e.currentTarget.value })
                  }
                />
                <TextInput
                  label="Environment"
                  placeholder="prod"
                  value={sel.environment}
                  disabled={disabled}
                  onChange={(e) =>
                    updateSelectorAt(idx, {
                      environment: e.currentTarget.value,
                    })
                  }
                />
                <TextInput
                  label="Secret Path"
                  placeholder="/"
                  value={sel.secret_path}
                  disabled={disabled}
                  onChange={(e) =>
                    updateSelectorAt(idx, {
                      secret_path: e.currentTarget.value,
                    })
                  }
                />
              </SimpleGrid>
              <Group gap="xl">
                <Switch
                  size="xs"
                  label="Include Imports"
                  checked={!!sel.include_imports}
                  disabled={disabled}
                  onChange={(e) =>
                    updateSelectorAt(idx, {
                      include_imports: e.currentTarget.checked,
                    })
                  }
                />
                <Switch
                  size="xs"
                  label="Expand Secret References"
                  checked={!!sel.expand_references}
                  disabled={disabled}
                  onChange={(e) =>
                    updateSelectorAt(idx, {
                      expand_references: e.currentTarget.checked,
                    })
                  }
                />
              </Group>
            </Stack>
          </Card>
        );
      })}
    </Stack>
  );
}

export function ResourceSecretsPreviewPanel({
  target,
}: {
  target: Types.ResourceTarget;
}) {
  const [requested, setRequested] = useState(false);
  const previewQuery = useRead(
    "PreviewResourceSecrets",
    { target },
    { enabled: requested },
  );

  return (
    <Card withBorder padding="sm" radius="md">
      <Stack gap="xs">
        <Group justify="space-between">
          <div>
            <Text fw={600} fz="sm">
              Secret Binding &amp; Provenance Preview (Zero Plaintext)
            </Text>
            <Text fz="xs" c="dimmed">
              Verifies saved external secret references, RBAC grants, and last
              deployed metadata without exposing secret values.
            </Text>
          </div>
          <Button
            size="xs"
            variant="light"
            leftSection={<ICONS.Inspect size="0.9rem" />}
            loading={previewQuery.isFetching}
            onClick={() => {
              if (!requested) {
                setRequested(true);
              } else {
                previewQuery.refetch();
              }
            }}
          >
            {requested ? "Refresh Preview" : "Preview Saved Secrets"}
          </Button>
        </Group>

        {previewQuery.data && (
          <Stack gap="xs">
            <Group gap="xs">
              <Badge color={previewQuery.data.valid ? "green" : "red"}>
                {previewQuery.data.valid ? "Valid" : "Issues Detected"}
              </Badge>
              {previewQuery.data.last_deployed_at && (
                <Text fz="xs" c="dimmed">
                  Last deployed provenance:{" "}
                  {new Date(
                    previewQuery.data.last_deployed_at,
                  ).toLocaleString()}
                </Text>
              )}
            </Group>

            {previewQuery.data.errors.map((err, i) => (
              <Alert key={i} color="red" icon={<ICONS.Alert size="0.9rem" />}>
                {err}
              </Alert>
            ))}

            {previewQuery.data.items.length > 0 && (
              <Table withTableBorder withRowBorders>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Target</Table.Th>
                    <Table.Th>External References</Table.Th>
                    <Table.Th>Status</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {previewQuery.data.items.map((item, i) => (
                    <Table.Tr key={`${item.target}:${i}`}>
                      <Table.Td>
                        <Code fz="xs">{item.target}</Code>
                      </Table.Td>
                      <Table.Td>
                        <Group gap={4}>
                          {item.references.map((ref) => (
                            <Code key={ref} fz="xs">
                              {ref}
                            </Code>
                          ))}
                        </Group>
                      </Table.Td>
                      <Table.Td>
                        <Group gap={6}>
                          <Badge size="xs" color={item.ok ? "green" : "red"}>
                            {item.status}
                          </Badge>
                          {item.version !== undefined && (
                            <Badge size="xs" variant="outline">
                              v{item.version}
                            </Badge>
                          )}
                        </Group>
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            )}
          </Stack>
        )}
      </Stack>
    </Card>
  );
}

export function StackExternalSecretsPanel({
  stackId,
  config,
  allServices,
  swarmId,
  disabled,
  onChange,
}: {
  stackId: string;
  config: Partial<Types.StackConfig>;
  allServices: string[];
  swarmId?: string;
  disabled?: boolean;
  onChange: (patch: Partial<Types.StackConfig>) => void;
}) {
  const target: Types.ResourceTarget = { type: "Stack", id: stackId };
  const sources = config.secret_sources ?? [];
  const bindings = config.secret_bindings ?? [];
  const providers =
    useRead("ListAvailableSecretProviders", { target }).data ?? [];

  const sourceOptions = Array.from(
    new Set([
      ...sources.map((s) => (s.name?.trim() ? s.name.trim() : s.provider)),
      ...providers.map((p) => p.slug),
    ]),
  ).filter(Boolean);

  const addBinding = () => {
    onChange({
      secret_bindings: [
        ...bindings,
        {
          source: sourceOptions[0] ?? "",
          remote_key: "",
          target: {
            type: "ComposeEnv",
            params: { variable: "" },
          },
          required: true,
          allow_empty: false,
          conflict_policy: Types.SecretConflictPolicy.ReplaceExisting,
        },
      ],
    });
  };

  const updateBinding = (
    idx: number,
    patch: Partial<Types.StackSecretBinding>,
  ) => {
    onChange({
      secret_bindings: bindings.map((b, i) =>
        i === idx ? { ...b, ...patch } : b,
      ),
    });
  };

  return (
    <Stack gap="md">
      <StackSecretWarningsBanner config={config} swarmId={swarmId} />

      <SecretSourcesEditor
        target={target}
        sources={sources}
        disabled={disabled}
        onChange={(secret_sources) => onChange({ secret_sources })}
      />

      <Stack gap="xs">
        <Group justify="space-between">
          <div>
            <Text fw={600} fz="sm">
              Structured Secret Bindings
            </Text>
            <Text fz="xs" c="dimmed">
              Bind remote keys into Stack <Code>.env</Code> (
              <Code>ComposeEnv</Code>), explicit service environments (
              <Code>ServiceEnv</Code>), or immutable <Code>0400</Code> Compose
              file secrets (<Code>ComposeFileSecret</Code>).
            </Text>
          </div>
          {!disabled && (
            <Button
              size="xs"
              variant="light"
              leftSection={<ICONS.Add size="0.9rem" />}
              onClick={addBinding}
            >
              Add Secret Binding
            </Button>
          )}
        </Group>

        {bindings.map((b, idx) => {
          const validationErr = validateStackSecretBinding(b);
          return (
            <Card key={idx} withBorder padding="sm" radius="md">
              <Stack gap="xs">
                <Group justify="space-between">
                  <Badge variant="light">Binding #{idx + 1}</Badge>
                  {!disabled && (
                    <ActionIcon
                      color="red"
                      variant="subtle"
                      onClick={() =>
                        onChange({
                          secret_bindings: bindings.filter((_, i) => i !== idx),
                        })
                      }
                    >
                      <ICONS.Remove size="1rem" />
                    </ActionIcon>
                  )}
                </Group>

                <SimpleGrid cols={{ base: 1, sm: 4 }}>
                  <TextInput
                    label="Source Alias / Provider Slug"
                    placeholder={sourceOptions[0] || "infisical-prod"}
                    value={b.source}
                    disabled={disabled}
                    onChange={(e) =>
                      updateBinding(idx, { source: e.currentTarget.value })
                    }
                  />
                  <TextInput
                    label="Remote Secret Key"
                    placeholder="DATABASE_URL"
                    value={b.remote_key}
                    disabled={disabled}
                    onChange={(e) =>
                      updateBinding(idx, {
                        remote_key: e.currentTarget.value,
                      })
                    }
                  />
                  <Select
                    label="Injection Scope"
                    value={b.target.type}
                    disabled={disabled}
                    onChange={(val) => {
                      if (!val) return;
                      if (val === "ComposeEnv") {
                        updateBinding(idx, {
                          target: {
                            type: "ComposeEnv",
                            params: { variable: b.remote_key || "" },
                          },
                        });
                      } else if (val === "ServiceEnv") {
                        updateBinding(idx, {
                          target: {
                            type: "ServiceEnv",
                            params: {
                              variable: b.remote_key || "",
                              services: allServices.slice(0, 1),
                            },
                          },
                        });
                      } else {
                        updateBinding(idx, {
                          target: {
                            type: "ComposeFileSecret",
                            params: {
                              secret_name: (b.remote_key || "app_secret")
                                .toLowerCase()
                                .replace(/[^a-z0-9_-]/g, "_"),
                              services: allServices.slice(0, 1),
                              mode: 0o400,
                            },
                          },
                        });
                      }
                    }}
                    data={[
                      {
                        label: "Compose .env Variable (ComposeEnv)",
                        value: "ComposeEnv",
                      },
                      {
                        label: "Service Container Env (ServiceEnv)",
                        value: "ServiceEnv",
                      },
                      {
                        label: "Compose File Secret 0400 (ComposeFileSecret)",
                        value: "ComposeFileSecret",
                      },
                    ]}
                  />
                  <Select
                    label="Conflict Policy"
                    value={
                      b.conflict_policy ??
                      Types.SecretConflictPolicy.ReplaceExisting
                    }
                    disabled={disabled}
                    onChange={(v) =>
                      v &&
                      updateBinding(idx, {
                        conflict_policy: v as Types.SecretConflictPolicy,
                      })
                    }
                    data={[
                      {
                        label: "Replace Existing",
                        value: Types.SecretConflictPolicy.ReplaceExisting,
                      },
                      {
                        label: "Fail on Duplicate",
                        value: Types.SecretConflictPolicy.Fail,
                      },
                    ]}
                  />
                </SimpleGrid>

                {b.target.type === "ComposeEnv" && (
                  <TextInput
                    label="Target .env Variable"
                    placeholder="DATABASE_URL"
                    value={b.target.params.variable}
                    disabled={disabled}
                    onChange={(e) =>
                      updateBinding(idx, {
                        target: {
                          type: "ComposeEnv",
                          params: { variable: e.currentTarget.value },
                        },
                      })
                    }
                  />
                )}

                {b.target.type === "ServiceEnv" && (
                  <SimpleGrid cols={{ base: 1, sm: 2 }}>
                    <TextInput
                      label="Container Environment Variable"
                      placeholder="DATABASE_URL"
                      value={b.target.params.variable}
                      disabled={disabled}
                      onChange={(e) => {
                        if (b.target.type !== "ServiceEnv") return;
                        updateBinding(idx, {
                          target: {
                            type: "ServiceEnv",
                            params: {
                              ...b.target.params,
                              variable: e.currentTarget.value,
                            },
                          },
                        });
                      }}
                    />
                    <TagsInput
                      label="Target Compose Services (Required)"
                      placeholder="Select or enter service names"
                      data={allServices}
                      value={b.target.params.services}
                      disabled={disabled}
                      onChange={(services) => {
                        if (b.target.type !== "ServiceEnv") return;
                        updateBinding(idx, {
                          target: {
                            type: "ServiceEnv",
                            params: { ...b.target.params, services },
                          },
                        });
                      }}
                    />
                  </SimpleGrid>
                )}

                {b.target.type === "ComposeFileSecret" && (
                  <SimpleGrid cols={{ base: 1, sm: 4 }}>
                    <TextInput
                      label="Compose Secret Name"
                      placeholder="db_password"
                      value={b.target.params.secret_name}
                      disabled={disabled}
                      onChange={(e) => {
                        if (b.target.type !== "ComposeFileSecret") return;
                        updateBinding(idx, {
                          target: {
                            type: "ComposeFileSecret",
                            params: {
                              ...b.target.params,
                              secret_name: e.currentTarget.value,
                            },
                          },
                        });
                      }}
                    />
                    <TagsInput
                      label="Target Services (Required)"
                      placeholder="api, worker"
                      data={allServices}
                      value={b.target.params.services}
                      disabled={disabled}
                      onChange={(services) => {
                        if (b.target.type !== "ComposeFileSecret") return;
                        updateBinding(idx, {
                          target: {
                            type: "ComposeFileSecret",
                            params: { ...b.target.params, services },
                          },
                        });
                      }}
                    />
                    <TextInput
                      label="Container Target Path (Optional)"
                      placeholder="/run/secrets/<secret_name>"
                      value={b.target.params.target_path ?? ""}
                      disabled={disabled}
                      onChange={(e) => {
                        if (b.target.type !== "ComposeFileSecret") return;
                        updateBinding(idx, {
                          target: {
                            type: "ComposeFileSecret",
                            params: {
                              ...b.target.params,
                              target_path:
                                e.currentTarget.value.trim() || undefined,
                            },
                          },
                        });
                      }}
                    />
                    <TextInput
                      label="Octal Mode (Default 0400)"
                      placeholder="0400"
                      value={
                        b.target.params.mode !== undefined
                          ? `0${b.target.params.mode.toString(8)}`
                          : "0400"
                      }
                      disabled={disabled}
                      onChange={(e) => {
                        if (b.target.type !== "ComposeFileSecret") return;
                        const parsed = parseInt(
                          e.currentTarget.value.trim(),
                          8,
                        );
                        updateBinding(idx, {
                          target: {
                            type: "ComposeFileSecret",
                            params: {
                              ...b.target.params,
                              mode: Number.isNaN(parsed) ? undefined : parsed,
                            },
                          },
                        });
                      }}
                    />
                  </SimpleGrid>
                )}

                <Group gap="xl">
                  <Switch
                    size="xs"
                    label="Required"
                    checked={b.required}
                    disabled={disabled}
                    onChange={(e) =>
                      updateBinding(idx, { required: e.currentTarget.checked })
                    }
                  />
                  <Switch
                    size="xs"
                    label="Allow Empty Value"
                    checked={b.allow_empty}
                    disabled={disabled}
                    onChange={(e) =>
                      updateBinding(idx, {
                        allow_empty: e.currentTarget.checked,
                      })
                    }
                  />
                </Group>

                {validationErr && (
                  <Text fz="xs" c="red">
                    {validationErr}
                  </Text>
                )}
              </Stack>
            </Card>
          );
        })}
      </Stack>

      <ResourceSecretsPreviewPanel target={target} />
    </Stack>
  );
}

export function DeploymentExternalSecretsPanel({
  deploymentId,
  config,
  swarmId,
  disabled,
  onChange,
}: {
  deploymentId: string;
  config: Partial<Types.DeploymentConfig>;
  swarmId?: string;
  disabled?: boolean;
  onChange: (patch: Partial<Types.DeploymentConfig>) => void;
}) {
  const target: Types.ResourceTarget = {
    type: "Deployment",
    id: deploymentId,
  };
  const sources = config.secret_sources ?? [];
  const bindings = config.secret_bindings ?? [];
  const providers =
    useRead("ListAvailableSecretProviders", { target }).data ?? [];
  const sourceOptions = Array.from(
    new Set([
      ...sources.map((s) => (s.name?.trim() ? s.name.trim() : s.provider)),
      ...providers.map((p) => p.slug),
    ]),
  ).filter(Boolean);

  return (
    <Stack gap="md">
      {swarmId && hasExternalSecretsInDeployment(config) && (
        <Alert color="red" icon={<ICONS.Alert size="1rem" />}>
          Docker Swarm native external secrets are not supported in V1. Deploys
          on Swarm with external secrets will fail closed.
        </Alert>
      )}

      <Alert color="blue" variant="light" icon={<ICONS.Info size="1rem" />}>
        <Text fz="xs">
          External secret values are passed directly into container environment
          variables via shell-safe argument escaping and masked across{" "}
          <Code>Inspect</Code> and log APIs. Prefer whole-value references (e.g.{" "}
          <Code>DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]</Code>).
        </Text>
      </Alert>

      <SecretSourcesEditor
        target={target}
        sources={sources}
        disabled={disabled}
        onChange={(secret_sources) => onChange({ secret_sources })}
      />

      <Stack gap="xs">
        <Group justify="space-between">
          <div>
            <Text fw={600} fz="sm">
              Container Environment Secret Bindings
            </Text>
            <Text fz="xs" c="dimmed">
              Bind remote Infisical keys to container environment variables
              without embedding tokens in text.
            </Text>
          </div>
          {!disabled && (
            <Button
              size="xs"
              variant="light"
              leftSection={<ICONS.Add size="0.9rem" />}
              onClick={() =>
                onChange({
                  secret_bindings: [
                    ...bindings,
                    {
                      source: sourceOptions[0] ?? "",
                      remote_key: "",
                      variable: "",
                      required: true,
                      allow_empty: false,
                      conflict_policy:
                        Types.SecretConflictPolicy.ReplaceExisting,
                    },
                  ],
                })
              }
            >
              Add Container Binding
            </Button>
          )}
        </Group>

        {bindings.map((b, idx) => (
          <Card key={idx} withBorder padding="sm" radius="md">
            <Stack gap="xs">
              <Group justify="space-between">
                <Badge variant="light">Binding #{idx + 1}</Badge>
                {!disabled && (
                  <ActionIcon
                    color="red"
                    variant="subtle"
                    onClick={() =>
                      onChange({
                        secret_bindings: bindings.filter((_, i) => i !== idx),
                      })
                    }
                  >
                    <ICONS.Remove size="1rem" />
                  </ActionIcon>
                )}
              </Group>
              <SimpleGrid cols={{ base: 1, sm: 4 }}>
                <TextInput
                  label="Source Alias / Provider Slug"
                  placeholder={sourceOptions[0] || "infisical-prod"}
                  value={b.source}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, source: e.currentTarget.value }
                          : item,
                      ),
                    })
                  }
                />
                <TextInput
                  label="Remote Secret Key"
                  placeholder="DATABASE_URL"
                  value={b.remote_key}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, remote_key: e.currentTarget.value }
                          : item,
                      ),
                    })
                  }
                />
                <TextInput
                  label="Container Variable"
                  placeholder="DATABASE_URL"
                  value={b.variable}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, variable: e.currentTarget.value }
                          : item,
                      ),
                    })
                  }
                />
                <Select
                  label="Conflict Policy"
                  value={
                    b.conflict_policy ??
                    Types.SecretConflictPolicy.ReplaceExisting
                  }
                  disabled={disabled}
                  onChange={(v) =>
                    v &&
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? {
                              ...item,
                              conflict_policy: v as Types.SecretConflictPolicy,
                            }
                          : item,
                      ),
                    })
                  }
                  data={[
                    {
                      label: "Replace Existing",
                      value: Types.SecretConflictPolicy.ReplaceExisting,
                    },
                    {
                      label: "Fail on Duplicate",
                      value: Types.SecretConflictPolicy.Fail,
                    },
                  ]}
                />
              </SimpleGrid>
              <Group gap="xl">
                <Switch
                  size="xs"
                  label="Required"
                  checked={b.required}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, required: e.currentTarget.checked }
                          : item,
                      ),
                    })
                  }
                />
                <Switch
                  size="xs"
                  label="Allow Empty Value"
                  checked={b.allow_empty}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, allow_empty: e.currentTarget.checked }
                          : item,
                      ),
                    })
                  }
                />
              </Group>
            </Stack>
          </Card>
        ))}
      </Stack>

      <ResourceSecretsPreviewPanel target={target} />
    </Stack>
  );
}

export function BuildExternalSecretsPanel({
  buildId,
  config,
  disabled,
  onChange,
}: {
  buildId: string;
  config: Partial<Types.BuildConfig>;
  disabled?: boolean;
  onChange: (patch: Partial<Types.BuildConfig>) => void;
}) {
  const target: Types.ResourceTarget = { type: "Build", id: buildId };
  const sources = config.secret_sources ?? [];
  const bindings = config.secret_bindings ?? [];
  const providers =
    useRead("ListAvailableSecretProviders", { target }).data ?? [];
  const sourceOptions = Array.from(
    new Set([
      ...sources.map((s) => (s.name?.trim() ? s.name.trim() : s.provider)),
      ...providers.map((p) => p.slug),
    ]),
  ).filter(Boolean);

  return (
    <Stack gap="md">
      <Alert color="blue" variant="light" icon={<ICONS.Info size="1rem" />}>
        <Stack gap={4}>
          <Text fz="xs">
            <b>BuildKit Secret Mounts Only:</b> External build secrets are
            written to an isolated <Code>0700</Code>/<Code>0600</Code> temporary
            directory outside the Docker build context and passed via{" "}
            <Code>--secret id=&lt;secret_id&gt;,src=...</Code> (cleaned up
            immediately after build). Consume them in your Dockerfile with{" "}
            <Code>RUN --mount=type=secret,id=&lt;secret_id&gt;</Code>.
          </Text>
          <Text fz="xs">
            <b>Layer Cache Warning:</b> Never copy a mounted secret into a
            persisted image file, <Code>ARG</Code>, or <Code>ENV</Code>, or it
            will remain in the pushed image layer. Enable{" "}
            <b>Bust Build Cache on Secret Rotation</b> below if a secret-mounted{" "}
            <Code>RUN</Code> step must re-run whenever rotated secrets change.
          </Text>
        </Stack>
      </Alert>

      <Switch
        label="Bust Build Cache on Secret Rotation (KOMODO_SECRET_CACHE_NONCE)"
        description="Passes a non-sensitive keyed digest nonce as --build-arg KOMODO_SECRET_CACHE_NONCE=<nonce> when external secrets are present."
        checked={!!config.secret_cache_bust}
        disabled={disabled}
        onChange={(e) =>
          onChange({ secret_cache_bust: e.currentTarget.checked })
        }
      />

      <SecretSourcesEditor
        target={target}
        sources={sources}
        disabled={disabled}
        onChange={(secret_sources) => onChange({ secret_sources })}
      />

      <Stack gap="xs">
        <Group justify="space-between">
          <div>
            <Text fw={600} fz="sm">
              BuildKit Secret Bindings
            </Text>
            <Text fz="xs" c="dimmed">
              Bind remote Infisical keys to BuildKit <Code>--secret</Code> IDs.
            </Text>
          </div>
          {!disabled && (
            <Button
              size="xs"
              variant="light"
              leftSection={<ICONS.Add size="0.9rem" />}
              onClick={() =>
                onChange({
                  secret_bindings: [
                    ...bindings,
                    {
                      source: sourceOptions[0] ?? "",
                      remote_key: "",
                      secret_id: "",
                      format: Types.SecretValueFormat.Text,
                      required: true,
                      allow_empty: false,
                      conflict_policy:
                        Types.SecretConflictPolicy.ReplaceExisting,
                    },
                  ],
                })
              }
            >
              Add BuildKit Secret
            </Button>
          )}
        </Group>

        {bindings.map((b, idx) => (
          <Card key={idx} withBorder padding="sm" radius="md">
            <Stack gap="xs">
              <Group justify="space-between">
                <Badge variant="light">Build Secret #{idx + 1}</Badge>
                {!disabled && (
                  <ActionIcon
                    color="red"
                    variant="subtle"
                    onClick={() =>
                      onChange({
                        secret_bindings: bindings.filter((_, i) => i !== idx),
                      })
                    }
                  >
                    <ICONS.Remove size="1rem" />
                  </ActionIcon>
                )}
              </Group>
              <SimpleGrid cols={{ base: 1, sm: 4 }}>
                <TextInput
                  label="Source Alias / Provider Slug"
                  placeholder={sourceOptions[0] || "infisical-prod"}
                  value={b.source}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, source: e.currentTarget.value }
                          : item,
                      ),
                    })
                  }
                />
                <TextInput
                  label="Remote Secret Key"
                  placeholder="NPM_TOKEN"
                  value={b.remote_key}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, remote_key: e.currentTarget.value }
                          : item,
                      ),
                    })
                  }
                />
                <TextInput
                  label="BuildKit Secret ID"
                  placeholder="npm_token"
                  value={b.secret_id}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, secret_id: e.currentTarget.value }
                          : item,
                      ),
                    })
                  }
                />
                <Select
                  label="Conflict Policy"
                  value={
                    b.conflict_policy ??
                    Types.SecretConflictPolicy.ReplaceExisting
                  }
                  disabled={disabled}
                  onChange={(v) =>
                    v &&
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? {
                              ...item,
                              conflict_policy: v as Types.SecretConflictPolicy,
                            }
                          : item,
                      ),
                    })
                  }
                  data={[
                    {
                      label: "Replace Existing",
                      value: Types.SecretConflictPolicy.ReplaceExisting,
                    },
                    {
                      label: "Fail on Duplicate",
                      value: Types.SecretConflictPolicy.Fail,
                    },
                  ]}
                />
              </SimpleGrid>
              <Group gap="xl">
                <Switch
                  size="xs"
                  label="Required"
                  checked={b.required}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, required: e.currentTarget.checked }
                          : item,
                      ),
                    })
                  }
                />
                <Switch
                  size="xs"
                  label="Allow Empty Value"
                  checked={b.allow_empty}
                  disabled={disabled}
                  onChange={(e) =>
                    onChange({
                      secret_bindings: bindings.map((item, i) =>
                        i === idx
                          ? { ...item, allow_empty: e.currentTarget.checked }
                          : item,
                      ),
                    })
                  }
                />
              </Group>
            </Stack>
          </Card>
        ))}
      </Stack>

      <ResourceSecretsPreviewPanel target={target} />
    </Stack>
  );
}
