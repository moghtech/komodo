import { useLocalStorage, useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { RequiredResourceComponents } from "@types";
import { CircleArrowUp, HardDrive, Rocket, Server } from "lucide-react";
import { cn } from "@lib/utils";
import { useServer } from "../server";
import {
  DeployDeployment,
  StartStopDeployment,
  DestroyDeployment,
  RestartDeployment,
  PauseUnpauseDeployment,
  PullDeployment,
} from "./actions";
import { DeploymentLogs } from "./log";
import {
  deployment_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { DeploymentTable } from "./table";
import { DeleteResource, NewResource, ResourceLink } from "../common";
import { RunBuild } from "../build/actions";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";
import { DeploymentConfig } from "./config";
import { DashboardPieChart } from "@pages/home/dashboard";
import {
  DockerResourceLink,
  ResourcePageHeader,
  StatusBadge,
} from "@components/util";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import { usePermissions } from "@lib/hooks";
import { ContainerTerminal } from "@components/terminal/container";

// const configOrLog = atomWithStorage("config-or-log-v1", "Config");

export const useDeployment = (id?: string) =>
  useRead("ListDeployments", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

export const useFullDeployment = (id: string) =>
  useRead("GetDeployment", { deployment: id }, { refetchInterval: 10_000 })
    .data;

const ConfigTabs = ({ id }: { id: string }) => {
  const deployment = useDeployment(id);
  if (!deployment) return null;
  return <ConfigTabsInner deployment={deployment} />;
};

const ConfigTabsInner = ({
  deployment,
}: {
  deployment: Types.DeploymentListItem;
}) => {
  // const [view, setView] = useAtom(configOrLog);
  const [_view, setView] = useLocalStorage<"Config" | "Log" | "Terminal">(
    "deployment-tabs-v1",
    "Config"
  );
  const { specific } = usePermissions({
    type: "Deployment",
    id: deployment.id,
  });
  const container_exec_disabled =
    useServer(deployment.info.server_id)?.info.container_exec_disabled ?? true;
  const state = deployment.info.state;
  const logsDisabled =
    !specific.includes(Types.SpecificPermission.DockerLog) ||
    state === undefined ||
    state === Types.DeploymentState.Unknown ||
    state === Types.DeploymentState.NotDeployed;
  const terminalDisabled =
    !specific.includes(Types.SpecificPermission.Terminal) ||
    container_exec_disabled ||
    state !== Types.DeploymentState.Running;
  const view =
    (logsDisabled && _view === "Log") ||
    (terminalDisabled && _view === "Terminal")
      ? "Config"
      : _view;

  const tabs = (
    <TabsList className="justify-start w-fit">
      <TabsTrigger value="Config" className="w-[110px]">
        Config
      </TabsTrigger>
      {specific.includes(Types.SpecificPermission.DockerLog) && (
        <TabsTrigger value="Log" className="w-[110px]" disabled={logsDisabled}>
          Log
        </TabsTrigger>
      )}
      {specific.includes(Types.SpecificPermission.Terminal) && (
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
  return (
    <Tabs value={view} onValueChange={setView as any} className="grid gap-4">
      <TabsContent value="Config">
        <DeploymentConfig id={deployment.id} titleOther={tabs} />
      </TabsContent>
      <TabsContent value="Log">
        <DeploymentLogs id={deployment.id} titleOther={tabs} />
      </TabsContent>
      <TabsContent value="Terminal">
        <ContainerTerminal
          query={{
            type: "deployment",
            query: {
              deployment: deployment.id,
              // This is handled inside ContainerTerminal
              shell: "",
            },
          }}
          titleOther={tabs}
        />
      </TabsContent>
    </Tabs>
  );
};

const DeploymentIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useDeployment(id)?.info.state;
  const color = stroke_color_class_by_intention(
    deployment_state_intention(state)
  );
  return <Rocket className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const DeploymentComponents: RequiredResourceComponents = {
  list_item: (id) => useDeployment(id),
  resource_links: (resource) =>
    (resource.config as Types.DeploymentConfig).links,

  Description: () => <>Deploy containers on your servers.</>,

  Dashboard: () => {
    const summary = useRead("GetDeploymentsSummary", {}).data;
    const all = [
      summary?.running ?? 0,
      summary?.stopped ?? 0,
      summary?.unhealthy ?? 0,
      summary?.unknown ?? 0,
    ];
    const [running, stopped, unhealthy, unknown] = all;
    return (
      <DashboardPieChart
        data={[
          all.every((item) => item === 0) && {
            title: "Not Deployed",
            intention: "Neutral",
            value: summary?.not_deployed ?? 0,
          },
          { intention: "Good", value: running, title: "Running" },
          {
            title: "Stopped",
            intention: "Warning",
            value: stopped,
          },
          {
            title: "Unhealthy",
            intention: "Critical",
            value: unhealthy,
          },
          {
            title: "Unknown",
            intention: "Unknown",
            value: unknown,
          },
        ]}
      />
    );
  },

  New: ({ server_id: _server_id, build_id }) => {
    const servers = useRead("ListServers", {}).data;
    const server_id = _server_id
      ? _server_id
      : servers && servers.length === 1
        ? servers[0].id
        : undefined;
    return (
      <NewResource
        type="Deployment"
        server_id={server_id}
        build_id={build_id}
      />
    );
  },

  Table: ({ resources }) => {
    return (
      <DeploymentTable deployments={resources as Types.DeploymentListItem[]} />
    );
  },

  GroupActions: () => (
    <GroupActions
      type="Deployment"
      actions={[
        "PullDeployment",
        "Deploy",
        "RestartDeployment",
        "StopDeployment",
        "DestroyDeployment",
      ]}
    />
  ),

  Icon: ({ id }) => <DeploymentIcon id={id} size={4} />,
  BigIcon: ({ id }) => <DeploymentIcon id={id} size={8} />,

  State: ({ id }) => {
    const state =
      useDeployment(id)?.info.state ?? Types.DeploymentState.Unknown;
    return (
      <StatusBadge text={state} intent={deployment_state_intention(state)} />
    );
  },

  Status: {
    UpdateAvailable: ({ id }) => <UpdateAvailable id={id} />,
  },

  Info: {
    Image: ({ id }) => {
      const config = useFullDeployment(id)?.config;
      const info = useDeployment(id)?.info;
      return info?.build_id ? (
        <ResourceLink type="Build" id={info.build_id} />
      ) : (
        <div className="flex gap-2 items-center text-sm">
          <HardDrive className="w-4 h-4" />
          <div>
            {info?.image.startsWith("sha256:")
              ? (
                  config?.image as Extract<
                    Types.DeploymentImage,
                    { type: "Image" }
                  >
                )?.params.image
              : info?.image || "N/A"}
          </div>
        </div>
      );
    },
    Server: ({ id }) => {
      const info = useDeployment(id)?.info;
      const server = useServer(info?.server_id);
      return server?.id ? (
        <ResourceLink type="Server" id={server?.id} />
      ) : (
        <div className="flex gap-2 items-center text-sm">
          <Server className="w-4 h-4" />
          <div>Unknown Server</div>
        </div>
      );
    },
    Container: ({ id }) => {
      const deployment = useDeployment(id);
      if (
        !deployment ||
        [
          Types.DeploymentState.Unknown,
          Types.DeploymentState.NotDeployed,
        ].includes(deployment.info.state)
      )
        return null;
      return (
        <DockerResourceLink
          type="container"
          name={deployment.name}
          server_id={deployment.info.server_id}
        />
      );
    },
  },

  Actions: {
    RunBuild: ({ id }) => {
      const build_id = useDeployment(id)?.info.build_id;
      if (!build_id) return null;
      return <RunBuild id={build_id} />;
    },
    DeployDeployment,
    PullDeployment,
    RestartDeployment,
    PauseUnpauseDeployment,
    StartStopDeployment,
    DestroyDeployment,
  },

  Page: {},

  Config: ConfigTabs,

  DangerZone: ({ id }) => <DeleteResource type="Deployment" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const deployment = useDeployment(id);

    return (
      <ResourcePageHeader
        intent={deployment_state_intention(deployment?.info.state)}
        icon={<DeploymentIcon id={id} size={8} />}
        type="Deployment"
        id={id}
        name={deployment?.name}
        state={
          deployment?.info.state === Types.DeploymentState.NotDeployed
            ? "Not Deployed"
            : deployment?.info.state
        }
        status={deployment?.info.status}
      />
    );
  },
};

export const UpdateAvailable = ({
  id,
  small,
}: {
  id: string;
  small?: boolean;
}) => {
  const info = useDeployment(id)?.info;
  const state = info?.state ?? Types.DeploymentState.Unknown;
  if (
    !info ||
    !info?.update_available ||
    [Types.DeploymentState.NotDeployed, Types.DeploymentState.Unknown].includes(
      state
    )
  ) {
    return null;
  }
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          className={cn(
            "px-2 py-1 border rounded-md border-blue-400 hover:border-blue-500 opacity-50 hover:opacity-70 transition-colors cursor-pointer flex items-center gap-2",
            small ? "px-2 py-1" : "px-3 py-2"
          )}
        >
          <CircleArrowUp className="w-4 h-4" />
          {!small && (
            <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
              Update Available
            </div>
          )}
        </div>
      </TooltipTrigger>
      <TooltipContent className="w-fit text-sm">
        There is a newer image available
      </TooltipContent>
    </Tooltip>
  );
};
