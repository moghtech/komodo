import { ICONS } from "@/lib/icons";
import {
  hasComposeWrapperConflict,
  hasExternalSecretsInStack,
  stageStatusBadgeColor,
} from "@/components/secret-provider/utils";
import {
  Alert,
  Badge,
  Card,
  Code,
  Group,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import { Types } from "komodo_client";

export function ConnectionTestResultView({
  result,
}: {
  result: Types.TestSecretProviderResponse;
}) {
  const stages: Array<{
    label: string;
    stage: Types.SecretTestStageResult;
    extra?: string;
  }> = [
    { label: "1. Authentication (Universal Auth)", stage: result.authentication },
    {
      label: "2. Scope Metadata Discovery (v4)",
      stage: result.metadata_access,
      extra:
        result.key_count !== undefined
          ? `${result.key_count} key(s) visible`
          : undefined,
    },
    {
      label: "3. Secret Value Read Permission (v4)",
      stage: result.value_read_access,
    },
  ];

  return (
    <Card withBorder padding="sm" radius="md">
      <Stack gap="xs">
        <Group justify="space-between">
          <Text fw={600} fz="sm">
            Connection Verification Report
          </Text>
          <Badge color={result.ready_for_deploy ? "green" : "orange"}>
            {result.ready_for_deploy
              ? "Ready for Deploy"
              : "Not Verified for Deploy"}
          </Badge>
        </Group>
        <Table withRowBorders>
          <Table.Tbody>
            {stages.map((row) => (
              <Table.Tr key={row.label}>
                <Table.Td fw={500} fz="xs">
                  {row.label}
                </Table.Td>
                <Table.Td w={100}>
                  <Badge
                    size="xs"
                    color={stageStatusBadgeColor(row.stage.status)}
                  >
                    {row.stage.status}
                  </Badge>
                </Table.Td>
                <Table.Td fz="xs">
                  {row.stage.message}
                  {row.extra ? ` (${row.extra})` : ""}
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      </Stack>
    </Card>
  );
}

export function StackSecretWarningsBanner({
  config,
  swarmId,
}: {
  config: Partial<Types.StackConfig>;
  swarmId?: string;
}) {
  const hasExternal = hasExternalSecretsInStack(config);
  const wrapperConflict = hasComposeWrapperConflict(config);

  return (
    <Stack gap="xs">
      {swarmId && hasExternal && (
        <Alert color="red" icon={<ICONS.Alert size="1rem" />}>
          Docker Swarm native external secrets are not supported in V1. Deploys
          on Swarm with external secret references or bindings will fail closed.
        </Alert>
      )}
      {wrapperConflict && (
        <Alert color="red" icon={<ICONS.Alert size="1rem" />}>
          <b>Wrapper Conflict:</b> Native external secrets and{" "}
          <Code>compose_cmd_wrapper</Code> cannot be active on the same Stack
          execution. Remove the wrapper or migrate wrapper secrets to native
          Secret Providers before deploying.
        </Alert>
      )}
      <Alert color="blue" variant="light" icon={<ICONS.Info size="1rem" />}>
        <Stack gap={4}>
          <Text fz="xs">
            <b>Scope &amp; .env Security Boundary:</b> Inline{" "}
            <Code>[[vault.&lt;slug&gt;.&lt;key&gt;]]</Code> references in{" "}
            <b>Environment</b> and <Code>ComposeEnv</Code> bindings are written
            atomically (<Code>0600</Code>) by Periphery to the Stack{" "}
            <Code>.env</Code> file for Compose <Code>${"{VAR}"}</Code>{" "}
            interpolation. They are masked in Komodo APIs/Inspect/Logs and
            excluded from <Code>StackInfo</Code>, <Code>ResourceSync</Code>, and
            Git, but remain readable on the target host by root/Periphery SSH
            users and host disk backups. Use <Code>ServiceEnv</Code> or{" "}
            <Code>ComposeFileSecret</Code> (<Code>0400</Code> runtime mount) to
            scope secrets to explicit services.
          </Text>
          <Text fz="xs">
            <b>Literal Replacement:</b> External values are substituted
            byte-for-byte without URL encoding or nested <Code>[[...]]</Code> /{" "}
            <Code>${"{...}"}</Code> rescanning.
          </Text>
        </Stack>
      </Alert>
    </Stack>
  );
}
