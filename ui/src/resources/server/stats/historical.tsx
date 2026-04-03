import { Section } from "mogh_ui";
import { useStatsGranularity } from "../hooks";
import { ReactNode, useMemo } from "react";
import { Types } from "komodo_client";
import { hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ChartLine, Download, Upload } from "lucide-react";
import ShowHideButton from "@/ui/show-hide-button";
import {
  Center,
  Group,
  Loader,
  Select,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { fmtSizeBytes } from "@/lib/formatting";
import { ICONS } from "@/lib/icons";
import { useLocalStorage } from "@mantine/hooks";

type StatType =
  | "Cpu"
  | "Memory"
  | "Disk"
  | "Network Ingress"
  | "Network Egress"
  | "Load Average";

type StatDatapoint = { date: number; value: number };

const STAT_TYPES: [StatType, ReactNode][] = [
  ["Load Average", <ICONS.LoadAvg size="1.1rem" />],
  ["Cpu", <ICONS.Cpu size="1.1rem" />],
  ["Memory", <ICONS.Memory size="1.1rem" />],
  ["Disk", <ICONS.Disk size="1.1rem" />],
  ["Network Ingress", <Download size="1.1rem" />],
  ["Network Egress", <Upload size="1.1rem" />],
];

export default function ServerHistoricalStats({ id }: { id: string }) {
  const [interval, setInterval] = useStatsGranularity();
  const [show, setShow] = useLocalStorage({
    key: "server-stats-historical-show-v1",
    defaultValue: true,
  });
  return (
    <Section
      withBorder
      title="Historical"
      icon={<ChartLine size="1.3rem" />}
      titleRight={
        <Group ml={{ sm: "xl" }} onClick={(e) => e.stopPropagation()}>
          <Select
            value={interval}
            onChange={(interval) =>
              interval && setInterval(interval as Types.Timelength)
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
          <ShowHideButton show={show} setShow={setShow} />
        </Group>
      }
      onHeaderClick={() => setShow((s) => !s)}
    >
      {show && (
        <SimpleGrid cols={{ base: 1, xl: 2 }} spacing="xl">
          {STAT_TYPES.map(([type, icon]) => (
            <StatChart key={type} serverId={id} type={type} icon={icon} />
          ))}
        </SimpleGrid>
      )}
    </Section>
  );
}

function StatChart({
  serverId,
  type,
  icon,
}: {
  serverId: string;
  type: StatType;
  icon: ReactNode;
}) {
  const [granularity] = useStatsGranularity();

  const { data, isPending } = useRead(
    "GetHistoricalServerStats",
    {
      server: serverId,
      granularity,
    },
    {
      refetchInterval:
        granularity === Types.Timelength.FiveSeconds
          ? 5_000
          : granularity === Types.Timelength.FifteenSeconds
            ? 10_000
            : 15_000,
    },
  );

  const seriesData = useMemo(() => {
    if (!data?.stats) return [] as { label: string; data: StatDatapoint[] }[];
    const records = [...data.stats].reverse();
    if (type === "Load Average") {
      const one = records.map((s) => ({
        date: s.ts,
        value: s.load_average?.one ?? 0,
      }));
      const five = records.map((s) => ({
        date: s.ts,
        value: s.load_average?.five ?? 0,
      }));
      const fifteen = records.map((s) => ({
        date: s.ts,
        value: s.load_average?.fifteen ?? 0,
      }));
      return [
        { label: "1m", data: one },
        { label: "5m", data: five },
        { label: "15m", data: fifteen },
      ];
    }
    const single = records.map((stat) => ({
      date: stat.ts,
      value: getStat(stat, type),
    }));
    return [{ label: type, data: single }];
  }, [data, type]);

  const stats = seriesData.flatMap((s) => s.data);

  const minTime = stats?.[0]?.date ?? 0;
  const maxTime = stats?.[stats.length - 1]?.date ?? 0;
  const timeDiff = maxTime - minTime;

  const colors = useMemo((): Record<string, string> => {
    if (type === "Load Average") {
      return {
        "1m": hexColorByIntention("Good")!,
        "5m": hexColorByIntention("Neutral")!,
        "15m": hexColorByIntention("Warning")!,
      };
    }
    return { [type]: getColor(type) };
  }, [type]);

  const chartData = useMemo(() => {
    if (!seriesData.length) return [];
    return seriesData[0].data.map((point, i) => {
      const record: Record<string, number> = { date: point.date };
      seriesData.forEach((series) => {
        record[series.label] = series.data[i]?.value ?? 0;
      });
      return record;
    });
  }, [seriesData]);

  const isNetwork = type === "Network Ingress" || type === "Network Egress";
  const isLoadAvg = type === "Load Average";

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

  const gradientId = (label: string) =>
    `grad-${serverId}-${type.replace(/\s+/g, "-")}-${label.replace(/\s+/g, "-")}`;

  return (
    <Stack gap={4} className="bordered-light" p="md" bdrs="md">
      <Group gap="xs">
        {icon}
        <Text fz="xl" fw={500}>
          {type}
        </Text>
      </Group>
      {isPending ? (
        <Center h={200}>
          <Loader size="xl" />
        </Center>
      ) : (
        <ResponsiveContainer width="100%" height={200}>
          <AreaChart
            data={chartData}
            margin={{ top: 5, right: 5, bottom: 0, left: 0 }}
          >
            <defs>
              {seriesData.map((series) => (
                <linearGradient
                  key={series.label}
                  id={gradientId(series.label)}
                  x1="0"
                  y1="0"
                  x2="0"
                  y2="1"
                >
                  <stop
                    offset="5%"
                    stopColor={colors[series.label]}
                    stopOpacity={0.25}
                  />
                  <stop
                    offset="95%"
                    stopColor={colors[series.label]}
                    stopOpacity={0}
                  />
                </linearGradient>
              ))}
            </defs>
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="rgba(255,255,255,0.07)"
              vertical={false}
            />
            <XAxis
              dataKey="date"
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
              domain={isNetwork || isLoadAvg ? ["auto", "auto"] : [0, 100]}
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
            {isLoadAvg && (
              <Legend
                iconType="line"
                wrapperStyle={{ fontSize: 11, paddingTop: 4 }}
              />
            )}
            {seriesData.map((series) => (
              <Area
                key={series.label}
                type="monotone"
                dataKey={series.label}
                stroke={colors[series.label]}
                fill={`url(#${gradientId(series.label)})`}
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 3 }}
                isAnimationActive={false}
              />
            ))}
          </AreaChart>
        </ResponsiveContainer>
      )}
    </Stack>
  );
}

function getStat(stat: Types.SystemStatsRecord, type: StatType) {
  if (type === "Cpu") return stat.cpu_perc || 0;
  if (type === "Memory") return (100 * stat.mem_used_gb) / stat.mem_total_gb;
  if (type === "Disk") return (100 * stat.disk_used_gb) / stat.disk_total_gb;
  if (type === "Network Ingress") return stat.network_ingress_bytes || 0;
  if (type === "Network Egress") return stat.network_egress_bytes || 0;
  return 0;
}

function getColor(type: StatType) {
  if (type === "Cpu") return hexColorByIntention("Good")!;
  if (type === "Memory") return hexColorByIntention("Warning")!;
  if (type === "Disk") return hexColorByIntention("Neutral")!;
  if (type === "Network Ingress") return hexColorByIntention("Good")!;
  if (type === "Network Egress") return hexColorByIntention("Critical")!;
  return hexColorByIntention("Unknown")!;
}
