import { ResourceLink } from "@components/resources/common";
import { ServerComponents } from "@components/resources/server";
import { DataTable, SortableHeader } from "@ui/data-table";
import { useRead } from "@lib/hooks";
import { useMemo } from "react";

export const ServerMonitoringTable = ({ search = "" }: { search?: string }) => {
  const servers = useRead("ListServers", {}).data;
  const searchSplit = useMemo(
    () => search.toLowerCase().split(" ").filter((t) => t),
    [search]
  );
  const filtered = useMemo(
    () =>
      servers?.filter((s) =>
        searchSplit.length === 0
          ? true
          : searchSplit.every((t) => s.name.toLowerCase().includes(t))
      ) ?? [],
    [servers, searchSplit]
  );
  return (
    <div className="flex flex-col gap-4">
      <DataTable<any, any>
        tableKey="servers-monitoring-v1"
        data={filtered}
        columns={[
          {
            accessorKey: "name",
            size: 250,
            header: ({ column }) => (
              <SortableHeader column={column} title="System" />
            ),
            cell: ({ row }) => (
              <div className="flex items-center gap-2">
                <ResourceLink type="Server" id={row.original.id} />
              </div>
            ),
          },
          {
            header: "CPU",
            size: 200,
            cell: ({ row }) => <CpuCell id={row.original.id} />,
          },
          {
            header: "Memory",
            size: 200,
            cell: ({ row }) => <MemCell id={row.original.id} />,
          },
          {
            header: "Disk",
            size: 200,
            cell: ({ row }) => <DiskCell id={row.original.id} />,
          },
          {
            header: "Load Avg",
            size: 160,
            cell: ({ row }) => <LoadAvgCell id={row.original.id} />,
          },
          {
            header: "Net",
            size: 100,
            cell: ({ row }) => <NetCell id={row.original.id} />,
          },
          {
            header: "Agent",
            size: 160,
            cell: ({ row }) => <ServerComponents.Info.Version id={row.original.id} />,
          },
        ]}
      />
    </div>
  );
};

const useStats = (id: string) =>
  useRead("GetSystemStats", { server: id }, { refetchInterval: 10_000 }).data;

const Bar = ({ valuePerc, intent }: { valuePerc?: number; intent: "Good" | "Warning" | "Critical" }) => {
  const w = Math.max(0, Math.min(100, valuePerc ?? 0)) / 100;
  const color = intent === "Good" ? "bg-green-500" : intent === "Warning" ? "bg-orange-500" : "bg-red-500";
  return (
    <span className="grow min-w-8 block bg-muted h-[1em] relative rounded-sm overflow-hidden">
      <span className={`absolute inset-0 w-full h-full origin-left ${color}`} style={{ transform: `scaleX(${w})` }} />
    </span>
  );
};

const CpuCell = ({ id }: { id: string }) => {
  const stats = useStats(id);
  const cpu = stats?.cpu_perc ?? 0;
  const intent: "Good" | "Warning" | "Critical" = cpu < 60 ? "Good" : cpu < 85 ? "Warning" : "Critical";
  return (
    <div className="flex gap-2 items-center tabular-nums tracking-tight">
      <span className="min-w-8">{cpu.toFixed(2)}%</span>
      <Bar valuePerc={cpu} intent={intent} />
    </div>
  );
};

const MemCell = ({ id }: { id: string }) => {
  const stats = useStats(id);
  const used = stats?.mem_used_gb ?? 0;
  const total = stats?.mem_total_gb ?? 0;
  const perc = total > 0 ? (used / total) * 100 : 0;
  const intent: "Good" | "Warning" | "Critical" = perc < 70 ? "Good" : perc < 90 ? "Warning" : "Critical";
  return (
    <div className="flex gap-2 items-center tabular-nums tracking-tight">
      <span className="min-w-8">{perc.toFixed(1)}%</span>
      <Bar valuePerc={perc} intent={intent} />
    </div>
  );
};

const DiskCell = ({ id }: { id: string }) => {
  const stats = useStats(id);
  const used = stats?.disks?.reduce((acc, d) => acc + (d.used_gb || 0), 0) ?? 0;
  const total = stats?.disks?.reduce((acc, d) => acc + (d.total_gb || 0), 0) ?? 0;
  const perc = total > 0 ? (used / total) * 100 : 0;
  const intent: "Good" | "Warning" | "Critical" = perc < 70 ? "Good" : perc < 90 ? "Warning" : "Critical";
  return (
    <div className="flex gap-2 items-center tabular-nums tracking-tight">
      <span className="min-w-8">{perc.toFixed(1)}%</span>
      <Bar valuePerc={perc} intent={intent} />
    </div>
  );
};

const formatRate = (bytes?: number) => {
  const b = bytes ?? 0;
  const kb = 1024;
  const mb = kb * 1024;
  const gb = mb * 1024;
  if (b >= gb) return `${(b / gb).toFixed(2)} GB/s`;
  if (b >= mb) return `${(b / mb).toFixed(2)} MB/s`;
  if (b >= kb) return `${(b / kb).toFixed(2)} KB/s`;
  return `${b.toFixed(0)} B/s`;
};

const NetCell = ({ id }: { id: string }) => {
  const stats = useStats(id);
  const ingress = stats?.network_ingress_bytes ?? 0;
  const egress = stats?.network_egress_bytes ?? 0;
  return (
    <span className="tabular-nums whitespace-nowrap">
      {formatRate(ingress + egress)}
    </span>
  );
};

const LoadAvgCell = ({ id }: { id: string }) => {
  const stats = useStats(id);
  const one = stats?.load_average?.one;
  const five = stats?.load_average?.five;
  const fifteen = stats?.load_average?.fifteen;
  if (one === undefined || five === undefined || fifteen === undefined) {
    return (
      <div className="w-full flex items-center gap-[.35em] tabular-nums text-muted-foreground tracking-tight">
        <span>N/A</span>
      </div>
    );
  }
  return (
    <div className="w-full flex items-center gap-[.35em] tabular-nums tracking-tight">
      <span>{one.toFixed(2)}</span>
      <span>{five.toFixed(2)}</span>
      <span>{fifteen.toFixed(2)}</span>
    </div>
  );
};


