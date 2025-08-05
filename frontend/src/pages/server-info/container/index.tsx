import { Section } from "@components/layouts";
import { ResourceLink, ResourcePageHeader } from "@components/resources/common";
import { useServer } from "@components/resources/server";
import {
  ConfirmButton,
  ContainerPortLink,
  DOCKER_LINK_ICONS,
  DockerLabelsSection,
  DockerResourceLink,
} from "@components/util";
import {
  useContainerPortsMap,
  useLocalStorage,
  useRead,
  useSetTitle,
  useWrite,
} from "@lib/hooks";
import { Button } from "@ui/button";
import { DataTable } from "@ui/data-table";
import {
  ChevronLeft,
  Clapperboard,
  Info,
  Loader2,
  PlusCircle,
} from "lucide-react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { ContainerLogs } from "./log";
import { Actions } from "./actions";
import { ConnectExecQuery, Types } from "komodo_client";
import { container_state_intention } from "@lib/color";
import { UsableResource } from "@types";
import { Fragment } from "react/jsx-runtime";
import { usePermissions } from "@lib/hooks";
import { ResourceNotifications } from "@pages/resource-notifications";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";
import { ContainerTerminal } from "@components/terminal/container";
import { ContainerInspect } from "./inspect";
import { useMemo } from "react";

export default function ContainerPage() {
  const { type, id, container } = useParams() as {
    type: string;
    id: string;
    container: string;
  };
  if (type !== "servers") {
    return <div>This resource type does not have any containers.</div>;
  }
  return (
    <ContainerPageInner id={id} container={decodeURIComponent(container)} />
  );
}

const ContainerPageInner = ({
  id,
  container: container_name,
}: {
  id: string;
  container: string;
}) => {
  const server = useServer(id);
  useSetTitle(`${server?.name} | container | ${container_name}`);
  const { canExecute } = usePermissions({ type: "Server", id });
  const list_container = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    { refetchInterval: 10_000 }
  ).data?.find((container) => container.name === container_name);
  const ports_map = useContainerPortsMap(list_container?.ports ?? []);

  const state = list_container?.state ?? Types.ContainerStateStatusEnum.Empty;
  const intention = container_state_intention(state);

  return (
    <div>
      <div className="w-full flex items-center justify-between mb-12">
        <Link to={"/servers/" + id}>
          <Button className="gap-2" variant="secondary">
            <ChevronLeft className="w-4" />
            Back
          </Button>
        </Link>
        <NewDeployment id={id} name={container_name} />
      </div>
      <div className="flex flex-col xl:flex-row gap-4">
        {/** HEADER */}
        <div className="w-full flex flex-col gap-4">
          <div className="flex flex-col gap-2 border rounded-md">
            {/* <Components.ResourcePageHeader id={id} /> */}
            <ResourcePageHeader
              type={undefined}
              id={undefined}
              intent={intention}
              icon={
                <DOCKER_LINK_ICONS.container
                  server_id={id}
                  name={container_name}
                  size={8}
                />
              }
              resource={undefined}
              name={container_name}
              state={state}
              status={list_container?.status}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-1 flex-wrap text-muted-foreground">
                <ResourceLink type="Server" id={id} />
                <AttachedResource id={id} container={container_name} />
                {list_container?.image && (
                  <>
                    |
                    <DockerResourceLink
                      type="image"
                      server_id={id}
                      name={list_container.image}
                      id={list_container.image_id}
                      muted
                    />
                  </>
                )}
                {list_container?.networks.map((network) => (
                  <Fragment key={network}>
                    |
                    <DockerResourceLink
                      type="network"
                      server_id={id}
                      name={network}
                      muted
                    />
                  </Fragment>
                ))}
                {list_container?.volumes.map((volume) => (
                  <Fragment key={volume}>
                    |
                    <DockerResourceLink
                      type="volume"
                      server_id={id}
                      name={volume}
                      muted
                    />
                  </Fragment>
                ))}
                {Object.keys(ports_map).map((host_port) => (
                  <Fragment key={host_port}>
                    |
                    <ContainerPortLink
                      host_port={host_port}
                      ports={ports_map[host_port]}
                      server_id={id}
                    />
                  </Fragment>
                ))}
              </div>
            </div>
          </div>
          {/* <ResourceDescription type="Server" id={id} disabled={!canWrite} /> */}
        </div>
        {/** NOTIFICATIONS */}
        <ResourceNotifications type="Server" id={id} />
      </div>

      <div className="mt-8 flex flex-col gap-12">
        {/* Actions */}
        {canExecute && (
          <Section title="Actions" icon={<Clapperboard className="w-4 h-4" />}>
            <div className="flex gap-4 items-center flex-wrap">
              {Object.entries(Actions).map(([key, Action]) => (
                <Action key={key} id={id} container={container_name} />
              ))}
            </div>
          </Section>
        )}

        <ContainerTabs server={id} container={container_name} state={state} />

        {/* TOP LEVEL CONTAINER INFO */}
        {list_container && (
          <Section title="Details" icon={<Info className="w-4 h-4" />}>
            <DataTable
              tableKey="container-info"
              data={[list_container]}
              columns={[
                {
                  header: "Id",
                  accessorKey: "id",
                },
                {
                  header: "Image",
                  accessorKey: "image",
                },
                {
                  header: "Network Mode",
                  accessorKey: "network_mode",
                },
                {
                  header: "Networks",
                  accessorKey: "networks",
                },
              ]}
            />
          </Section>
        )}

        <DockerLabelsSection labels={list_container?.labels} />
      </div>
    </div>
  );
};

const ContainerTabs = ({
  server,
  container,
  state,
}: {
  server: string;
  container: string;
  state: Types.ContainerStateStatusEnum;
}) => {
  const [_view, setView] = useLocalStorage<"Log" | "Inspect" | "Terminal">(
    `server-${server}-${container}-tabs-v1`,
    "Log"
  );
  const { specificLogs, specificInspect, specificTerminal } = usePermissions({
    type: "Server",
    id: server,
  });
  const container_exec_disabled =
    useServer(server)?.info.container_exec_disabled ?? true;
  const logDisabled =
    !specificLogs || state === Types.ContainerStateStatusEnum.Empty;
  const inspectDisabled =
    !specificInspect || state === Types.ContainerStateStatusEnum.Empty;
  const terminalDisabled =
    !specificTerminal ||
    container_exec_disabled ||
    state !== Types.ContainerStateStatusEnum.Running;
  const view =
    (inspectDisabled && _view === "Inspect") ||
    (terminalDisabled && _view === "Terminal")
      ? "Log"
      : _view;
  const tabs = useMemo(() => {
    return (
      <TabsList className="justify-start w-fit">
        <TabsTrigger value="Log" className="w-[110px]" disabled={logDisabled}>
          Log
        </TabsTrigger>
        {specificInspect && (
          <TabsTrigger
            value="Inspect"
            className="w-[110px]"
            disabled={inspectDisabled}
          >
            Inspect
          </TabsTrigger>
        )}
        {specificTerminal && (
          <TabsTrigger
            value="Terminal"
            className="w-[110px]"
            disabled={terminalDisabled}
          >
            Terminal
          </TabsTrigger>
        )}
      </TabsList>
    );
  }, [
    logDisabled,
    specificInspect,
    inspectDisabled,
    specificTerminal,
    terminalDisabled,
  ]);
  const terminalQuery = useMemo(
    () =>
      ({
        type: "container",
        query: {
          server,
          container,
          // This is handled inside ContainerTerminal
          shell: "",
        },
      }) as ConnectExecQuery,
    [server, container]
  );
  return (
    <Tabs value={view} onValueChange={setView as any}>
      <TabsContent value="Log">
        <ContainerLogs
          id={server}
          container_name={container}
          titleOther={tabs}
          disabled={logDisabled}
        />
      </TabsContent>
      <TabsContent value="Inspect">
        <ContainerInspect id={server} container={container} titleOther={tabs} />
      </TabsContent>
      <TabsContent value="Terminal">
        <ContainerTerminal query={terminalQuery} titleOther={tabs} />
      </TabsContent>
    </Tabs>
  );
};

const AttachedResource = ({
  id,
  container,
}: {
  id: string;
  container: string;
}) => {
  const { data: attached, isPending } = useRead(
    "GetResourceMatchingContainer",
    { server: id, container },
    { refetchInterval: 10_000 }
  );

  if (isPending) {
    return <Loader2 className="w-4 h-4 animate-spin" />;
  }

  if (!attached || !attached.resource) {
    return null;
  }

  return (
    <>
      |
      <ResourceLink
        type={attached.resource.type as UsableResource}
        id={attached.resource.id}
      />
    </>
  );
};

const NewDeployment = ({ id, name }: { id: string; name: string }) => {
  const { data: attached, isPending } = useRead(
    "GetResourceMatchingContainer",
    { server: id, container: name }
  );

  if (isPending) {
    return <Loader2 className="w-4 h-4 animate-spin" />;
  }

  if (!attached) {
    return null;
  }

  if (!attached?.resource) {
    return <NewDeploymentInner name={name} server_id={id} />;
  }
};

const NewDeploymentInner = ({
  server_id,
  name,
}: {
  name: string;
  server_id: string;
}) => {
  const nav = useNavigate();
  const { mutateAsync, isPending } = useWrite("CreateDeploymentFromContainer");
  return (
    <ConfirmButton
      title="New Deployment"
      icon={<PlusCircle className="w-4 h-4" />}
      onClick={async () => {
        const id = (await mutateAsync({ name, server: server_id }))._id?.$oid!;
        nav(`/deployments/${id}`);
      }}
      loading={isPending}
    />
  );
};
