import { hex_color_by_intention } from "@lib/color";
import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { useMemo } from "react";
import { useStatsGranularity } from "./hooks";
import { Loader2 } from "lucide-react";
import { AxisOptions, Chart } from "react-charts";
import { convertTsMsToLocalUnixTsInMs } from "@lib/utils";
import { useTheme } from "@ui/theme";
import { fmt_utc_date } from "@lib/formatting";

type StatType = "Cpu" | "Memory" | "Disk" | "Network Ingress" | "Network Egress" | "Load Average";

type StatDatapoint = { date: number; value: number };

export const StatChart = ({
  server_id,
  type,
  className,
}: {
  server_id: string;
  type: StatType;
  className?: string;
}) => {
  const [granularity] = useStatsGranularity();

  const { data, isPending } = useRead("GetHistoricalServerStats", {
    server: server_id,
    granularity,
  });

  const seriesData = useMemo(() => {
    if (!data?.stats) return [] as { label: string; data: StatDatapoint[] }[];
    const records = [...data.stats].reverse();
    if (type === "Load Average") {
      const one = records.map((s) => ({
        date: convertTsMsToLocalUnixTsInMs(s.ts),
        value: (s.load_average?.one ?? 0),
      }));
      const five = records.map((s) => ({
        date: convertTsMsToLocalUnixTsInMs(s.ts),
        value: (s.load_average?.five ?? 0),
      }));
      const fifteen = records.map((s) => ({
        date: convertTsMsToLocalUnixTsInMs(s.ts),
        value: (s.load_average?.fifteen ?? 0),
      }));
      return [
        { label: "1m", data: one },
        { label: "5m", data: five },
        { label: "15m", data: fifteen },
      ];
    }
    const single = records.map((stat) => ({
      date: convertTsMsToLocalUnixTsInMs(stat.ts),
      value: getStat(stat, type),
    }));
    return [{ label: type, data: single }];
  }, [data, type]);

  return (
    <div className={className}>
      <h1 className="px-2 py-1">{type}</h1>
      {isPending ? (
        <div className="w-full max-w-full h-full flex items-center justify-center">
          <Loader2 className="w-8 h-8 animate-spin" />
        </div>
      ) : seriesData.length > 0 ? (
        <InnerStatChart type={type} stats={seriesData.flatMap((s) => s.data)} seriesData={seriesData} />
      ) : null}
    </div>
  );
};

const BYTES_PER_GB = 1073741824.0;
const BYTES_PER_MB = 1048576.0;
const BYTES_PER_KB = 1024.0;

export const InnerStatChart = ({
  type,
  stats,
  seriesData,
}: {
  type: StatType;
  stats: StatDatapoint[] | undefined;
  seriesData?: { label: string; data: StatDatapoint[] }[];
}) => {
  const { theme: _theme } = useTheme();
  const theme =
    _theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : _theme;

  const min = stats?.[0]?.date ?? 0;
  const max = stats?.[stats.length - 1]?.date ?? 0;
  const diff = max - min;

  const timeAxis = useMemo((): AxisOptions<StatDatapoint> => {
    return {
      getValue: (datum) => new Date(datum.date),
      hardMax: new Date(max + diff * 0.02),
      hardMin: new Date(min - diff * 0.02),
      tickCount: 6,
      formatters: {
        // scale: (value?: Date) => fmt_date(value ?? new Date()),
        tooltip: (value?: Date) => (
          <div className="text-lg font-mono">
            {fmt_utc_date(value ?? new Date())}
          </div>
        ),
        cursor: (_value?: Date) => false,
      },
    };
  }, []);

  // Determine the dynamic scaling for network-related types
  const allValues = (seriesData ?? [{ data: stats ?? [] }]).flatMap((s) => s.data.map((d) => d.value));
  const maxStatValue = Math.max(...(allValues.length ? allValues : [0]));

  const { unit, maxUnitValue } = useMemo(() => {
    if (type === "Network Ingress" || type === "Network Egress") {
      if (maxStatValue <= BYTES_PER_KB) {
        return { unit: "KB", maxUnitValue: BYTES_PER_KB };
      } else if (maxStatValue <= BYTES_PER_MB) {
        return { unit: "MB", maxUnitValue: BYTES_PER_MB };
      } else if (maxStatValue <= BYTES_PER_GB) {
        return { unit: "GB", maxUnitValue: BYTES_PER_GB };
      } else {
        return { unit: "TB", maxUnitValue: BYTES_PER_GB * 1024 }; // Larger scale for high values
      }
    }
    if (type === "Load Average") {
      // Leave unitless; set max slightly above observed
      return { unit: "", maxUnitValue: maxStatValue === 0 ? 1 : maxStatValue * 1.2 };
    }
    return { unit: "", maxUnitValue: 100 }; // Default for CPU, memory, disk
  }, [type, maxStatValue]);

  const valueAxis = useMemo(
    (): AxisOptions<StatDatapoint>[] => [
      {
        getValue: (datum) => datum.value,
        elementType: type === "Load Average" ? "line" : "area",
        stacked: type !== "Load Average",
        min: 0,
        max: maxUnitValue,
        formatters: {
          tooltip: (value?: number) => (
            <div className="text-lg font-mono">
              {(type === "Network Ingress" || type === "Network Egress") && unit
                ? `${(value ?? 0) / (maxUnitValue / 1024)} ${unit}`
                : type === "Load Average"
                  ? `${(value ?? 0).toFixed(2)}`
                  : `${value?.toFixed(2)}%`}
            </div>
          ),
        },
      },
    ],
    [type, maxUnitValue, unit]
  );
  return (
    <Chart
      options={{
        data: seriesData ?? [{ label: type, data: stats ?? [] }],
        primaryAxis: timeAxis,
        secondaryAxes: valueAxis,
        defaultColors:
          type === "Load Average"
            ? [
                hex_color_by_intention("Good"),
                hex_color_by_intention("Neutral"),
                hex_color_by_intention("Unknown"),
              ]
            : [getColor(type)],
        dark: theme === "dark",
        padding: {
          left: 10,
          right: 10,
        },
        // tooltip: {
        //   showDatumInTooltip: () => false,
        // },
      }}
    />
  );
};

const getStat = (stat: Types.SystemStatsRecord, type: StatType) => {
  if (type === "Cpu") return stat.cpu_perc || 0;
  if (type === "Memory") return (100 * stat.mem_used_gb) / stat.mem_total_gb;
  if (type === "Disk") return (100 * stat.disk_used_gb) / stat.disk_total_gb;
  if (type === "Network Ingress") return stat.network_ingress_bytes || 0;
  if (type === "Network Egress") return stat.network_egress_bytes || 0;
  return 0;
};

const getColor = (type: StatType) => {
  if (type === "Cpu") return hex_color_by_intention("Good");
  if (type === "Memory") return hex_color_by_intention("Warning");
  if (type === "Disk") return hex_color_by_intention("Neutral");
  if (type === "Network Ingress") return hex_color_by_intention("Good");
  if (type === "Network Egress") return hex_color_by_intention("Critical");
  return hex_color_by_intention("Unknown");
};
