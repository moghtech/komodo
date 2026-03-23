import { serverStateIntention, hexColorByIntention } from "@/lib/color";
import { useExecute, useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
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
import ResourceHeader from "../header";
import { useIsServerAvailable } from "./hooks";
import BatchExecutions from "@/components/batch-executions";
import { ServerLoadAverage } from "./stats/current/load-average";
import { ServerRamUsage } from "./stats/current/ram";
import ServerDiskUsage from "./diskUsage";
import ServerCpuUsage from "./stats/current/cpu";

export function useServer(id: string | undefined) {
  return useRead("ListServers", {}).data?.find((r) => r.id === id);
}

export function useFullServer(id: string) {
  return useRead("GetServer", { server: id }, { refetchInterval: 30_000 }).data;
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
    const summary = useRead(
      "GetServersSummary",
      {},
      { refetchInterval: 10_000 },
    ).data;
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

  BatchExecutions: () => (
    <BatchExecutions
      type="Server"
      executions={[
        ["PruneContainers", ICONS.Container],
        ["PruneNetworks", ICONS.Network],
        ["PruneVolumes", ICONS.Volume],
        ["PruneImages", ICONS.Image],
        ["PruneSystem", ICONS.System],
        ["RestartAllContainers", ICONS.Restart],
        ["StopAllContainers", ICONS.Stop],
      ]}
    />
  ),

  Table: ServerTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const coreVersion = useRead("GetVersion", {}).data?.version;
    const info = useRead("ListServers", {}).data?.find(
      (r) => r.id === id,
    )?.info;
    const state = info?.state;
    const color = noColor
      ? undefined
      : state &&
        hexColorByIntention(
          serverStateIntention(
            state,
            (info.version && coreVersion && info.version !== coreVersion) ||
              false,
          ),
        );
    return <ICONS.Server size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const coreVersion = useRead("GetVersion", {}).data?.version;
    const server = useServer(id);
    const versionMismatch =
      !!coreVersion &&
      !!server?.info.version &&
      coreVersion !== server.info.version;
    return (
      <ResourceHeader
        type="Server"
        id={id}
        resource={server}
        intent={serverStateIntention(
          server?.info.state,
          !!coreVersion &&
            !!server?.info.version &&
            coreVersion !== server.info.version,
        )}
        icon={ICONS.Server}
        name={server?.name}
        state={
          server?.info.state === Types.ServerState.Ok && versionMismatch
            ? "Version Mismatch"
            : fmtUpperCamelcase(server?.info.state ?? "")
        }
        status={server?.info.region}
      />
    );
  },

  State: ({ id }) => {
    const coreVersion = useRead("GetVersion", {}).data?.version;
    const info = useServer(id)?.info;
    return (
      <StatusBadge
        text={info?.state}
        intent={serverStateIntention(
          info?.state,
          !!coreVersion && !!info?.version && coreVersion !== info.version,
        )}
      />
    );
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
      const isServerAvailable = useIsServerAvailable(id);
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
          <HoverCard.Dropdown>
            <ServerCpuUsage id={id} />
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    LoadAvg: ({ id }) => {
      const isServerAvailable = useIsServerAvailable(id);
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
          <HoverCard.Dropdown>
            <ServerLoadAverage id={id} stats={stats} />
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    Memory: ({ id }) => {
      const isServerAvailable = useIsServerAvailable(id);
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
          <HoverCard.Dropdown>
            <ServerRamUsage id={id} stats={stats} />
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    Disk: ({ id }) => {
      const isServerAvailable = useIsServerAvailable(id);
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
          <HoverCard.Dropdown>
            <ServerDiskUsage id={id} stats={stats} withHeader />
          </HoverCard.Dropdown>
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
