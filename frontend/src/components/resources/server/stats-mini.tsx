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
    Number((value || 0).toFixed(2));
  const getTextColor = (percentage: number) => {
    if (percentage >= 90) return "text-red-600";
    if (percentage >= 75) return "text-yellow-600";
    return "text-green-600";
  };

  const server = useRead("ListServers", {}).data?.find((s) => s.id === id);
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
  const memoryPercentage = calculatePercentage(
    (stats.mem_used_gb / stats.mem_total_gb) * 100,
  );

  const diskUsed = stats.disks.reduce((acc, disk) => acc + disk.used_gb, 0);
  const diskTotal = stats.disks.reduce((acc, disk) => acc + disk.total_gb, 0);
  const diskPercentage = calculatePercentage((diskUsed / diskTotal) * 100);

  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <div className="flex items-center gap-2">
        <Cpu className="w-3 h-3 text-muted-foreground" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-muted-foreground">CPU</span>
            <span
              className={cn("text-xs font-medium", getTextColor(cpuPercentage))}
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
        <MemoryStick className="w-3 h-3 text-muted-foreground" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-muted-foreground">Memory</span>
            <span
              className={cn(
                "text-xs font-medium",
                getTextColor(memoryPercentage),
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
        <Database className="w-3 h-3 text-muted-foreground" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-muted-foreground">Disk</span>
            <span
              className={cn(
                "text-xs font-medium",
                getTextColor(diskPercentage),
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
