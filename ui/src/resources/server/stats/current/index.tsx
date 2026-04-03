import { ICONS } from "@/lib/icons";
import { Section } from "mogh_ui";
import StatBar from "@/ui/stat-bar";
import { Group, Stack } from "@mantine/core";
import { Types } from "komodo_client";
import { useFullServer } from "@/resources/server";
import { ServerLoadAverage } from "./load-average";
import ServerNetworkUsage from "./network-usage";
import { Clock } from "lucide-react";
import { ServerRamUsage } from "./ram";

export default function ServerCurrentStats({
  id,
  stats,
}: {
  id: string;
  stats: Types.SystemStats | undefined;
}) {
  const server = useFullServer(id);
  const usedDisk = stats?.disks.reduce((acc, curr) => (acc += curr.used_gb), 0);
  const totalDisk = stats?.disks.reduce(
    (acc, curr) => (acc += curr.total_gb),
    0,
  );
  return (
    <Section title="Current" icon={<Clock size="1.3rem" />}>
      <Group align="stretch">
        <ServerLoadAverage id={id} stats={stats} />
        <Stack w={{ base: "100%", lg: "auto" }}>
          <StatBar
            title="CPU Usage"
            icon={<ICONS.Cpu size="1.3rem" />}
            percentage={stats?.cpu_perc}
            warning={server?.config?.cpu_warning}
            critical={server?.config?.cpu_critical}
            flex="1"
          />
          <ServerRamUsage id={id} stats={stats} />
        </Stack>
        <Stack w={{ base: "100%", lg: "auto" }}>
          <StatBar
            title="Disk Usage"
            icon={<ICONS.Disk size="1.3rem" />}
            description={
              usedDisk &&
              totalDisk && (
                <>
                  <b>{usedDisk?.toFixed(1)} GB</b> of{" "}
                  <b>{totalDisk?.toFixed(1)} GB</b> in use
                </>
              )
            }
            percentage={((usedDisk ?? 0) / (totalDisk ?? 0)) * 100}
            warning={server?.config?.disk_warning}
            critical={server?.config?.disk_critical}
          />
          <ServerNetworkUsage stats={stats} />
        </Stack>
      </Group>
    </Section>
  );
}
