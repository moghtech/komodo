import { ICONS } from "@/theme/icons";
import Section from "@/ui/section";
import StatBar from "@/ui/stat-bar";
import { Group, Stack } from "@mantine/core";
import { Types } from "komodo_client";
import { useFullServer } from "@/resources/server";
import { ServerLoadAverage } from "./load-average";
import ServerNetworkUsage from "./network-usage";

export default function ServerCurrentStats({
  id,
  stats,
}: {
  id: string;
  stats: Types.SystemStats | undefined;
}) {
  const server = useFullServer(id);
  const usedRam = stats?.mem_used_gb;
  const totalRam = stats?.mem_total_gb;
  const usedDisk = stats?.disks.reduce((acc, curr) => (acc += curr.used_gb), 0);
  const totalDisk = stats?.disks.reduce(
    (acc, curr) => (acc += curr.total_gb),
    0,
  );
  return (
    <Section title="Current">
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
          <StatBar
            title="RAM Usage"
            icon={<ICONS.Memory size="1.3rem" />}
            percentage={((usedRam ?? 0) / (totalRam ?? 0)) * 100}
            warning={server?.config?.mem_warning}
            critical={server?.config?.mem_critical}
            flex="1"
          />
        </Stack>
        <Stack w={{ base: "100%", lg: "auto" }}>
          <StatBar
            title="Disk Usage"
            icon={<ICONS.Disk size="1.3rem" />}
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
