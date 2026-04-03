import { fmtUpperCamelcase } from "@/lib/formatting";
import { useExecute, useSelectedResources, useWrite } from "@/lib/hooks";
import { sendCopyNotification, usableResourceExecuteKey } from "@/lib/utils";
import { UsableResource } from "@/resources";
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
import { Types } from "komodo_client";
import { ChevronDown } from "lucide-react";
import { FC, useState } from "react";

type Request = Types.ExecuteRequest["type"] | Types.WriteRequest["type"];

export interface BatchExecutionsProps<T extends Request> {
  type: UsableResource;
  executions: [T, FC<{ size?: string | number }>][];
}

export default function BatchExecutions<T extends Request>({
  type,
  executions,
}: BatchExecutionsProps<T>) {
  const [execution, setExecution] = useState<T>();
  const [selected] = useSelectedResources(type);

  return (
    <>
      <BatchExecutionsDropdownMenu
        type={type}
        executions={executions}
        onSelect={setExecution}
        disabled={!selected.length}
      />
      <BatchExecutionsModal
        type={type}
        execution={execution}
        icon={executions.find((e) => e[0] === execution)?.[1] ?? ICONS.Check}
        onClose={() => setExecution(undefined)}
      />
    </>
  );
}

function BatchExecutionsDropdownMenu<T extends Request>({
  type,
  executions,
  onSelect,
  disabled,
}: {
  type: UsableResource;
  executions: [T, FC<{ size?: string | number }>][];
  onSelect: (item: T) => void;
  disabled: boolean;
}) {
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
          {type === "ResourceSync" && (
            <Menu.Item
              onClick={() => onSelect("RefreshResourceSyncPending" as any)}
              renderRoot={(props) => <Button fullWidth {...props} />}
            >
              Refresh
            </Menu.Item>
          )}

          {executions.map(([execution, Icon]) => (
            <Menu.Item
              key={execution}
              leftSection={<Icon size="1rem" />}
              onClick={() => onSelect(execution)}
              renderRoot={(props) => <Button fullWidth {...props} />}
            >
              {fmtUpperCamelcase(
                execution.replaceAll("Batch", "").replaceAll(type, ""),
              )}
            </Menu.Item>
          ))}

          <Divider />

          <Menu.Item
            onClick={() => onSelect(`Delete${type}` as any)}
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
            Delete
          </Menu.Item>
        </Stack>
      </Menu.Dropdown>
    </Menu>
  );
}

function BatchExecutionsModal({
  type,
  execution,
  icon: Icon,
  onClose: _onClose,
}: {
  type: UsableResource;
  execution: Request | undefined;
  icon: FC<{ size?: string | number }>;
  onClose: () => void;
}) {
  const [selected, setSelected] = useSelectedResources(type);
  const [input, setInput] = useState("");
  const onClose = () => {
    setInput("");
    _onClose();
  };

  const { mutate: execute, isPending: executePending } = useExecute(
    execution as Types.ExecuteRequest["type"],
    {
      onSuccess: onClose,
    },
  );
  const { mutate: write, isPending: writePending } = useWrite(
    execution as Types.WriteRequest["type"],
    {
      onSuccess: onClose,
    },
  );

  if (!execution) return;

  const formatted = fmtUpperCamelcase(
    execution.replaceAll("Batch", "").replaceAll(type, ""),
  );
  const isPending = executePending || writePending;

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
              <List.Item key={resource}>{resource}</List.Item>
            ))}
          </List>
        </Box>

        {!execution.startsWith("Refresh") && !execution.startsWith("Check") && (
          <>
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
          </>
        )}

        <Group justify="end">
          <Button
            leftSection={
              isPending ? <Loader size="1rem" /> : <Icon size="1rem" />
            }
            onClick={() => {
              for (const resource of selected) {
                if (execution.startsWith("Delete")) {
                  write({ id: resource } as any);
                } else if (
                  execution.startsWith("Refresh") ||
                  execution.startsWith("Check")
                ) {
                  write({ [usableResourceExecuteKey(type)]: resource } as any);
                } else {
                  execute({
                    [usableResourceExecuteKey(type)]: resource,
                  } as any);
                }
              }
              if (execution.startsWith("Delete")) {
                setSelected([]);
              }
            }}
            disabled={
              execution.startsWith("Refresh") || execution.startsWith("Check")
                ? false
                : input !== formatted
            }
          >
            {formatted}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
