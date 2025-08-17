import { useRead } from "@lib/hooks";
import { cn } from "@lib/utils";
import { Progress } from "@ui/progress";
import { ServerState } from "komodo_client/dist/types";
import { Cpu, Database, MemoryStick, LucideIcon } from "lucide-react";

interface ServerStatsMiniProps {
  id: string;
  className?: string;
}

interface StatItemProps {
  icon: LucideIcon;
  label: string;
  percentage: number;
  type: "cpu" | "memory" | "disk";
  isUnreachable: boolean;
  getTextColor: (percentage: number, type: "cpu" | "memory" | "disk") => string;
}

const StatItem = ({ icon: Icon, label, percentage, type, isUnreachable, getTextColor }: StatItemProps) => (
  <div className="flex items-center gap-2">
    <Icon className="w-3 h-3 text-muted-foreground" aria-hidden="true" />
    <div className="flex-1 min-w-0">
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs text-muted-foreground">{label}</span>
        <span
          className={cn(
            "text-xs font-medium",
            isUnreachable ? "text-muted-foreground" : getTextColor(percentage, type)
          )}
        >
          {isUnreachable ? "N/A" : `${percentage}%`}
        </span>
      </div>
      <Progress
        value={isUnreachable ? 0 : percentage}
        className={cn("h-1", "[&>div]:transition-all")}
      />
    </div>
  </div>
);

export const ServerStatsMini = ({ id, className }: ServerStatsMiniProps) => {
  const calculatePercentage = (value: number) =>
    Number((value ?? 0).toFixed(2));

  const server = useRead("ListServers", {}).data?.find((s) => s.id === id);
  const serverDetails = useRead("GetServer", { server: id }).data;
  
  const cpuWarning = serverDetails?.config?.cpu_warning ?? 75;
  const cpuCritical = serverDetails?.config?.cpu_critical ?? 90;
  const memWarning = serverDetails?.config?.mem_warning ?? 75;
  const memCritical = serverDetails?.config?.mem_critical ?? 90;
  const diskWarning = serverDetails?.config?.disk_warning ?? 75;
  const diskCritical = serverDetails?.config?.disk_critical ?? 90;

  const getTextColor = (percentage: number, type: "cpu" | "memory" | "disk") => {
    const warning = type === "cpu" ? cpuWarning : type === "memory" ? memWarning : diskWarning;
    const critical = type === "cpu" ? cpuCritical : type === "memory" ? memCritical : diskCritical;
    
    if (percentage >= critical) return "text-red-600";
    if (percentage >= warning) return "text-yellow-600";
    return "text-green-600";
  };
  const stats = useRead(
    "GetSystemStats",
    { server: id },
    {
      enabled: server ? server.info.state !== "Disabled" : false,
      refetchInterval: 10_000,
    },
  ).data;

  if (!server || server.info.state === "Disabled") {
    return null;
  }

  const cpuPercentage = stats ? calculatePercentage(stats.cpu_perc) : 0;
  const memoryPercentage = stats && stats.mem_total_gb > 0 ? calculatePercentage((stats.mem_used_gb / stats.mem_total_gb) * 100) : 0;

  const diskUsed = stats ? stats.disks.reduce((acc, disk) => acc + disk.used_gb, 0) : 0;
  const diskTotal = stats ? stats.disks.reduce((acc, disk) => acc + disk.total_gb, 0) : 0;
  const diskPercentage = diskTotal > 0? calculatePercentage((diskUsed / diskTotal) * 100) : 0;
    
  const isUnreachable = !stats || server.info.state === ServerState.NotOk;
  const unreachableClass = isUnreachable ? "opacity-50" : "";

  const statItems = [
    { icon: Cpu, label: "CPU", percentage: cpuPercentage, type: "cpu" as const },
    { icon: MemoryStick, label: "Memory", percentage: memoryPercentage, type: "memory" as const },
    { icon: Database, label: "Disk", percentage: diskPercentage, type: "disk" as const },
  ];

  return (
    <div className={cn("flex flex-col gap-2", unreachableClass, className)}>
      {isUnreachable && (
        <div className="text-xs text-muted-foreground italic text-center">
          Unreachable
        </div>
      )}
      {statItems.map((item) => (
        <StatItem
          key={item.label}
          icon={item.icon}
          label={item.label}
          percentage={item.percentage}
          type={item.type}
          isUnreachable={isUnreachable}
          getTextColor={getTextColor}
        />
      ))}
    </div>
  );
};
