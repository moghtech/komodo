import { serverStateIntention, hexColorByIntention } from "@/lib/color";
import { useExecute, useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";
import ServerTable from "./table";
import NewResource from "@/resources/new";
import ServerConfig from "./config";
import ConfirmButton from "@/ui/confirm-button";
import ConfirmModal from "@/ui/confirm-modal";
import { Prune } from "./executions";

export const ServerComponents: RequiredResourceComponents<
  Types.ServerConfig,
  Types.ServerInfo,
  Types.ServerListItemInfo
> = {
  useList: () => useRead("ListServers", {}).data,
  useListItem: (id) => ServerComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetServer", { server: id }).data,

  useResourceLinks: (server) => server?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetServersSummary", {}).data;
    return [
      { title: "Healthy", intention: "Good", value: summary?.healthy ?? 0 },
      {
        title: "Warning",
        intention: "Warning",
        value: summary?.warning ?? 0,
      },
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
    ];
  },

  Description: () => (
    <>Connect servers for alerting, building, and deploying.</>
  ),

  New: () => <NewResource type="Server" />,

  GroupExecutions: () => <></>,

  Table: ServerTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListServers", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = state && hexColorByIntention(serverStateIntention(state));
    return <ICONS.Server size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const server = ServerComponents.useListItem(id);
    return (
      <ResourceHeader
        intent={serverStateIntention(server?.info.state)}
        icon={<ServerComponents.Icon id={id} size="2rem" />}
        name={server?.name}
        state={server?.info.state}
        status={server?.info.region}
      />
    );
  },

  State: ({ id }) => {
    let state = ServerComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={serverStateIntention(state)} />;
  },
  Status: {},
  Info: {},

  Executions: {
    StartAll: ({ id }) => {
      const server = ServerComponents.useListItem(id);
      const { mutate, isPending } = useExecute("StartAllContainers");
      const starting = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 },
      ).data?.starting_containers;
      const dontShow =
        useRead("ListDockerContainers", {
          server: id,
        }).data?.every(
          (container) =>
            container.state === Types.ContainerStateStatusEnum.Running,
        ) ?? true;
      if (dontShow) {
        return null;
      }
      const pending = isPending || starting;
      return (
        server && (
          <ConfirmButton
            icon={<ICONS.Start size="1rem" />}
            onClick={() => mutate({ server: id })}
            loading={pending}
            disabled={pending}
          >
            Start Containers
          </ConfirmButton>
        )
      );
    },
    RestartAll: ({ id }) => {
      const server = ServerComponents.useListItem(id);
      const { mutateAsync: restart, isPending } = useExecute(
        "RestartAllContainers",
      );
      const restarting = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 },
      ).data?.restarting_containers;
      const pending = isPending || restarting;
      return (
        server && (
          <ConfirmModal
            confirmText={server?.name}
            icon={<ICONS.Restart size="1rem" />}
            onConfirm={() => restart({ server: id })}
            disabled={pending}
            loading={pending}
          >
            Restart Containers
          </ConfirmModal>
        )
      );
    },
    PauseAll: ({ id }) => {
      const server = ServerComponents.useListItem(id);
      const { mutateAsync: pause, isPending } =
        useExecute("PauseAllContainers");
      const pausing = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 },
      ).data?.pausing_containers;
      const dontShow =
        useRead("ListDockerContainers", {
          server: id,
        }).data?.every(
          (container) =>
            container.state !== Types.ContainerStateStatusEnum.Running,
        ) ?? true;
      if (dontShow) {
        return null;
      }
      const pending = isPending || pausing;
      return (
        server && (
          <ConfirmModal
            confirmText={server?.name}
            icon={<ICONS.Pause size="1rem" />}
            onConfirm={() => pause({ server: id })}
            disabled={pending}
            loading={pending}
          >
            Pause Containers
          </ConfirmModal>
        )
      );
    },
    UnpauseAll: ({ id }) => {
      const server = ServerComponents.useListItem(id);
      const { mutateAsync: unpause, isPending } = useExecute(
        "UnpauseAllContainers",
      );
      const unpausing = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 },
      ).data?.unpausing_containers;
      const dontShow =
        useRead("ListDockerContainers", {
          server: id,
        }).data?.every(
          (container) =>
            container.state !== Types.ContainerStateStatusEnum.Paused,
        ) ?? true;
      if (dontShow) {
        return null;
      }
      const pending = isPending || unpausing;
      return (
        server && (
          <ConfirmButton
            icon={<ICONS.Start size="1rem" />}
            onClick={() => unpause({ server: id })}
            loading={pending}
            disabled={pending}
          >
            Unpause Containers
          </ConfirmButton>
        )
      );
    },
    StopAll: ({ id }) => {
      const server = ServerComponents.useListItem(id);
      const { mutateAsync: stop, isPending } = useExecute("StopAllContainers");
      const stopping = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 },
      ).data?.stopping_containers;
      const pending = isPending || stopping;
      return (
        server && (
          <ConfirmModal
            confirmText={server.name}
            icon={<ICONS.Stop size="1rem" />}
            onConfirm={() => stop({ server: id })}
            disabled={pending}
            loading={pending}
          >
            Stop Containers
          </ConfirmModal>
        )
      );
    },
    PruneBuildx: ({ id }) => <Prune serverId={id} type="Buildx" />,
    PruneSystem: ({ id }) => <Prune serverId={id} type="System" />,
  },

  Config: ServerConfig,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
