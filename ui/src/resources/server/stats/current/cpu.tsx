import { useServerStats } from "@/resources/server/hooks";
import { useFullServer } from "@/resources/server";
import { ICONS } from "@/lib/icons";
import StatBar from "@/ui/stat-bar";

export default function ServerCpuUsage({ id }: { id: string }) {
  const server = useFullServer(id);
  const stats = useServerStats(id);
  return (
    <StatBar
      title="CPU Usage"
      icon={<ICONS.Cpu size="1.3rem" />}
      percentage={stats?.cpu_perc}
      warning={server?.config?.cpu_warning}
      critical={server?.config?.cpu_critical}
      flex="1"
    />
  );
}
