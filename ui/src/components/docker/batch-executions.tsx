import { fmtUpperCamelcase, sendCopyNotification } from "mogh_ui";
import { useExecute, useRead, useSelectedDockerResources } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import {
  Box,
  Button,
  Divider,
  Group,
  List,
  Loader,
  Menu,
  Modal,
  Stack,
  Text,
  TextInput,
  useMatches,
} from "@mantine/core";
import { ChevronDown } from "lucide-react";
import { FC, useState } from "react";
import { DockerResourceType } from ".";

type DockerExecution =
  | "StartContainer"
  | "RestartContainer"
  | "PauseContainer"
  | "UnpauseContainer"
  | "StopContainer"
  | "DestroyContainer"
  | "DeleteNetwork"
  | "DeleteImage"
  | "DeleteVolume";

const DOCKER_EXECUTIONS: {
  [type in DockerResourceType]: [
    DockerExecution,
    FC<{ size?: string | number }>,
  ][];
} = {
  Container: [
    ["StartContainer", ICONS.Start],
    ["RestartContainer", ICONS.Restart],
    ["PauseContainer", ICONS.Pause],
    ["UnpauseContainer", ICONS.Start],
    ["StopContainer", ICONS.Stop],
  ],
  Network: [],
  Image: [],
  Volume: [],
};

const DELETE_EXECUTIONS: { [type in DockerResourceType]: DockerExecution } = {
  Container: "DestroyContainer",
  Network: "DeleteNetwork",
  Image: "DeleteImage",
  Volume: "DeleteVolume",
};

/** Splits `${server_id} ${resource_name}` held in
 * useSelectedDockerResources into its parts. */
function splitDockerResource(resource: string): [string, string] {
  const i = resource.indexOf(" ");
  return [resource.slice(0, i), resource.slice(i + 1)];
}

export interface DockerBatchExecutionsProps {
  type: DockerResourceType;
}

export default function DockerBatchExecutions({
  type,
}: DockerBatchExecutionsProps) {
  const [execution, setExecution] = useState<DockerExecution>();
  const [selected] = useSelectedDockerResources(type);

  return (
    <>
      <DockerBatchExecutionsDropdownMenu
        type={type}
        onSelect={setExecution}
        disabled={!selected.length}
      />
      <DockerBatchExecutionsModal
        type={type}
        execution={execution}
        icon={
          DOCKER_EXECUTIONS[type].find((e) => e[0] === execution)?.[1] ??
          ICONS.Delete
        }
        onClose={() => setExecution(undefined)}
      />
    </>
  );
}

function DockerBatchExecutionsDropdownMenu({
  type,
  onSelect,
  disabled,
}: {
  type: DockerResourceType;
  onSelect: (item: DockerExecution) => void;
  disabled: boolean;
}) {
  const executions = DOCKER_EXECUTIONS[type];
  const width = useMatches({
    base: "target",
    xs: 250,
  });
  return (
    <Menu position="bottom-start" offset={16} disabled={disabled} width={width}>
      <Menu.Target>
        <Button
          leftSection={<ICONS.Execution size="1rem" />}
          rightSection={<ChevronDown size="1rem" />}
          disabled={disabled}
          w={{ base: "100%", sm: "fit-content" }}
        >
          Execute
        </Button>
      </Menu.Target>
      <Menu.Dropdown>
        <Stack gap="xs" p="sm">
          {executions.map(([execution, Icon]) => (
            <Menu.Item
              key={execution}
              leftSection={<Icon size="1rem" />}
              onClick={() => onSelect(execution)}
              renderRoot={(props) => <Button fullWidth {...props} />}
            >
              {fmtUpperCamelcase(execution.replaceAll(type, ""))}
            </Menu.Item>
          ))}

          {executions.length > 0 && <Divider />}

          <Menu.Item
            onClick={() => onSelect(DELETE_EXECUTIONS[type])}
            renderRoot={(props) => (
              <Button
                variant="filled"
                color="red"
                leftSection={<ICONS.Delete size="1rem" />}
                fullWidth
                {...props}
              />
            )}
          >
            {fmtUpperCamelcase(DELETE_EXECUTIONS[type].replaceAll(type, ""))}
          </Menu.Item>
        </Stack>
      </Menu.Dropdown>
    </Menu>
  );
}

function DockerBatchExecutionsModal({
  type,
  execution,
  icon: Icon,
  onClose: _onClose,
}: {
  type: DockerResourceType;
  execution: DockerExecution | undefined;
  icon: FC<{ size?: string | number }>;
  onClose: () => void;
}) {
  const [selected, setSelected] = useSelectedDockerResources(type);
  const [input, setInput] = useState("");
  const onClose = () => {
    setInput("");
    _onClose();
  };

  const { mutate: execute, isPending } = useExecute(execution!, {
    onSuccess: onClose,
  });

  if (!execution) return;

  const formatted = fmtUpperCamelcase(execution.replaceAll(type, ""));
  const isDelete = execution === DELETE_EXECUTIONS[type];

  return (
    <Modal
      opened={!!execution}
      onClose={() => onClose()}
      title={<Text size="lg">Group Execute - {formatted}</Text>}
      size="lg"
    >
      <Stack>
        <Box bg="accent.1" p="md">
          <List>
            {selected.map((resource) => (
              <SelectedDockerResource key={resource} resource={resource} />
            ))}
          </List>
        </Box>

        <Text
          onClick={() => {
            navigator.clipboard.writeText(formatted);
            sendCopyNotification();
          }}
          style={{ cursor: "pointer" }}
        >
          Please enter <b>{formatted}</b> below to confirm this action.
          {(location.origin.startsWith("https") ||
            // For dev
            location.origin.startsWith("http://localhost:")) && (
            <Text fz="sm" c="dimmed">
              You may click the text in bold to copy it
            </Text>
          )}
        </Text>

        <TextInput
          value={input}
          onChange={(e) => setInput(e.target.value)}
          error={input === formatted ? undefined : "Does not match"}
        />

        <Group justify="end">
          <Button
            leftSection={
              isPending ? <Loader size="1rem" /> : <Icon size="1rem" />
            }
            onClick={() => {
              for (const resource of selected) {
                const [server, name] = splitDockerResource(resource);
                execute(
                  type === "Container"
                    ? ({ server, container: name } as any)
                    : ({ server, name } as any),
                );
              }
              if (isDelete) {
                setSelected([]);
              }
            }}
            disabled={input !== formatted}
          >
            {formatted}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}

function SelectedDockerResource({ resource }: { resource: string }) {
  const [serverId, name] = splitDockerResource(resource);
  const server = useRead("ListServers", {}).data?.find(
    (server) => server.id === serverId,
  );
  return (
    <List.Item>
      <Group gap="xs" wrap="nowrap">
        <Text>{name}</Text>
        <Text c="dimmed" fz="sm">
          {server?.name ?? serverId}
        </Text>
      </Group>
    </List.Item>
  );
}
