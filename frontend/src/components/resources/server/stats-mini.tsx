import { useRead } from "@lib/hooks";
import { cn } from "@lib/utils";
import { Progress } from "@ui/progress";
import { Cpu, Database, MemoryStick } from "lucide-react";

interface ServerStatsMiniProps {
  id: string;
  className?: string;
}

export const ServerStatsMini = ({ id, className }: ServerStatsMiniProps) => {
  const calculatePercentage = (value: number) =>
    Number((value ?? 0).toFixed(2));

  const server = useRead("ListServers", {}).data?.find((s) => s.id === id);
  const serverDetails = useRead("GetServer", { server: id }).data;
  
  // Use server-specific thresholds if available, otherwise use defaults
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

  if (!server || server.info.state === "Disabled" || !stats) {
    return null;
  }

  const cpuPercentage = calculatePercentage(stats.cpu_perc);
  const memoryPercentage = stats.mem_total_gb > 0 
    ? calculatePercentage((stats.mem_used_gb / stats.mem_total_gb) * 100)
    : 0;

  const diskUsed = stats.disks.reduce((acc, disk) => acc + disk.used_gb, 0);
  const diskTotal = stats.disks.reduce((acc, disk) => acc + disk.total_gb, 0);
  const diskPercentage = diskTotal > 0 
    ? calculatePercentage((diskUsed / diskTotal) * 100)
    : 0;

  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <div className="flex items-center gap-2">
        <Cpu className="w-3 h-3 text-muted-foreground" aria-hidden="true" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-muted-foreground">CPU</span>
            <span
              className={cn("text-xs font-medium", getTextColor(cpuPercentage, "cpu"))}
            >
              {cpuPercentage}%
            </span>
          </div>
          <Progress
            value={cpuPercentage}
            className={cn("h-1", "[&>div]:transition-all")}
          />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <MemoryStick className="w-3 h-3 text-muted-foreground" aria-hidden="true" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-muted-foreground">Memory</span>
            <span
              className={cn(
                "text-xs font-medium",
                getTextColor(memoryPercentage, "memory"),
              )}
            >
              {memoryPercentage}%
            </span>
          </div>
          <Progress
            value={memoryPercentage}
            className={cn("h-1", "[&>div]:transition-all")}
          />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Database className="w-3 h-3 text-muted-foreground" aria-hidden="true" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-muted-foreground">Disk</span>
            <span
              className={cn(
                "text-xs font-medium",
                getTextColor(diskPercentage, "disk"),
              )}
            >
              {diskPercentage}%
            </span>
          </div>
          <Progress
            value={diskPercentage}
            className={cn("h-1", "[&>div]:transition-all")}
          />
        </div>
      </div>
    </div>
  );
};
