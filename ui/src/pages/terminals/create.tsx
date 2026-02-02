import { useRead, useShiftKeyListener, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { terminalLink } from "@/lib/utils";
import ResourceSelector from "@/resources/selector";
import {
  Button,
  Grid,
  Group,
  Popover,
  Select,
  SimpleGrid,
  Stack,
  TextInput,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { Fragment, ReactNode, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

const TERMINAL_TYPES: Types.TerminalTarget["type"][] = [
  "Server",
  "Container",
  "Stack",
  "Deployment",
] as const;

export default function CreateTerminal() {
  const [opened, { open, close, toggle }] = useDisclosure();

  useShiftKeyListener("N", () => open());
  return (
    <Popover opened={opened}>
      <Popover.Target>
        <Button leftSection={<ICONS.Create size="1rem" />} onClick={toggle}>
          New Terminal
        </Button>
      </Popover.Target>
      <Popover.Dropdown></Popover.Dropdown>
    </Popover>
  );
}

type Node = {
  label: string;
  node: ReactNode;
  hidden?: boolean;
};

type BaseRequest = {
  type: Types.TerminalTarget["type"];
  name: string;
  mode: "exec" | "attach";
  command: string | undefined;
};

function CreateTerminalLayout({
  nodes: _nodes,
  finalize,
  showMode,
  commandPlaceholder = "sh (Optional)",
}: {
  nodes: Node[];
  finalize: (baseRequest: BaseRequest) => Types.CreateTerminal;
  showMode?: boolean;
  commandPlaceholder?: string;
}) {
  const nav = useNavigate();
  const [baseRequest, setBaseRequest] = useState<BaseRequest>({
    type: "Server",
    name: "",
    mode: "exec",
    command: undefined,
  });
  const { mutate, isPending } = useWrite("CreateTerminal", {
    onSuccess: () => {
      nav(terminalLink({ target: { type: baseRequest.type, params: {} } }));
      close();
      // setRequest(defaultCreateServerTerminal(firstServer));
    },
  });
  const nodes: Node[] = [
    {
      label: "Type",
      node: (
        <Select
          value={baseRequest.type}
          onChange={(type) =>
            type &&
            setBaseRequest({
              ...baseRequest,
              type: type as Types.TerminalTarget["type"],
            })
          }
          data={TERMINAL_TYPES}
        />
      ),
    },
    ..._nodes,
    {
      label: "Terminal Name",
      node: (
        <TextInput
          autoFocus
          placeholder="terminal-name"
          value={baseRequest.name}
          onChange={(e) =>
            setBaseRequest({ ...baseRequest, name: e.target.value })
          }
          onKeyDown={(e) => {
            // if (e.key === "Enter") {
            //   onConfirm(baseRequest);
            // }
          }}
        />
      ),
    },
    {
      label: "Mode",
      hidden: !showMode,
      node: (
        <Select
          value={baseRequest.mode}
          onChange={(mode) =>
            mode &&
            setBaseRequest({ ...baseRequest, mode: mode as "exec" | "attach" })
          }
          data={["exec", "attach"]}
        />
      ),
    },
    {
      label: "Command",
      hidden: showMode && baseRequest.mode === "attach",
      node: (
        <TextInput
          placeholder={commandPlaceholder}
          value={baseRequest.command}
          onChange={(e) =>
            setBaseRequest({ ...baseRequest, command: e.target.value })
          }
          onKeyDown={(e) => {
            // if (e.key === "Enter") {
            //   onConfirm(baseRequest);
            // }
          }}
        />
      ),
    },
  ].filter((n) => !n.hidden);
  return (
    <Stack>
      <Stack hiddenFrom="md">
        {nodes.map(({ label, node }) => (
          <Stack key={label} gap="0">
            {label}
            {node}
          </Stack>
        ))}
      </Stack>

      <Grid visibleFrom="md">
        {nodes.map(({ label, node }) => (
          <Fragment key={label}>
            <Grid.Col span={4}>{label}</Grid.Col>
            <Grid.Col span={8}>{node}</Grid.Col>
          </Fragment>
        ))}
      </Grid>

      <Button
        loading={isPending}
        onClick={() => mutate(finalize(baseRequest))}
        leftSection={<ICONS.Create size="1rem" />}
      >
        Create
      </Button>
    </Stack>
  );
}

function CreateServerTerminal() {
  const firstServer = (useRead("ListServers", {}).data ?? [])[0]?.id ?? "";
  const [server, _setServer] = useState(firstServer);
  const [changed, setChanged] = useState(false);
  const setServer = (server: string) => {
    setChanged(true);
    _setServer(server);
  };
  useEffect(() => {
    if (changed) return;
    setServer(firstServer);
  }, [open, firstServer]);
  return (
    <CreateTerminalLayout
      finalize={(baseRequest) => ({
        name: baseRequest.name,
        target: { type: "Server", params: { server: server } },
      })}
      nodes={[
        {
          label: "Server",
          node: (
            <ResourceSelector
              type="Server"
              state={Types.ServerState.Ok}
              selected={server}
              onSelect={(server) => setServer(server)}
              position="bottom-end"
              targetProps={{ fullWidth: true }}
            />
          ),
        },
      ]}
    />
  );
}
