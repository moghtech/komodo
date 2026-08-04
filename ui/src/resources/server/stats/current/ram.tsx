import { ICONS } from "@/lib/icons";
import { StatBar } from "mogh_ui";
import { Types } from "komodo_client";
import { useFullServer } from "@/resources/server";
import { Group, Text } from "@mantine/core";

export function ServerRamUsage({
  id,
  stats,
}: {
  id: string;
  stats: Types.SystemStats | undefined;
}) {
  const server = useFullServer(id);
  const usedRam = stats?.mem_used_gb;
  const totalRam = stats?.mem_total_gb;
  const zfsArc = stats?.mem_zfs_arc_gb ?? 0;
  const buffCache = stats?.mem_buff_cache_gb ?? 0;
  // = free + reclaimable cache + ARC
  const availableRam =
    usedRam !== undefined && totalRam !== undefined
      ? Math.max(0, totalRam - usedRam)
      : undefined;
  return (
    <StatBar
      title="RAM Usage"
      icon={<ICONS.Memory size="1.3rem" />}
      description={
        usedRam !== undefined &&
        totalRam !== undefined && (
          <>
            <Text span fz="sm">
              Used <b>{usedRam.toFixed(1)} GB</b> of{" "}
              <b>{totalRam.toFixed(1)} GB</b>
            </Text>
            <Group gap="xs" fz="xs" c="dimmed" mt={2}>
              <span>{availableRam?.toFixed(1)} GB available</span>
              {zfsArc > 0 && <span>· ZFS ARC {zfsArc.toFixed(1)} GB</span>}
              {buffCache > 0 && <span>· Cache {buffCache.toFixed(1)} GB</span>}
            </Group>
          </>
        )
      }
      percentage={((usedRam ?? 0) / (totalRam ?? 0)) * 100}
      warning={server?.config?.mem_warning}
      critical={server?.config?.mem_critical}
      flex="1"
    />
  );
}
