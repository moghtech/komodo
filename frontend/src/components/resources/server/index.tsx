import { useExecute, useLocalStorage, useRead, useUser } from "@lib/hooks";
import { cn } from "@lib/utils";
import { Types } from "komodo_client";
import { RequiredResourceComponents } from "@types";
import {
  Server,
  Cpu,
  MemoryStick,
  Database,
  Milestone,
  Play,
  RefreshCcw,
  Pause,
  Square,
} from "lucide-react";
import { Section } from "@components/layouts";
import { Prune, RenameServer } from "./actions";
import {
  server_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { ServerConfig } from "./config";
import { DeploymentTable } from "../deployment/table";
import { ServerTable } from "./table";
import { DeleteResource, NewResource } from "../common";
import {
  ActionWithDialog,
  ConfirmButton,
  ResourcePageHeader,
  StatusBadge,
} from "@components/util";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";
import { RepoTable } from "../repo/table";
import { DashboardPieChart } from "@pages/home/dashboard";
import { StackTable } from "../stack/table";
import { ResourceComponents } from "..";
import { ServerInfo } from "./info";
import { ServerStats } from "./stats";

export const useServer = (id?: string) =>
  useRead("ListServers", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

export const useFullServer = (id: string) =>
  useRead("GetServer", { server: id }, { refetchInterval: 10_000 }).data;

const Icon = ({ id, size }: { id?: string; size: number }) => {
  const state = useServer(id)?.info.state;
  return (
    <Server
      className={cn(
        `w-${size} h-${size}`,
        state && stroke_color_class_by_intention(server_state_intention(state))
      )}
    />
  );
};

const ConfigStatsDockerResources = ({ id }: { id: string }) => {
  const [view, setView] = useLocalStorage<
    "Config" | "Stats" | "Docker" | "Resources"
  >(`server-${id}-tab`, "Config");

  const is_admin = useUser().data?.admin ?? false;
  const disable_non_admin_create =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;

  const deployments =
    useRead("ListDeployments", {}).data?.filter(
      (deployment) => deployment.info.server_id === id
    ) ?? [];
  const noDeployments = deployments.length === 0;
  const repos =
    useRead("ListRepos", {}).data?.filter(
      (repo) => repo.info.server_id === id
    ) ?? [];
  const noRepos = repos.length === 0;
  const stacks =
    useRead("ListStacks", {}).data?.filter(
      (stack) => stack.info.server_id === id
    ) ?? [];
  const noStacks = stacks.length === 0;

  const noResources = noDeployments && noRepos && noStacks;

  const currentView = view === "Resources" && noResources ? "Config" : view;

  const tabsList = (
    <TabsList className="justify-start w-fit">
      <TabsTrigger value="Config" className="w-[110px]">
        Config
      </TabsTrigger>

      <TabsTrigger value="Stats" className="w-[110px]">
        Stats
      </TabsTrigger>

      <TabsTrigger value="Docker" className="w-[110px]">
        Docker
      </TabsTrigger>

      <TabsTrigger
        value="Resources"
        className="w-[110px]"
        disabled={noResources}
      >
        Resources
      </TabsTrigger>
    </TabsList>
  );
  return (
    <Tabs
      value={currentView}
      onValueChange={setView as any}
      className="grid gap-4"
    >
      <TabsContent value="Config">
        <ServerConfig id={id} titleOther={tabsList} />
      </TabsContent>

      <TabsContent value="Stats">
        <ServerStats id={id} titleOther={tabsList} />
      </TabsContent>

      <TabsContent value="Docker">
        <ServerInfo id={id} titleOther={tabsList} />
      </TabsContent>

      <TabsContent value="Resources">
        <Section titleOther={tabsList}>
          <Section
            title="Deployments"
            actions={
              (is_admin || !disable_non_admin_create) && (
                <ResourceComponents.Deployment.New server_id={id} />
              )
            }
          >
            <DeploymentTable deployments={deployments} />
          </Section>
          <Section
            title="Stacks"
            actions={
              (is_admin || !disable_non_admin_create) && (
                <ResourceComponents.Stack.New server_id={id} />
              )
            }
          >
            <StackTable stacks={stacks} />
          </Section>
          <Section
            title="Repos"
            actions={
              (is_admin || !disable_non_admin_create) && (
                <ResourceComponents.Repo.New server_id={id} />
              )
            }
          >
            <RepoTable repos={repos} />
          </Section>
        </Section>
      </TabsContent>
    </Tabs>
  );
};

export const ServerComponents: RequiredResourceComponents = {
  list_item: (id) => useServer(id),
  resource_links: (resource) => (resource.config as Types.ServerConfig).links,

  Description: () => (
    <>Connect servers for alerting, building, and deploying.</>
  ),

  Dashboard: () => {
    const summary = useRead("GetServersSummary", {}).data;
    return (
      <DashboardPieChart
        data={[
          { title: "Healthy", intention: "Good", value: summary?.healthy ?? 0 },
          {
            title: "Unhealthy",
            intention: "Critical",
            value: summary?.unhealthy ?? 0,
          },
          {
            title: "Disabled",
            intention: "Neutral",
            value: summary?.disabled ?? 0,
          },
        ]}
      />
    );
  },

  New: () => {
    const user = useUser().data;
    if (!user) return null;
    if (!user.admin && !user.create_server_permissions) return null;
    return <NewResource type="Server" />;
  },

  Table: ({ resources }) => (
    <ServerTable servers={resources as Types.ServerListItem[]} />
  ),

  Icon: ({ id }) => <Icon id={id} size={4} />,
  BigIcon: ({ id }) => <Icon id={id} size={8} />,

  State: ({ id }) => {
    const state = useServer(id)?.info.state;
    return <StatusBadge text={state} intent={server_state_intention(state)} />;
  },

  Status: {},

  Info: {
    Version: ({ id }) => {
      const version = useRead(
        "GetPeripheryVersion",
        { server: id },
        { refetchInterval: 5000 }
      ).data?.version;
      const _version =
        version === undefined || version === "unknown" ? "unknown" : version;
      return (
        <div className="flex items-center gap-2">
          <Milestone className="w-4 h-4" />
          {_version}
        </div>
      );
    },
    Cpu: ({ id }) => {
      const server = useServer(id);
      const core_count =
        useRead(
          "GetSystemInformation",
          { server: id },
          {
            enabled: server ? server.info.state !== "Disabled" : false,
            refetchInterval: 5000,
          }
        ).data?.core_count ?? 0;
      return (
        <div className="flex gap-2 items-center">
          <Cpu className="w-4 h-4" />
          {core_count || "N/A"} Core{core_count > 1 ? "s" : ""}
        </div>
      );
    },
    Mem: ({ id }) => {
      const server = useServer(id);
      const stats = useRead(
        "GetSystemStats",
        { server: id },
        {
          enabled: server ? server.info.state !== "Disabled" : false,
          refetchInterval: 5000,
        }
      ).data;
      return (
        <div className="flex gap-2 items-center">
          <MemoryStick className="w-4 h-4" />
          {stats?.mem_total_gb.toFixed(2) ?? "N/A"} GB
        </div>
      );
    },
    Disk: ({ id }) => {
      const server = useServer(id);
      const stats = useRead(
        "GetSystemStats",
        { server: id },
        {
          enabled: server ? server.info.state !== "Disabled" : false,
          refetchInterval: 5000,
        }
      ).data;
      const disk_total_gb = stats?.disks.reduce(
        (acc, curr) => acc + curr.total_gb,
        0
      );
      return (
        <div className="flex gap-2 items-center">
          <Database className="w-4 h-4" />
          {disk_total_gb?.toFixed(2) ?? "N/A"} GB
        </div>
      );
    },
  },

  Actions: {
    StartAll: ({ id }) => {
      const server = useServer(id);
      const { mutate, isPending } = useExecute("StartAllContainers");
      const starting = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 }
      ).data?.starting_containers;
      const dontShow =
        useRead("ListDockerContainers", {
          server: id,
        }).data?.every(
          (container) =>
            container.state === Types.ContainerStateStatusEnum.Running
        ) ?? true;
      if (dontShow) {
        return null;
      }
      const pending = isPending || starting;
      return (
        server && (
          <ConfirmButton
            title="Start Containers"
            icon={<Play className="w-4 h-4" />}
            onClick={() => mutate({ server: id })}
            loading={pending}
            disabled={pending}
          />
        )
      );
    },
    RestartAll: ({ id }) => {
      const server = useServer(id);
      const { mutate, isPending } = useExecute("RestartAllContainers");
      const restarting = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 }
      ).data?.restarting_containers;
      const pending = isPending || restarting;
      return (
        server && (
          <ActionWithDialog
            name={server?.name}
            title="Restart Containers"
            icon={<RefreshCcw className="w-4 h-4" />}
            onClick={() => mutate({ server: id })}
            disabled={pending}
            loading={pending}
          />
        )
      );
    },
    PauseAll: ({ id }) => {
      const server = useServer(id);
      const { mutate, isPending } = useExecute("PauseAllContainers");
      const pausing = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 }
      ).data?.pausing_containers;
      const dontShow =
        useRead("ListDockerContainers", {
          server: id,
        }).data?.every(
          (container) =>
            container.state !== Types.ContainerStateStatusEnum.Running
        ) ?? true;
      if (dontShow) {
        return null;
      }
      const pending = isPending || pausing;
      return (
        server && (
          <ActionWithDialog
            name={server?.name}
            title="Pause Containers"
            icon={<Pause className="w-4 h-4" />}
            onClick={() => mutate({ server: id })}
            disabled={pending}
            loading={pending}
          />
        )
      );
    },
    UnpauseAll: ({ id }) => {
      const server = useServer(id);
      const { mutate, isPending } = useExecute("UnpauseAllContainers");
      const unpausing = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 }
      ).data?.unpausing_containers;
      const dontShow =
        useRead("ListDockerContainers", {
          server: id,
        }).data?.every(
          (container) =>
            container.state !== Types.ContainerStateStatusEnum.Paused
        ) ?? true;
      if (dontShow) {
        return null;
      }
      const pending = isPending || unpausing;
      return (
        server && (
          <ConfirmButton
            title="Unpause Containers"
            icon={<Play className="w-4 h-4" />}
            onClick={() => mutate({ server: id })}
            loading={pending}
            disabled={pending}
          />
        )
      );
    },
    StopAll: ({ id }) => {
      const server = useServer(id);
      const { mutate, isPending } = useExecute("StopAllContainers");
      const stopping = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 }
      ).data?.stopping_containers;
      const pending = isPending || stopping;
      return (
        server && (
          <ActionWithDialog
            name={server.name}
            title="Stop Containers"
            icon={<Square className="w-4 h-4" />}
            onClick={() => mutate({ server: id })}
            disabled={pending}
            loading={pending}
          />
        )
      );
    },
    PruneBuildx: ({ id }) => <Prune server_id={id} type="Buildx" />,
    PruneSystem: ({ id }) => <Prune server_id={id} type="System" />,
  },

  Page: {},

  Config: ConfigStatsDockerResources,

  DangerZone: ({ id }) => (
    <>
      <RenameServer id={id} />
      <DeleteResource type="Server" id={id} />
    </>
  ),

  ResourcePageHeader: ({ id }) => {
    const server = useServer(id);

    return (
      <ResourcePageHeader
        intent={server_state_intention(server?.info.state)}
        icon={<Icon id={id} size={8} />}
        name={server?.name}
        state={
          server?.info.state === Types.ServerState.NotOk
            ? "Not Ok"
            : server?.info.state
        }
        status={server?.info.region}
      />
    );
  },
};
