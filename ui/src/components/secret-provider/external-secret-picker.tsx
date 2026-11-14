import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import {
  buildInlineVaultToken,
  buildWholeValueEnvSnippet,
} from "@/components/secret-provider/utils";
import {
  Alert,
  Badge,
  Button,
  Code,
  Group,
  Modal,
  Select,
  SimpleGrid,
  Stack,
  Table,
  Text,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";

export interface ExternalSecretPickerProps {
  target: Types.ResourceTarget;
  disabled?: boolean;
  onInsertLine?: (line: string) => void;
}

export default function ExternalSecretPicker({
  target,
  disabled,
  onInsertLine,
}: ExternalSecretPickerProps) {
  const [opened, { open, close }] = useDisclosure(false);

  const providers =
    useRead("ListAvailableSecretProviders", { target }).data ?? [];
  const enabledProviders = providers.filter(
    (p) => p.enabled && p.has_credentials && p.granted,
  );

  const [selectedSlug, setSelectedSlug] = useState<string>("");
  const [projectId, setProjectId] = useState("");
  const [environment, setEnvironment] = useState("");
  const [secretPath, setSecretPath] = useState("/");
  const [keyFilter, setKeyFilter] = useState("");

  useEffect(() => {
    if (!selectedSlug && enabledProviders.length > 0) {
      const first = enabledProviders[0];
      setSelectedSlug(first.slug);
      const def = first.default_selector?.params;
      if (def) {
        setProjectId(def.project_id);
        setEnvironment(def.environment);
        setSecretPath(def.secret_path || "/");
      }
    }
  }, [enabledProviders, selectedSlug]);

  const activeProvider = enabledProviders.find((p) => p.slug === selectedSlug);

  const explicitSelector: Types.SecretSelector | undefined =
    projectId.trim() && environment.trim()
      ? {
          type: "Infisical",
          params: {
            project_id: projectId.trim(),
            environment: environment.trim(),
            secret_path: secretPath.trim() || "/",
            include_imports:
              activeProvider?.default_selector?.params.include_imports ?? false,
            expand_references:
              activeProvider?.default_selector?.params.expand_references ??
              false,
          },
        }
      : undefined;

  const keysQuery = useRead(
    "ListExternalSecretKeys",
    {
      target,
      provider: selectedSlug,
      selector: explicitSelector,
    },
    {
      enabled:
        opened &&
        !!selectedSlug &&
        !!activeProvider?.capabilities.supports_metadata_listing,
    },
  );

  if (providers.length === 0) {
    return null;
  }

  const keys = (keysQuery.data ?? []).filter((item) => {
    if (!keyFilter.trim()) return true;
    const term = keyFilter.toLowerCase().trim();
    return (
      item.key.toLowerCase().includes(term) ||
      (item.comment ?? "").toLowerCase().includes(term)
    );
  });

  const copyText = (value: string, label: string) => {
    navigator.clipboard.writeText(value);
    notifications.show({ message: label });
  };

  return (
    <>
      <Button
        leftSection={<ICONS.Key size="1rem" />}
        variant="light"
        disabled={disabled}
        onClick={open}
      >
        External Secrets
      </Button>

      <Modal
        opened={opened}
        onClose={close}
        title={
          <Text fw={600} size="lg">
            External Secret Key Picker (Metadata Only)
          </Text>
        }
        size="xl"
      >
        <Stack gap="md">
          <Alert color="blue" variant="light" icon={<ICONS.Info size="1rem" />}>
            <Stack gap={4}>
              <Text fz="xs">
                <b>Literal substitution only:</b> Komodo replaces{" "}
                <Code>[[vault.&lt;provider-slug&gt;.&lt;secret-key&gt;]]</Code>{" "}
                byte-for-byte with zero URL encoding, escaping, or nested
                template expansion.
              </Text>
              <Text fz="xs">
                <b>Recommended:</b> Store complete, pre-encoded connection
                strings in Infisical and reference the whole value (e.g.{" "}
                <Code>
                  DATABASE_URL=[[vault.
                  {selectedSlug || "infisical-prod"}.DATABASE_URL]]
                </Code>
                ), though mixed interpolation (e.g.{" "}
                <Code>
                  postgres://user:[[vault.
                  {selectedSlug || "infisical-prod"}.DB_PASSWORD]]@db:5432/app
                </Code>
                ) is also supported.
              </Text>
            </Stack>
          </Alert>

          {enabledProviders.length === 0 ? (
            <Alert color="yellow" icon={<ICONS.Alert size="1rem" />}>
               Secret Provider(s) exist in Komodo, but none currently have an
              active enabled grant and stored credentials for this resource. Ask
              an administrator to configure a grant under{" "}
              <b>Settings → Providers → Secret Providers</b>.
            </Alert>
          ) : (
            <>
              <SimpleGrid cols={{ base: 1, sm: 4 }}>
                <Select
                  label="Secret Provider"
                  value={selectedSlug}
                  onChange={(slug) => {
                    if (!slug) return;
                    setSelectedSlug(slug);
                    const p = enabledProviders.find((item) => item.slug === slug);
                    const def = p?.default_selector?.params;
                    setProjectId(def?.project_id ?? "");
                    setEnvironment(def?.environment ?? "");
                    setSecretPath(def?.secret_path || "/");
                  }}
                  data={enabledProviders.map((p) => ({
                    label: `${p.name} (${p.slug})`,
                    value: p.slug,
                  }))}
                />
                <TextInput
                  label="Project ID"
                  placeholder="Infisical projectId"
                  value={projectId}
                  onChange={(e) => setProjectId(e.currentTarget.value)}
                />
                <TextInput
                  label="Environment"
                  placeholder="prod"
                  value={environment}
                  onChange={(e) => setEnvironment(e.currentTarget.value)}
                />
                <TextInput
                  label="Secret Path"
                  placeholder="/"
                  value={secretPath}
                  onChange={(e) => setSecretPath(e.currentTarget.value)}
                />
              </SimpleGrid>

              <TextInput
                leftSection={<ICONS.Search size="1rem" />}
                placeholder="Filter remote secret keys by name or comment..."
                value={keyFilter}
                onChange={(e) => setKeyFilter(e.currentTarget.value)}
              />

              {keysQuery.isError && (
                <Alert color="red" icon={<ICONS.Alert size="1rem" />}>
                  Unable to list metadata for the selected scope. Check that
                  your grant includes <Code>MetadataList</Code> for this
                  Project/Environment/Path, or enter{" "}
                  <Code>
                    [[vault.{selectedSlug}.&lt;SECRET_KEY&gt;]]
                  </Code>{" "}
                  directly.
                </Alert>
              )}

              <Table withTableBorder withRowBorders>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Remote Key</Table.Th>
                    <Table.Th>Metadata</Table.Th>
                    <Table.Th>Copy / Insert Reference</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {keys.map((item) => {
                    const token = buildInlineVaultToken(selectedSlug, item.key);
                    const wholeLine = buildWholeValueEnvSnippet(
                      item.key.replace(/[^A-Za-z0-9_]/g, "_"),
                      selectedSlug,
                      item.key,
                    );
                    return (
                      <Table.Tr key={`${item.source_path ?? ""}:${item.key}`}>
                        <Table.Td>
                          <Stack gap={2}>
                            <Code>{item.key}</Code>
                            {item.comment && (
                              <Text fz="xs" c="dimmed">
                                {item.comment}
                              </Text>
                            )}
                          </Stack>
                        </Table.Td>
                        <Table.Td>
                          <Group gap={4}>
                            {item.version !== undefined && (
                              <Badge size="xs" variant="light">
                                v{item.version}
                              </Badge>
                            )}
                            {item.imported && (
                              <Badge size="xs" color="cyan" variant="outline">
                                imported
                              </Badge>
                            )}
                            {item.value_hidden && (
                              <Badge size="xs" color="red">
                                ReadValue missing
                              </Badge>
                            )}
                          </Group>
                        </Table.Td>
                        <Table.Td>
                          <Group gap="xs" wrap="nowrap">
                            <Button
                              size="compact-xs"
                              variant="default"
                              onClick={() =>
                                copyText(token, `Copied ${token}`)
                              }
                            >
                              Copy Token
                            </Button>
                            <Button
                              size="compact-xs"
                              variant="light"
                              onClick={() =>
                                copyText(wholeLine, `Copied ${wholeLine}`)
                              }
                            >
                              Copy KEY=Token
                            </Button>
                            {onInsertLine && (
                              <Button
                                size="compact-xs"
                                onClick={() => {
                                  onInsertLine(wholeLine);
                                  notifications.show({
                                    message: `Inserted ${item.key} reference`,
                                  });
                                }}
                              >
                                Insert
                              </Button>
                            )}
                          </Group>
                        </Table.Td>
                      </Table.Tr>
                    );
                  })}
                  {!keysQuery.isLoading && keys.length === 0 && (
                    <Table.Tr>
                      <Table.Td colSpan={3}>
                        <Text fz="sm" c="dimmed" ta="center" py="sm">
                          No remote keys matched in this scope (secret values
                          are never fetched here).
                        </Text>
                      </Table.Td>
                    </Table.Tr>
                  )}
                </Table.Tbody>
              </Table>
            </>
          )}
        </Stack>
      </Modal>
    </>
  );
}
