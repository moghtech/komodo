import TagsFilter from "@/components/tags/filter";
import {
  komodo_client,
  useDebouncedTermSearch,
  useRead,
  useSetTitle,
  useTagsFilter,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import ResourceMultiSelector from "@/resources/multi-selector";
import { useStatsGranularity } from "@/resources/server/hooks";
import StatsServerTable from "@/resources/server/table/stats";
import {
  Center,
  Group,
  Loader,
  Select,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { keepPreviousData, useQueries } from "@tanstack/react-query";
import { Types } from "komodo_client";
import { ChartLine, Download, Upload } from "lucide-react";
import {
  fmtSizeBytes,
  InfoCard,
  Page,
  SearchInput,
  Section,
  StatBar,
} from "mogh_ui";
import { ReactNode, useMemo } from "react";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

/**
 * Fixed categorical palette for per-server chart series (Mantine hues,
 * CVD-separation and contrast validated on light and dark surfaces).
 * Colors are assigned by position in the full server list, so filtering
 * never repaints the remaining servers.
 */
const SERVER_COLORS = [
  "#1c7ed6", // blue.7
  "#e8590c", // orange.8
  "#0ca678", // teal.7
  "#be4bdb", // grape.6
  "#f03e3e", // red.7
  "#1098ad", // cyan.7
  "#7950f2", // violet.6
  "#d6336c", // pink.7
];

/** Beyond this many series, lines become unreadable and colors would repeat. */
const MAX_CHART_SERVERS = SERVER_COLORS.length;

type StatType =
  | "Cpu"
  | "Memory"
  | "Disk"
  | "Network Ingress"
  | "Network Egress"
  | "Load Average";

const STAT_TYPES: [StatType, ReactNode][] = [
  ["Cpu", <ICONS.Cpu size="1.1rem" />],
  ["Memory", <ICONS.Memory size="1.1rem" />],
  ["Disk", <ICONS.Disk size="1.1rem" />],
  ["Load Average", <ICONS.LoadAvg size="1.1rem" />],
  ["Network Ingress", <Download size="1.1rem" />],
  ["Network Egress", <Upload size="1.1rem" />],
];

export default function Stats() {
  useSetTitle("Stats");

  // Server ids
  const [selectedServers, setSelectedServers] = useLocalStorage<string[]>({
    key: "stats.selected-servers.v1",
    defaultValue: [],
  });

  const { search, setSearch, terms } = useDebouncedTermSearch({
    localStorageKey: "stats.search.v1",
  });
  const tags = useTagsFilter();

  const servers =
    useRead(
      "ListServers",
      { query: { terms, tags }, limit: 0 },
      { refetchInterval: 30_000, placeholderData: keepPreviousData },
    ).data ?? [];

  // Stable color per server, keyed on the full (unfiltered) list
  // so the assignment survives filtering.
  const serverColors = useMemo(() => {
    const colors = new Map<string, string>();
    servers.forEach((server, i) =>
      colors.set(server.id, SERVER_COLORS[i % SERVER_COLORS.length]),
    );
    return colors;
  }, [servers]);

  const filteredServers = useMemo(
    () =>
      selectedServers.length
        ? servers.filter((server) => selectedServers.includes(server.name))
        : servers,
    [servers, selectedServers],
  );

  const Content = useMemo(
    () =>
      servers.length === 0 ? (
        <Text c="dimmed">No Servers found.</Text>
      ) : (
        <>
          <CurrentStatsSummary servers={filteredServers} />
          <HistoricalStats
            servers={filteredServers}
            serverColors={serverColors}
          />
          <Section title="Servers" icon={<ICONS.Server size="1.3rem" />}>
            <StatsServerTable resources={filteredServers} />
          </Section>
        </>
      ),
    [servers, filteredServers],
  );

  return (
    <Page
      title="Stats"
      icon={ICONS.Stats}
      description="System stats across all servers."
    >
      <Stack>
        <Group justify="space-between">
          <ResourceMultiSelector
            type="Server"
            value={selectedServers}
            onChange={setSelectedServers}
          />

          <Group w={{ base: "100%", xs: "fit-content" }}>
            <TagsFilter />
            <SearchInput value={search} onSearch={setSearch} />
          </Group>
        </Group>

        {Content}
      </Stack>
    </Page>
  );
}

function CurrentStatsSummary({ servers }: { servers: Types.ServerListItem[] }) {
  const statsQueries = useQueries({
    queries: servers.map((server) => ({
      // Matches the useRead("GetSystemStats", ...) query key,
      // so the cache is shared with the server table cells below.
      queryKey: ["GetSystemStats", { server: server.id }],
      queryFn: () =>
        komodo_client().read("GetSystemStats", { server: server.id }),
      enabled: server.info.state === Types.ServerState.Ok,
      refetchInterval: 10_000,
    })),
  });

  const okCount = servers.filter(
    (server) => server.info.state === Types.ServerState.Ok,
  ).length;
  const disabledCount = servers.filter(
    (server) => server.info.state === Types.ServerState.Disabled,
  ).length;
  const unreachableCount = servers.length - okCount - disabledCount;

  const stats = statsQueries.flatMap((query) => query.data ?? []);
  const cpu =
    stats.length > 0
      ? stats.reduce((acc, s) => acc + s.cpu_perc, 0) / stats.length
      : 0;
  const memUsed = stats.reduce((acc, s) => acc + s.mem_used_gb, 0);
  const memTotal = stats.reduce((acc, s) => acc + s.mem_total_gb, 0);
  const diskUsed = stats.reduce(
    (acc, s) => acc + s.disks.reduce((acc, disk) => acc + disk.used_gb, 0),
    0,
  );
  const diskTotal = stats.reduce(
    (acc, s) => acc + s.disks.reduce((acc, disk) => acc + disk.total_gb, 0),
    0,
  );

  return (
    <Section>
      <Group align="stretch">
        <InfoCard
          title="Servers"
          info={<ICONS.Server size="1.3rem" />}
          w={{ base: "100%", lg: 300 }}
          gap="0.2rem"
          justify="space-between"
        >
          <Text c="dimmed" size="sm">
            <b>{okCount}</b> of <b>{servers.length}</b> reachable
          </Text>
          <Group gap="md">
            {unreachableCount > 0 && (
              <Text c="red" size="sm">
                {unreachableCount} unreachable
              </Text>
            )}
            {disabledCount > 0 && (
              <Text c="dimmed" size="sm">
                {disabledCount} disabled
              </Text>
            )}
          </Group>
        </InfoCard>
        <StatBar
          title="CPU Usage"
          icon={<ICONS.Cpu size="1.3rem" />}
          description={
            <>
              Average across <b>{stats.length}</b> reachable server
              {stats.length === 1 ? "" : "s"}
            </>
          }
          percentage={cpu}
          warning={75}
          critical={90}
        />
        <StatBar
          title="Memory Usage"
          icon={<ICONS.Memory size="1.3rem" />}
          description={
            memTotal > 0 && (
              <>
                <b>{memUsed.toFixed(1)} GB</b> of{" "}
                <b>{memTotal.toFixed(1)} GB</b> in use
              </>
            )
          }
          percentage={memTotal > 0 ? (memUsed / memTotal) * 100 : 0}
          warning={75}
          critical={90}
        />
        <StatBar
          title="Disk Usage"
          icon={<ICONS.Disk size="1.3rem" />}
          description={
            diskTotal > 0 && (
              <>
                <b>{diskUsed.toFixed(1)} GB</b> of{" "}
                <b>{diskTotal.toFixed(1)} GB</b> in use
              </>
            )
          }
          percentage={diskTotal > 0 ? (diskUsed / diskTotal) * 100 : 0}
          warning={75}
          critical={90}
        />
      </Group>
    </Section>
  );
}

function HistoricalStats({
  servers,
  serverColors,
}: {
  servers: Types.ServerListItem[];
  serverColors: Map<string, string>;
}) {
  const [granularity, setGranularity] = useStatsGranularity();

  const chartServers = servers
    .filter((server) => server.info.state === Types.ServerState.Ok)
    .slice(0, MAX_CHART_SERVERS);

  const historicalQueries = useQueries({
    queries: chartServers.map((server) => ({
      queryKey: [
        "GetHistoricalServerStats",
        { server: server.id, granularity },
      ],
      queryFn: () =>
        komodo_client().read("GetHistoricalServerStats", {
          server: server.id,
          granularity,
        }),
      refetchInterval:
        granularity === Types.Timelength.FiveSeconds
          ? 5_000
          : granularity === Types.Timelength.FifteenSeconds
            ? 10_000
            : 15_000,
    })),
  });

  const series = chartServers.map((server, i) => ({
    name: server.name,
    color: serverColors.get(server.id) ?? SERVER_COLORS[0],
    stats: historicalQueries[i].data?.stats ?? [],
  }));
  const isPending = historicalQueries.some((query) => query.isPending);

  return (
    <Section
      withBorder
      title="Historical"
      icon={<ChartLine size="1.3rem" />}
      titleRight={
        <Group ml={{ sm: "xl" }}>
          <Select
            value={granularity}
            onChange={(granularity) =>
              granularity && setGranularity(granularity as Types.Timelength)
            }
            data={[
              Types.Timelength.FiveSeconds,
              Types.Timelength.FifteenSeconds,
              Types.Timelength.ThirtySeconds,
              Types.Timelength.OneMinute,
              Types.Timelength.FiveMinutes,
              Types.Timelength.FifteenMinutes,
              Types.Timelength.ThirtyMinutes,
              Types.Timelength.OneHour,
              Types.Timelength.SixHours,
              Types.Timelength.OneDay,
            ]}
            w={120}
          />
        </Group>
      }
    >
      <Stack>
        {servers.length > MAX_CHART_SERVERS && (
          <Text c="dimmed" size="sm">
            Charting the first {MAX_CHART_SERVERS} of {servers.length} servers.
            Filter to specific Servers to compare the others.
          </Text>
        )}
        <SimpleGrid cols={{ base: 1, xl: 2 }} spacing="xl">
          {STAT_TYPES.map(([type, icon]) => (
            <MultiServerStatChart
              key={type}
              type={type}
              icon={icon}
              series={series}
              isPending={isPending}
            />
          ))}
        </SimpleGrid>
      </Stack>
    </Section>
  );
}

function MultiServerStatChart({
  type,
  icon,
  series,
  isPending,
}: {
  type: StatType;
  icon: ReactNode;
  series: { name: string; color: string; stats: Types.SystemStatsRecord[] }[];
  isPending: boolean;
}) {
  // Timestamps are bucket-aligned to the granularity server side,
  // so records merge across servers by exact ts.
  const chartData = useMemo(() => {
    const byTs = new Map<number, Record<string, number>>();
    for (const { name, stats } of series) {
      for (const record of stats) {
        const entry = byTs.get(record.ts) ?? { date: record.ts };
        entry[name] = getStat(record, type);
        byTs.set(record.ts, entry);
      }
    }
    return [...byTs.values()].sort((a, b) => a.date - b.date);
  }, [series, type]);

  const minTime = chartData[0]?.date ?? 0;
  const maxTime = chartData[chartData.length - 1]?.date ?? 0;
  const timeDiff = maxTime - minTime;

  const isNetwork = type === "Network Ingress" || type === "Network Egress";
  const isLoadAvg = type === "Load Average";
  const isPercent = !isNetwork && !isLoadAvg;

  const yTickFormatter = (value: number) => {
    if (isNetwork) return fmtSizeBytes(value);
    if (isLoadAvg) return value.toFixed(1);
    return `${value.toFixed(0)}%`;
  };

  const xTickFormatter = (ts: number) => {
    const date = new Date(ts);
    if (timeDiff < 2 * 60 * 60 * 1000) {
      return date.toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      });
    }
    return date.toLocaleString([], {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <Stack gap={4} className="bordered-light" p="md" bdrs="md">
      <Group gap="xs">
        {icon}
        <Text fz="xl" fw={500}>
          {isLoadAvg ? "Load Average (1m)" : type}
        </Text>
      </Group>
      {isPending ? (
        <Center h={200}>
          <Loader size="xl" />
        </Center>
      ) : (
        <ResponsiveContainer width="100%" height={200}>
          <LineChart
            data={chartData}
            margin={{ top: 5, right: 5, bottom: 0, left: 0 }}
          >
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="rgba(255,255,255,0.07)"
              vertical={false}
            />
            <XAxis
              dataKey="date"
              type="number"
              domain={["dataMin", "dataMax"]}
              tickFormatter={xTickFormatter}
              tick={{ fontSize: 10, fill: "var(--mantine-color-dimmed)" }}
              axisLine={false}
              tickLine={false}
              minTickGap={60}
            />
            <YAxis
              tickFormatter={yTickFormatter}
              tick={{ fontSize: 10, fill: "var(--mantine-color-dimmed)" }}
              axisLine={false}
              tickLine={false}
              width={isNetwork ? 70 : 42}
              domain={isPercent ? [0, 100] : ["auto", "auto"]}
            />
            <Legend
              iconType="line"
              wrapperStyle={{ fontSize: 11, paddingTop: 4 }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: "var(--mantine-color-dark-7)",
                border: "1px solid var(--mantine-color-dark-4)",
                borderRadius: "var(--mantine-radius-sm)",
                fontSize: 12,
              }}
              labelStyle={{ color: "var(--mantine-color-dimmed)" }}
              labelFormatter={(label) => new Date(label).toLocaleString()}
              // List the busiest server first.
              itemSorter={(item) => -Number(item.value)}
              formatter={(value, name) => {
                const v = Number(value);
                const formatted = isNetwork
                  ? fmtSizeBytes(v)
                  : isLoadAvg
                    ? v.toFixed(2)
                    : `${v.toFixed(1)}%`;
                return [formatted, name];
              }}
            />
            {series.map(({ name, color }) => (
              <Line
                key={name}
                type="monotone"
                dataKey={name}
                stroke={color}
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 3 }}
                connectNulls
                isAnimationActive={false}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      )}
    </Stack>
  );
}

function getStat(stat: Types.SystemStatsRecord, type: StatType) {
  switch (type) {
    case "Cpu":
      return stat.cpu_perc || 0;
    case "Memory":
      return stat.mem_total_gb > 0
        ? (100 * stat.mem_used_gb) / stat.mem_total_gb
        : 0;
    case "Disk":
      return stat.disk_total_gb > 0
        ? (100 * stat.disk_used_gb) / stat.disk_total_gb
        : 0;
    case "Network Ingress":
      return stat.network_ingress_bytes || 0;
    case "Network Egress":
      return stat.network_egress_bytes || 0;
    case "Load Average":
      return stat.load_average?.one ?? 0;
  }
}
