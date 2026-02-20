import { useDashboardPreferences, useRead } from "@/lib/hooks";
import { useServer } from ".";
import { Types } from "komodo_client";
import { useMemo } from "react";
import { ICONS } from "@/theme/icons";
import { Center, Group, Overlay, Progress, Stack, Text } from "@mantine/core";
import { ColorIntention, hexColorByIntention } from "@/lib/color";
import { LucideIcon } from "lucide-react";

export interface ServerStatsCardProps {
  id: string;
}

export default function ServerStatsCard({ id }: ServerStatsCardProps) {
  const { preferences } = useDashboardPreferences();
  const server = useServer(id);
  const isServerAvailable = server?.info.state === Types.ServerState.Ok;
  const enabled = preferences.showServerStats && isServerAvailable;

  const serverDetails = useRead(
    "GetServer",
    { server: id },
    {
      enabled,
    },
  ).data;

  const cpuWarning = serverDetails?.config?.cpu_warning ?? 75;
  const cpuCritical = serverDetails?.config?.cpu_critical ?? 90;
  const memWarning = serverDetails?.config?.mem_warning ?? 75;
  const memCritical = serverDetails?.config?.mem_critical ?? 90;
  const diskWarning = serverDetails?.config?.disk_warning ?? 75;
  const diskCritical = serverDetails?.config?.disk_critical ?? 90;

  const intention = (percentage: number, type: "cpu" | "memory" | "disk") => {
    const warning =
      type === "cpu"
        ? cpuWarning
        : type === "memory"
          ? memWarning
          : diskWarning;
    const critical =
      type === "cpu"
        ? cpuCritical
        : type === "memory"
          ? memCritical
          : diskCritical;

    if (percentage >= critical) return "Critical";
    if (percentage >= warning) return "Warning";
    return "Good";
  };

  const stats = useRead(
    "GetSystemStats",
    { server: id },
    {
      enabled,
      refetchInterval: 15_000,
      staleTime: 5_000,
    },
  ).data;

  if (!server) {
    return null;
  }

  const calculations = useMemo(() => {
    const diskUsed = stats
      ? stats.disks.reduce((acc, disk) => acc + disk.used_gb, 0)
      : 0;
    const diskTotal = stats
      ? stats.disks.reduce((acc, disk) => acc + disk.total_gb, 0)
      : 0;

    return {
      cpuPercentage: stats?.cpu_perc ?? 0,
      memoryPercentage:
        stats && stats.mem_total_gb > 0
          ? (stats.mem_used_gb / stats.mem_total_gb) * 100
          : 0,
      diskPercentage: diskTotal > 0 ? (diskUsed / diskTotal) * 100 : 0,
      isUnreachable: !stats || server.info.state === Types.ServerState.NotOk,
      isDisabled: server.info.state === Types.ServerState.Disabled,
    };
  }, [stats, server.info.state]);

  const {
    cpuPercentage,
    memoryPercentage,
    diskPercentage,
    isUnreachable,
    isDisabled,
  } = calculations;

  const statItems = useMemo(
    () => [
      {
        label: "CPU",
        icon: ICONS.Cpu,
        percentage: cpuPercentage,
        type: "cpu" as const,
      },
      {
        label: "Memory",
        icon: ICONS.Memory,
        percentage: memoryPercentage,
        type: "memory" as const,
      },
      {
        label: "Disk",
        icon: ICONS.Disk,
        percentage: diskPercentage,
        type: "disk" as const,
      },
    ],
    [cpuPercentage, memoryPercentage, diskPercentage],
  );
  return (
    <Stack gap="xs" pos="relative">
      {statItems.map((item) => (
        <StatItem
          key={item.label}
          isUnreachable={isUnreachable || isDisabled}
          intention={intention}
          {...item}
        />
      ))}
      {(isUnreachable || isDisabled) && (
        <Center
          renderRoot={(props) => (
            <Overlay color="black" backgroundOpacity={0.5} {...props} />
          )}
        >
          <Text fw="bold" fs="italic">
            {isDisabled ? "Disabled" : "Unreachable"}
          </Text>
        </Center>
      )}
    </Stack>
  );
}

function StatItem({
  icon: Icon,
  label,
  percentage,
  type,
  isUnreachable,
  intention,
}: {
  icon: LucideIcon;
  label: string;
  percentage: number;
  type: "cpu" | "memory" | "disk";
  isUnreachable: boolean;
  intention: (
    percentage: number,
    type: "cpu" | "memory" | "disk",
  ) => ColorIntention;
}) {
  return (
    <Group gap="xs" wrap="nowrap" c="dimmed">
      <Icon size={13} />
      <Stack gap="0.1rem" w="calc(100% - 20px)">
        <Group justify="space-between">
          <Text size="sm">{label}</Text>
          <Text
            size="sm"
            c={
              isUnreachable
                ? "dimmed"
                : hexColorByIntention(intention(percentage, type))
            }
          >
            {isUnreachable ? "N/A" : `${percentage.toFixed(1)}%`}
          </Text>
        </Group>
        <Progress color="bw" value={percentage} size={4} />
      </Stack>
    </Group>
  );
}
