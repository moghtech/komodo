import { Section } from "@components/layouts";
import { Card, CardContent, CardHeader, CardTitle } from "@ui/card";
import { Progress } from "@ui/progress";
import {
  Cpu,
  Database,
  Loader2,
  MemoryStick,
  Search,
} from "lucide-react";
import { useLocalStorage, usePermissions, useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { DataTable, SortableHeader } from "@ui/data-table";
import { ReactNode, useMemo, useState } from "react";
import { Input } from "@ui/input";
import { StatChart } from "./stat-chart";
import { useStatsGranularity } from "./hooks";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { DockerResourceLink, ShowHideButton } from "@components/util";
import { filterBySplit } from "@lib/utils";

export const ServerStats = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) => {
  const [interval, setInterval] = useStatsGranularity();

  const { specific } = usePermissions({ type: "Server", id });

  const stats = useRead(
    "GetSystemStats",
    { server: id },
    { refetchInterval: 10_000 }
  ).data;
  const info = useRead("GetSystemInformation", { server: id }).data;

  // Get all the containers with stats
  const containers = useRead("ListDockerContainers", {
    server: id,
  }).data?.filter((c) => c.stats);
  const [showContainers, setShowContainers] = useLocalStorage(
    "stats-show-container-table-v1",
    true
  );
  const [containerSearch, setContainerSearch] = useState("");
  const filteredContainers = filterBySplit(
    containers,
    containerSearch,
    (container) => container.name
  );

  const [showDisks, setShowDisks] = useLocalStorage(
    "stats-show-disks-table-v1",
    true
  );
  const disk_used = stats?.disks.reduce(
    (acc, curr) => (acc += curr.used_gb),
    0
  );
  const disk_total = stats?.disks.reduce(
    (acc, curr) => (acc += curr.total_gb),
    0
  );

  return (
    <Section titleOther={titleOther}>
      <div className="flex flex-col gap-8">
        {/* System Info */}
        <Section title="System Info">
          <DataTable
            tableKey="system-info"
            data={
              info
                ? [{ ...info, mem_total: stats?.mem_total_gb, disk_total }]
                : []
            }
            columns={[
              {
                header: "Hostname",
                accessorKey: "host_name",
              },
              {
                header: "Os",
                accessorKey: "os",
              },
              {
                header: "Kernel",
                accessorKey: "kernel",
              },
              {
                header: "CPU",
                accessorKey: "cpu_brand",
              },
              {
                header: "Core Count",
                accessorFn: ({ core_count }) =>
                  `${core_count} Core${(core_count || 0) > 1 ? "s" : ""}`,
              },
              {
                header: "Total Memory",
                accessorFn: ({ mem_total }) => `${mem_total?.toFixed(2)} GB`,
              },
              {
                header: "Total Disk Size",
                accessorFn: ({ disk_total }) => `${disk_total?.toFixed(2)} GB`,
              },
            ]}
          />
        </Section>

        {/* Current Overview */}
        <Section title="Current">
          <div className="flex flex-col xl:flex-row gap-4">
            <LOAD_AVERAGE id={id} stats={stats} />
            <CPU stats={stats} />
            <RAM stats={stats} />
            <DISK stats={stats} />
            <NETWORK stats={stats} />
          </div>
        </Section>

        {/* Container Breakdown */}
        <Section
          title="Containers"
          actions={
            <div className="flex gap-4 items-center">
              <div className="relative">
                <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
                <Input
                  value={containerSearch}
                  onChange={(e) => setContainerSearch(e.target.value)}
                  placeholder="search..."
                  className="pl-8 w-[200px] lg:w-[300px]"
                />
              </div>
              <ShowHideButton
                show={showContainers}
                setShow={setShowContainers}
              />
            </div>
          }
        >
          {showContainers && (
            <DataTable
              tableKey="container-stats"
              data={filteredContainers}
              columns={[
                {
                  accessorKey: "name",
                  size: 200,
                  header: ({ column }) => (
                    <SortableHeader column={column} title="Name" />
                  ),
                  cell: ({ row }) => (
                    <DockerResourceLink
                      type="container"
                      server_id={id}
                      name={row.original.name}
                    />
                  ),
                },
                {
                  accessorKey: "stats.cpu_perc",
                  size: 100,
                  header: ({ column }) => (
                    <SortableHeader column={column} title="CPU" />
                  ),
                },
                {
                  accessorKey: "stats.mem_perc",
                  size: 200,
                  header: ({ column }) => (
                    <SortableHeader column={column} title="Memory" />
                  ),
                  cell: ({ row }) => (
                    <div className="flex items-center gap-2">
                      {row.original.stats?.mem_perc}
                      <div className="text-muted-foreground text-sm">
                        ({row.original.stats?.mem_usage})
                      </div>
                    </div>
                  ),
                },
                {
                  accessorKey: "stats.net_io",
                  size: 150,
                  header: ({ column }) => (
                    <SortableHeader column={column} title="Net I/O" />
                  ),
                },
                {
                  accessorKey: "stats.block_io",
                  size: 150,
                  header: ({ column }) => (
                    <SortableHeader column={column} title="Block I/O" />
                  ),
                },
                {
                  accessorKey: "stats.pids",
                  size: 100,
                  header: ({ column }) => (
                    <SortableHeader column={column} title="PIDs" />
                  ),
                },
              ]}
            />
          )}
        </Section>

        {/* Current Disk Breakdown */}
        <Section
          title="Disks"
          actions={
            <div className="flex gap-4 items-center">
              <div className="flex gap-2 items-center">
                <div className="text-muted-foreground">Used:</div>
                {disk_used?.toFixed(2)} GB
              </div>
              <div className="flex gap-2 items-center">
                <div className="text-muted-foreground">Total:</div>
                {disk_total?.toFixed(2)} GB
              </div>
              <ShowHideButton show={showDisks} setShow={setShowDisks} />
            </div>
          }
        >
          {showDisks && (
            <DataTable
              sortDescFirst
              tableKey="server-disks"
              data={
                stats?.disks.map((disk) => ({
                  ...disk,
                  percentage: 100 * (disk.used_gb / disk.total_gb),
                })) ?? []
              }
              columns={[
                {
                  header: "Path",
                  cell: ({ row }) => (
                    <div className="overflow-hidden overflow-ellipsis">
                      {row.original.mount}
                    </div>
                  ),
                },
                {
                  accessorKey: "used_gb",
                  header: ({ column }) => (
                    <SortableHeader
                      column={column}
                      title="Used"
                      sortDescFirst
                    />
                  ),
                  cell: ({ row }) => <>{row.original.used_gb.toFixed(2)} GB</>,
                },
                {
                  accessorKey: "total_gb",
                  header: ({ column }) => (
                    <SortableHeader
                      column={column}
                      title="Total"
                      sortDescFirst
                    />
                  ),
                  cell: ({ row }) => <>{row.original.total_gb.toFixed(2)} GB</>,
                },
                {
                  accessorKey: "percentage",
                  header: ({ column }) => (
                    <SortableHeader
                      column={column}
                      title="Percentage"
                      sortDescFirst
                    />
                  ),
                  cell: ({ row }) => (
                    <>{row.original.percentage.toFixed(2)}% Full</>
                  ),
                },
              ]}
            />
          )}
        </Section>

        {specific.includes(Types.SpecificPermission.Processes) && (
          <Processes id={id} />
        )}

        {/* Historical Charts */}
        <Section
          title="Historical"
          actions={
            <div className="flex gap-4 items-center">
              {/* Granularity Dropdown */}
              <div className="flex items-center gap-2">
                <div className="text-muted-foreground">Interval:</div>
                <Select
                  value={interval}
                  onValueChange={(interval) =>
                    setInterval(interval as Types.Timelength)
                  }
                >
                  <SelectTrigger className="w-[150px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[
                      Types.Timelength.FifteenSeconds,
                      Types.Timelength.ThirtySeconds,
                      Types.Timelength.OneMinute,
                      Types.Timelength.FiveMinutes,
                      Types.Timelength.FifteenMinutes,
                      Types.Timelength.ThirtyMinutes,
                      Types.Timelength.OneHour,
                      Types.Timelength.SixHours,
                      Types.Timelength.OneDay,
                    ].map((timelength) => (
                      <SelectItem key={timelength} value={timelength}>
                        {timelength}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          }
        >
          <div className="flex flex-col gap-8">
            <StatChart server_id={id} type="Cpu" className="w-full h-[250px]" />
            <StatChart
              server_id={id}
              type="Memory"
              className="w-full h-[250px]"
            />
            <StatChart
              server_id={id}
              type="Disk"
              className="w-full h-[250px]"
            />
            <StatChart
              server_id={id}
              type="Load Average"
              className="w-full h-[250px]"
            />
            <StatChart
              server_id={id}
              type="Network Ingress"
              className="w-full h-[250px]"
            />
            <StatChart
              server_id={id}
              type="Network Egress"
              className="w-full h-[250px]"
            />
          </div>
        </Section>
      </div>
    </Section>
  );
};

const Processes = ({ id }: { id: string }) => {
  const [show, setShow] = useState(false);
  const [search, setSearch] = useState("");
  const searchSplit = search.toLowerCase().split(" ");
  return (
    <Section
      title="Processes"
      actions={
        <div className="flex gap-4 items-center">
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
          <ShowHideButton show={show} setShow={setShow} />
        </div>
      }
    >
      {show && <ProcessesInner id={id} searchSplit={searchSplit} />}
    </Section>
  );
};

const ProcessesInner = ({
  id,
  searchSplit,
}: {
  id: string;
  searchSplit: string[];
}) => {
  const { data: processes, isPending } = useRead("ListSystemProcesses", {
    server: id,
  });
  const filtered = useMemo(
    () =>
      processes?.filter((process) => {
        if (searchSplit.length === 0) return true;
        const name = process.name.toLowerCase();
        return searchSplit.every((search) => name.includes(search));
      }),
    [processes, searchSplit]
  );
  if (isPending)
    return (
      <div className="flex items-center justify-center h-[200px]">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  if (!processes) return null;
  return (
    <DataTable
      sortDescFirst
      tableKey="server-processes"
      data={filtered ?? []}
      columns={[
        {
          header: "Name",
          accessorKey: "name",
        },
        {
          header: "Exe",
          accessorKey: "exe",
          cell: ({ row }) => (
            <div className="overflow-hidden overflow-ellipsis">
              {row.original.exe}
            </div>
          ),
        },
        {
          accessorKey: "cpu_perc",
          header: ({ column }) => (
            <SortableHeader column={column} title="Cpu" sortDescFirst />
          ),
          cell: ({ row }) => <>{row.original.cpu_perc.toFixed(2)}%</>,
        },
        {
          accessorKey: "mem_mb",
          header: ({ column }) => (
            <SortableHeader column={column} title="Memory" sortDescFirst />
          ),
          cell: ({ row }) => (
            <>
              {row.original.mem_mb > 1000
                ? `${(row.original.mem_mb / 1024).toFixed(2)} GB`
                : `${row.original.mem_mb.toFixed(2)} MB`}
            </>
          ),
        },
      ]}
    />
  );
};

const StatBar = ({
  title,
  icon,
  percentage,
}: {
  title: string;
  icon: ReactNode;
  percentage: number | undefined;
}) => {
  return (
    <Card className="w-full">
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle>{title}</CardTitle>
        <div className="flex gap-2 items-center">
          <div className="text-lg">{percentage?.toFixed(2)}%</div>
          {icon}
        </div>
      </CardHeader>
      <CardContent>
        <Progress value={percentage} className="h-4" />
      </CardContent>
    </Card>
  );
};

const CPU = ({ stats }: { stats: Types.SystemStats | undefined }) => {
  return (
    <StatBar
      title="CPU Usage"
      icon={<Cpu className="w-5 h-5" />}
      percentage={stats?.cpu_perc}
    />
  );
};

const LOAD_AVERAGE = ({ id, stats }: { id: string; stats: Types.SystemStats | undefined }) => {
  if (!stats?.load_average) return null;
  const { one = 0, five = 0, fifteen = 0 } = stats.load_average || {};
  const cores = useRead("GetSystemInformation", { server: id }).data?.core_count;

  const pct = (load: number) => (cores && cores > 0) ? Math.min((load / cores) * 100, 100) : undefined;
  const textColor = (load: number) => {
    const p = pct(load);
    if (p === undefined) return "text-muted-foreground";
    return p <= 50 ? "text-green-600" : p <= 80 ? "text-yellow-600" : "text-red-600";
  };

  return (
    <Card className="w-full">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle>Load Average</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Current Load */}
        <div className="space-y-2">
          <div className="flex items-baseline justify-between">
            <span className={`text-3xl font-bold tabular-nums ${textColor(one)}`}>{one.toFixed(2)}</span>
            <span className="text-sm text-muted-foreground">
              {cores && cores > 0 ? `${(pct(one) ?? 0).toFixed(0)}% of ${cores} cores` : "N/A"}
            </span>
          </div>
          <Progress
            value={pct(one) ?? 0}
            className="h-2"
          />
        </div>

        {/* Time Intervals */}
        <div className="space-y-3">
          <div className="grid grid-cols-3 gap-4 text-sm">
            {[
              ["1m", one],
              ["5m", five],
              ["15m", fifteen],
            ].map(([label, value]) => (
              <div className="space-y-1" key={label as string}>
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground">{label}</span>
                  <span className={`font-medium tabular-nums ${textColor(value as number)}`}>
                    {(value as number).toFixed(2)}
                  </span>
                </div>
                <Progress
                  value={(pct(value as number) ?? 0)}
                  className="h-1"
                />
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const RAM = ({ stats }: { stats: Types.SystemStats | undefined }) => {
  const used = stats?.mem_used_gb;
  const total = stats?.mem_total_gb;
  return (
    <StatBar
      title="RAM Usage"
      icon={<MemoryStick className="w-5 h-5" />}
      percentage={((used ?? 0) / (total ?? 0)) * 100}
    />
  );
};

const DISK = ({ stats }: { stats: Types.SystemStats | undefined }) => {
  const used = stats?.disks.reduce((acc, curr) => (acc += curr.used_gb), 0);
  const total = stats?.disks.reduce((acc, curr) => (acc += curr.total_gb), 0);
  return (
    <StatBar
      title="Disk Usage"
      icon={<Database className="w-5 h-5" />}
      percentage={((used ?? 0) / (total ?? 0)) * 100}
    />
  );
};

const formatBytes = (bytes: number) => {
  const BYTES_PER_KB = 1024;
  const BYTES_PER_MB = 1024 * BYTES_PER_KB;
  const BYTES_PER_GB = 1024 * BYTES_PER_MB;

  if (bytes >= BYTES_PER_GB) {
    return { value: bytes / BYTES_PER_GB, unit: "GB" };
  } else if (bytes >= BYTES_PER_MB) {
    return { value: bytes / BYTES_PER_MB, unit: "MB" };
  } else if (bytes >= BYTES_PER_KB) {
    return { value: bytes / BYTES_PER_KB, unit: "KB" };
  } else {
    return { value: bytes, unit: "bytes" };
  }
};

const NETWORK = ({ stats }: { stats: Types.SystemStats | undefined }) => {
  const ingress = stats?.network_ingress_bytes ?? 0;
  const egress = stats?.network_egress_bytes ?? 0;

  const formattedIngress = formatBytes(ingress);
  const formattedEgress = formatBytes(egress);

  return (
    <Card className="w-full">
      <CardHeader className="flex-row justify-between">
        <CardTitle>Network Usage</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex justify-between items-center mb-4">
          <p className="font-medium">Ingress</p>
          <span className="text-sm text-gray-600">
            {formattedIngress.value.toFixed(2)} {formattedIngress.unit}
          </span>
        </div>
        <div className="flex justify-between items-center">
          <p className="font-medium">Egress</p>
          <span className="text-sm text-gray-600">
            {formattedEgress.value.toFixed(2)} {formattedEgress.unit}
          </span>
        </div>
      </CardContent>
    </Card>
  );
};
