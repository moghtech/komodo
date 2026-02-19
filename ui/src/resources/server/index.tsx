import { serverStateIntention, hexColorByIntention } from "@/lib/color";
import { useExecute, useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import ServerTable from "./table";
import NewResource from "@/resources/new";
import ConfirmButton from "@/ui/confirm-button";
import { Prune } from "./executions";
import ServerVersion from "./version";
import { Group, HoverCard } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import ConfirmServerPubkey from "./confirm-pubkey";
import ServerTabs from "./tabs";
import { fmtUpperCamelcase } from "@/lib/formatting";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import ResourceHeaderAction from "../header-action";

export function useServer(id: string | undefined) {
  return useRead("ListServers", {}).data?.find((r) => r.id === id);
}

export function useFullServer(id: string) {
  return useRead("GetServer", { server: id }).data;
}

export const ServerComponents: RequiredResourceComponents<
  Types.ServerConfig,
  Types.ServerInfo,
  Types.ServerListItemInfo
> = {
  useList: () => useRead("ListServers", {}).data,
  useListItem: useServer,
  useFull: useFullServer,

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

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListServers", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(serverStateIntention(state));
    return <ICONS.Server size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const server = useServer(id);
    return (
      <EntityHeader
        intent={serverStateIntention(server?.info.state)}
        icon={ICONS.Server}
        name={server?.name}
        state={fmtUpperCamelcase(server?.info.state ?? "")}
        status={server?.info.region}
        action={
          <ResourceHeaderAction type="Server" id={id} resource={server} />
        }
      />
    );
  },

  State: ({ id }) => {
    let state = useServer(id)?.info.state;
    return <StatusBadge text={state} intent={serverStateIntention(state)} />;
  },
  Info: {
    ServerVersion,
    PublicIp: ({ id }) => {
      const publicIp = useServer(id)?.info.public_ip;
      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Group
              gap="xs"
              onClick={() => {
                publicIp &&
                  navigator.clipboard
                    .writeText(publicIp)
                    .then(() =>
                      notifications.show({ message: "Copied public IP" }),
                    );
              }}
              style={{ cursor: "pointer" }}
            >
              <ICONS.IP size="1rem" />
              {publicIp ?? "Unknown IP"}
            </Group>
          </HoverCard.Target>
          <HoverCard.Dropdown>Public IP (click to copy)</HoverCard.Dropdown>
        </HoverCard>
      );
    },
    Cpu: ({ id }) => {
      const isServerAvailable =
        useServer(id)?.info.state === Types.ServerState.Ok;
      const coreCount =
        useRead(
          "GetSystemInformation",
          { server: id },
          {
            enabled: isServerAvailable,
            refetchInterval: 5000,
          },
        ).data?.core_count ?? 0;
      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Group gap="xs">
              <ICONS.Cpu size="1rem" />
              {coreCount
                ? `${coreCount} Core${coreCount === 1 ? "" : "s"}`
                : "N/A"}
            </Group>
          </HoverCard.Target>
          <HoverCard.Dropdown>CPU Core Count</HoverCard.Dropdown>
        </HoverCard>
      );
    },
    LoadAvg: ({ id }) => {
      const isServerAvailable =
        useServer(id)?.info.state === Types.ServerState.Ok;
      const stats = useRead(
        "GetSystemStats",
        { server: id },
        {
          enabled: isServerAvailable,
          refetchInterval: 5000,
        },
      ).data;

      const one = stats?.load_average?.one;

      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Group gap="xs">
              <ICONS.LoadAvg size="1rem" />
              {one?.toFixed(2) ?? "N/A"}
            </Group>
          </HoverCard.Target>
          <HoverCard.Dropdown>1m Load Average</HoverCard.Dropdown>
        </HoverCard>
      );
    },
    Memory: ({ id }) => {
      const isServerAvailable =
        useServer(id)?.info.state === Types.ServerState.Ok;
      const stats = useRead(
        "GetSystemStats",
        { server: id },
        {
          enabled: isServerAvailable,
          refetchInterval: 5000,
        },
      ).data;
      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Group gap="xs">
              <ICONS.Memory size="1rem" />
              {stats?.mem_total_gb.toFixed(2).concat(" GB") ?? "N/A"}
            </Group>
          </HoverCard.Target>
          <HoverCard.Dropdown>Total Memory</HoverCard.Dropdown>
        </HoverCard>
      );
    },
    Disk: ({ id }) => {
      const isServerAvailable =
        useServer(id)?.info.state === Types.ServerState.Ok;
      const stats = useRead(
        "GetSystemStats",
        { server: id },
        {
          enabled: isServerAvailable,
          refetchInterval: 5000,
        },
      ).data;
      const diskTotalGb = stats?.disks.reduce(
        (acc, curr) => acc + curr.total_gb,
        0,
      );
      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Group gap="xs">
              <ICONS.Disk size="1rem" />
              {diskTotalGb?.toFixed(2).concat(" GB") ?? "N/A"}
            </Group>
          </HoverCard.Target>
          <HoverCard.Dropdown>Total Disk Capacity</HoverCard.Dropdown>
        </HoverCard>
      );
    },
    ConfirmServerPubkey,
  },

  Executions: {
    StartAll: ({ id }) => {
      const server = useServer(id);
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
      const server = useServer(id);
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
          <ConfirmModalWithDisable
            confirmText={server?.name}
            icon={<ICONS.Restart size="1rem" />}
            onConfirm={() => restart({ server: id })}
            disabled={pending}
            loading={pending}
          >
            Restart Containers
          </ConfirmModalWithDisable>
        )
      );
    },
    PauseAll: ({ id }) => {
      const server = useServer(id);
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
          <ConfirmModalWithDisable
            confirmText={server?.name}
            icon={<ICONS.Pause size="1rem" />}
            onConfirm={() => pause({ server: id })}
            disabled={pending}
            loading={pending}
          >
            Pause Containers
          </ConfirmModalWithDisable>
        )
      );
    },
    UnpauseAll: ({ id }) => {
      const server = useServer(id);
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
      const server = useServer(id);
      const { mutateAsync: stop, isPending } = useExecute("StopAllContainers");
      const stopping = useRead(
        "GetServerActionState",
        { server: id },
        { refetchInterval: 5000 },
      ).data?.stopping_containers;
      const pending = isPending || stopping;
      return (
        server && (
          <ConfirmModalWithDisable
            confirmText={server.name}
            icon={<ICONS.Stop size="1rem" />}
            onConfirm={() => stop({ server: id })}
            disabled={pending}
            loading={pending}
          >
            Stop Containers
          </ConfirmModalWithDisable>
        )
      );
    },
    PruneBuildx: ({ id }) => <Prune serverId={id} type="Buildx" />,
    PruneSystem: ({ id }) => <Prune serverId={id} type="System" />,
  },

  Config: ServerTabs,

  Page: {},
};
