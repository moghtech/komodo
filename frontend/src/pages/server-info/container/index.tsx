import { Section } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { useServer } from "@components/resources/server";
import {
  ConfirmButton,
  DOCKER_LINK_ICONS,
  DockerLabelsSection,
  DockerResourceLink,
  ResourcePageHeader,
  ShowHideButton,
} from "@components/util";
import { useLocalStorage, useRead, useSetTitle, useWrite } from "@lib/hooks";
import { Button } from "@ui/button";
import { DataTable } from "@ui/data-table";
import {
  ChevronLeft,
  Clapperboard,
  Info,
  Loader2,
  PlusCircle,
  SearchCode,
} from "lucide-react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { ContainerLogs } from "./log";
import { Actions } from "./actions";
import { Types } from "komodo_client";
import { container_state_intention } from "@lib/color";
import { UsableResource } from "@types";
import { Fragment } from "react/jsx-runtime";
import { useEditPermissions } from "@pages/resource";
import { ResourceNotifications } from "@pages/resource-notifications";
import { MonacoEditor } from "@components/monaco";
import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";
import { ContainerTerminal } from "@components/terminal";

export const ContainerPage = () => {
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
};

const ContainerPageInner = ({
  id,
  container: container_name,
}: {
  id: string;
  container: string;
}) => {
  const [showInspect, setShowInspect] = useState(false);
  const server = useServer(id);
  useSetTitle(`${server?.name} | container | ${container_name}`);
  const { canExecute } = useEditPermissions({ type: "Server", id });
  const {
    data: container,
    isPending,
    isError,
  } = useRead("InspectDockerContainer", {
    server: id,
    container: container_name,
  });
  const list_container = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    { refetchInterval: 10_000 }
  ).data?.find((container) => container.name === container_name);

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }
  if (isError) {
    return <div className="flex w-full py-4">Failed to inspect container.</div>;
  }
  if (!container) {
    return (
      <div className="flex w-full py-4">
        No container found with given name: {container_name}
      </div>
    );
  }

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
              intent={intention}
              icon={
                <DOCKER_LINK_ICONS.container
                  server_id={id}
                  name={container_name}
                  size={8}
                />
              }
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

        <LogOrTerminal
          server={id}
          container_name={container_name}
          state={state}
        />

        {/* TOP LEVEL CONTAINER INFO */}
        <Section title="Details" icon={<Info className="w-4 h-4" />}>
          <DataTable
            tableKey="container-info"
            data={[container]}
            columns={[
              {
                accessorKey: "Id",
                header: "Id",
              },
              {
                accessorKey: "Image",
                header: "Image",
              },
              {
                accessorKey: "Driver",
                header: "Driver",
              },
            ]}
          />
        </Section>

        <DockerLabelsSection labels={container.Config?.Labels} />

        <Section
          title="Inspect"
          icon={<SearchCode className="w-4 h-4" />}
          titleRight={
            <div className="pl-2">
              <ShowHideButton show={showInspect} setShow={setShowInspect} />
            </div>
          }
        >
          {showInspect && (
            <MonacoEditor
              value={JSON.stringify(container, null, 2)}
              language="json"
              readOnly
            />
          )}
        </Section>
      </div>
    </div>
  );
};

const LogOrTerminal = ({
  server,
  container_name,
  state,
}: {
  server: string;
  container_name: string;
  state: Types.ContainerStateStatusEnum;
}) => {
  const [_view, setView] = useLocalStorage<"Log" | "Terminal">(
    `server-${server}-${container_name}-tabs-v1`,
    "Log"
  );
  const terminalDisabled = state !== Types.ContainerStateStatusEnum.Running;
  const view = terminalDisabled && _view === "Terminal" ? "Log" : _view;
  const tabs = (
    <TabsList className="justify-start w-fit">
      <TabsTrigger value="Log" className="w-[110px]">
        Log
      </TabsTrigger>
      <TabsTrigger
        value="Terminal"
        className="w-[110px]"
        disabled={terminalDisabled}
      >
        Terminal
      </TabsTrigger>
    </TabsList>
  );
  return (
    <Tabs value={view} onValueChange={setView as any} className="grid gap-4">
      <TabsContent value="Log">
        <ContainerLogs
          id={server}
          container_name={container_name}
          titleOther={tabs}
        />
      </TabsContent>
      <TabsContent value="Terminal">
        <ContainerTerminal
          server={server}
          container_name={container_name}
          titleOther={tabs}
        />
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
