import { Fragment, ReactNode, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button, Grid, Menu, Select, Stack, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useRead, useShiftKeyListener, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ResourceSelector from "@/resources/selector";
import ContainerSelector from "@/components/docker/container-selector";
import StackServiceSelector from "@/components/stack-service-selector";

const TERMINAL_TYPES: Types.TerminalTarget["type"][] = [
  "Server",
  "Container",
  "Stack",
  "Deployment",
] as const;

export default function CreateTerminal() {
  const [opened, { open, close, toggle }] = useDisclosure();
  const [type, setType] = useState<Types.TerminalTarget["type"]>("Server");
  useShiftKeyListener("N", () => open());
  return (
    <Menu
      opened={opened}
      position="bottom-start"
      offset={21}
      width="auto"
      onClose={close}
    >
      <Menu.Target>
        <Button leftSection={<ICONS.Create size="1rem" />} onClick={toggle}>
          New Terminal
        </Button>
      </Menu.Target>
      <Menu.Dropdown p="lg">
        {type === "Server" ? (
          <CreateServerTerminal
            type={type}
            setType={setType}
            opened={opened}
            close={close}
          />
        ) : type === "Container" ? (
          <CreateContainerTerminal
            type={type}
            setType={setType}
            opened={opened}
            close={close}
          />
        ) : type === "Stack" ? (
          <CreateStackServiceTerminal
            type={type}
            setType={setType}
            opened={opened}
            close={close}
          />
        ) : type === "Deployment" ? (
          <CreateDeploymentTerminal
            type={type}
            setType={setType}
            opened={opened}
            close={close}
          />
        ) : (
          <></>
        )}
      </Menu.Dropdown>
    </Menu>
  );
}

type Node = {
  label: string;
  node: ReactNode;
  hidden?: boolean;
};

type BaseRequest = {
  name: string;
  mode: Types.ContainerTerminalMode;
  command: string | undefined;
};

function CreateTerminalLayout({
  type,
  setType,
  nodes: _nodes,
  finalize,
  onSuccess,
  showMode,
  commandPlaceholder = "sh (Optional)",
}: {
  type: Types.TerminalTarget["type"];
  setType: (type: Types.TerminalTarget["type"]) => void;
  nodes: Node[];
  finalize: (baseRequest: BaseRequest) => Types.CreateTerminal;
  onSuccess: (baseRequest: BaseRequest) => void;
  showMode?: boolean;
  commandPlaceholder?: string;
}) {
  const [baseRequest, setBaseRequest] = useState<BaseRequest>({
    name: "",
    mode: Types.ContainerTerminalMode.Exec,
    command: undefined,
  });
  const { mutate, isPending } = useWrite("CreateTerminal", {
    onSuccess: () => {
      onSuccess(baseRequest);
      setBaseRequest({
        name: "",
        mode: Types.ContainerTerminalMode.Exec,
        command: undefined,
      });
    },
  });
  const onConfirm = () => {
    mutate(finalize(baseRequest));
  };
  const nodes: Node[] = [
    {
      label: "Type",
      node: (
        <Select
          value={type}
          onChange={(type) =>
            type && setType(type as Types.TerminalTarget["type"])
          }
          data={TERMINAL_TYPES}
          comboboxProps={{ withinPortal: false }}
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
            if (e.key === "Enter") {
              onConfirm();
            }
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
            setBaseRequest({
              ...baseRequest,
              name:
                mode === Types.ContainerTerminalMode.Attach
                  ? "attach"
                  : baseRequest.name,
              mode: mode as Types.ContainerTerminalMode,
            })
          }
          data={["exec", "attach"]}
          comboboxProps={{ withinPortal: false }}
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
            if (e.key === "Enter") {
              onConfirm();
            }
          }}
        />
      ),
    },
  ].filter((n) => !n.hidden);
  return (
    <Stack w={{ base: 300, lg: 500 }}>
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
        disabled={!baseRequest.name}
      >
        Create
      </Button>
    </Stack>
  );
}

function CreateServerTerminal({
  type,
  setType,
  opened,
  close,
}: {
  type: Types.TerminalTarget["type"];
  setType: (type: Types.TerminalTarget["type"]) => void;
  opened: boolean;
  close: () => void;
}) {
  const nav = useNavigate();
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
  }, [opened, firstServer]);
  return (
    <CreateTerminalLayout
      type={type}
      setType={setType}
      finalize={(baseRequest) => ({
        name: baseRequest.name,
        target: { type: "Server", params: { server: server } },
      })}
      onSuccess={(request) => {
        nav(`/servers/${server}/terminal/${request.name}`);
        close();
      }}
      nodes={[
        {
          label: "Server",
          node: (
            <ResourceSelector
              type="Server"
              state={Types.ServerState.Ok}
              selected={server}
              onSelect={(server) => setServer(server)}
              withinPortal={false}
              targetProps={{ w: "100%", maw: "100%" }}
            />
          ),
        },
      ]}
      commandPlaceholder="bash (Optional)"
    />
  );
}

function CreateContainerTerminal({
  type,
  setType,
  opened,
  close,
}: {
  type: Types.TerminalTarget["type"];
  setType: (type: Types.TerminalTarget["type"]) => void;
  opened: boolean;
  close: () => void;
}) {
  const nav = useNavigate();
  const firstServer = (useRead("ListServers", {}).data ?? [])[0]?.id ?? "";
  const [params, _setParams] = useState({ server: firstServer, container: "" });
  const [changed, setChanged] = useState(false);
  const setParams = (params: { server: string; container: string }) => {
    setChanged(true);
    _setParams(params);
  };
  useEffect(() => {
    if (changed) return;
    setParams({ ...params, server: firstServer });
  }, [opened, firstServer]);
  return (
    <CreateTerminalLayout
      type={type}
      setType={setType}
      finalize={(baseRequest) => ({
        name: baseRequest.name,
        mode: baseRequest.mode,
        target: {
          type: "Container",
          params: { server: params.server, container: params.container },
        },
      })}
      onSuccess={(request) => {
        nav(
          `/servers/${params.server}/container/${params.container}/terminal/${request.name}`,
        );
        close();
        setParams({ server: firstServer, container: "" });
      }}
      nodes={[
        {
          label: "Server",
          node: (
            <ResourceSelector
              type="Server"
              state={Types.ServerState.Ok}
              selected={params.server}
              onSelect={(server) => setParams({ ...params, server })}
              withinPortal={false}
              targetProps={{ w: "100%", maw: "100%" }}
            />
          ),
        },
        {
          label: "Container",
          node: (
            <ContainerSelector
              serverId={params.server}
              selected={params.container}
              state={Types.ContainerStateStatusEnum.Running}
              onSelect={(container) => setParams({ ...params, container })}
              withinPortal={false}
              targetProps={{ w: "100%", maw: "100%" }}
            />
          ),
        },
      ]}
      showMode
    />
  );
}

function CreateStackServiceTerminal({
  type,
  setType,
  opened,
  close,
}: {
  type: Types.TerminalTarget["type"];
  setType: (type: Types.TerminalTarget["type"]) => void;
  opened: boolean;
  close: () => void;
}) {
  const nav = useNavigate();
  const firstStack = (useRead("ListStacks", {}).data ?? [])[0]?.id ?? "";
  const [params, _setParams] = useState({ stack: firstStack, service: "" });
  const [changed, setChanged] = useState(false);
  const setParams = (params: { stack: string; service: string }) => {
    setChanged(true);
    _setParams(params);
  };
  useEffect(() => {
    if (changed) return;
    setParams({ ...params, stack: firstStack });
  }, [opened, firstStack]);
  return (
    <CreateTerminalLayout
      type={type}
      setType={setType}
      finalize={(baseRequest) => ({
        name: baseRequest.name,
        mode: baseRequest.mode,
        target: {
          type: "Stack",
          params: { stack: params.stack, service: params.service },
        },
      })}
      onSuccess={(request) => {
        nav(
          `/stacks/${params.stack}/service/${params.service}/terminal/${request.name}`,
        );
        close();
        setParams({ stack: firstStack, service: "" });
      }}
      nodes={[
        {
          label: "Stack",
          node: (
            <ResourceSelector
              type="Stack"
              state={Types.StackState.Running}
              selected={params.stack}
              onSelect={(stack) => setParams({ ...params, stack })}
              withinPortal={false}
              targetProps={{ w: "100%", maw: "100%" }}
            />
          ),
        },
        {
          label: "Service",
          node: (
            <StackServiceSelector
              stackId={params.stack}
              selected={params.service}
              state={Types.ContainerStateStatusEnum.Running}
              onSelect={(service) => setParams({ ...params, service })}
              withinPortal={false}
              targetProps={{ w: "100%", maw: "100%" }}
            />
          ),
        },
      ]}
      showMode
    />
  );
}

function CreateDeploymentTerminal({
  type,
  setType,
  opened,
  close,
}: {
  type: Types.TerminalTarget["type"];
  setType: (type: Types.TerminalTarget["type"]) => void;
  opened: boolean;
  close: () => void;
}) {
  const nav = useNavigate();
  const firstDeployment =
    (useRead("ListDeployments", {}).data ?? [])[0]?.id ?? "";
  const [deployment, _setDeployment] = useState(firstDeployment);
  const [changed, setChanged] = useState(false);
  const setDeployment = (deployment: string) => {
    setChanged(true);
    _setDeployment(deployment);
  };
  useEffect(() => {
    if (changed) return;
    setDeployment(firstDeployment);
  }, [opened, firstDeployment]);
  return (
    <CreateTerminalLayout
      type={type}
      setType={setType}
      finalize={(baseRequest) => ({
        name: baseRequest.name,
        target: { type: "Deployment", params: { deployment: deployment } },
      })}
      onSuccess={(request) => {
        nav(`/deployments/${deployment}/terminal/${request.name}`);
        close();
      }}
      nodes={[
        {
          label: "Deployment",
          node: (
            <ResourceSelector
              type="Deployment"
              state={Types.DeploymentState.Running}
              selected={deployment}
              onSelect={(deployment) => setDeployment(deployment)}
              withinPortal={false}
              targetProps={{ w: "100%", maw: "100%" }}
            />
          ),
        },
      ]}
    />
  );
}
