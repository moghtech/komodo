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

type StatType = "cpu" | "mem" | "disk" | "network_ingress" | "network_egress";

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

  const stats = useMemo(
    () =>
      data?.stats
        .map((stat) => {
          return {
            date: convertTsMsToLocalUnixTsInMs(stat.ts),
            value: getStat(stat, type),
          };
        })
        .reverse(),
    [data]
  );

  return (
    <div className={className}>
      {isPending ? (
        <div className="w-full max-w-full h-full flex items-center justify-center">
          <Loader2 className="w-8 h-8 animate-spin" />
        </div>
      ) : (
        stats &&
        stats.length > 0 && <InnerStatChart type={type} stats={stats} />
      )}
    </div>
  );
};

const BYTES_PER_GB = 1073741824.0;
const BYTES_PER_MB = 1048576.0;
const BYTES_PER_KB = 1024.0;

export const InnerStatChart = ({
  type,
  stats,
}: {
  type: StatType;
  stats: StatDatapoint[] | undefined;
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
  const maxStatValue = Math.max(...(stats?.map((d) => d.value) ?? [0]));

  const { unit, maxUnitValue } = useMemo(() => {
    if (type === "network_ingress" || type === "network_egress") {
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
    return { unit: "", maxUnitValue: 100 }; // Default for CPU, memory, disk
  }, [type, maxStatValue]);

  const valueAxis = useMemo(
    (): AxisOptions<StatDatapoint>[] => [
      {
        getValue: (datum) => datum.value,
        elementType: "area",
        min: 0,
        max: maxUnitValue,
        formatters: {
          tooltip: (value?: number) => (
            <div className="text-lg font-mono">
              {(type === "network_ingress" || type === "network_egress") && unit
                ? `${(value ?? 0) / (maxUnitValue / 1024)} ${unit}`
                : `${value?.toFixed(2)}%`}
            </div>
          ),
        },
      },
    ],
    [type, maxUnitValue, unit]
  );

  // const valueAxis = useMemo(
  //   (): AxisOptions<StatDatapoint>[] => [
  //     {
  //       getValue: (datum) => datum.value,
  //       elementType: "area",
  //       min: 0,
  //       max: 100,
  //       formatters: {
  //         tooltip: (value?: number) => (
  //           <div className="text-lg font-mono">
  //             {(value ?? 0) >= 10 ? value?.toFixed(2) : "0" + value?.toFixed(2)}
  //             %
  //           </div>
  //         ),
  //       },
  //     },
  //   ],
  //   []
  // );
  return (
    <Chart
      options={{
        data: [
          {
            label: type,
            data: stats ?? [],
          },
        ],
        primaryAxis: timeAxis,
        secondaryAxes: valueAxis,
        defaultColors: [getColor(type)],
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
  if (type === "cpu") return stat.cpu_perc || 0;
  if (type === "mem") return (100 * stat.mem_used_gb) / stat.mem_total_gb;
  if (type === "disk") return (100 * stat.disk_used_gb) / stat.disk_total_gb;
  if (type === "network_ingress") return stat.network_ingress_bytes || 0;
  if (type === "network_egress") return stat.network_egress_bytes || 0;
  return 0;
};

const getColor = (type: StatType) => {
  if (type === "cpu") return hex_color_by_intention("Good");
  if (type === "mem") return hex_color_by_intention("Warning");
  if (type === "disk") return hex_color_by_intention("Neutral");
  if (type === "network_ingress") return hex_color_by_intention("Good");
  if (type === "network_egress") return hex_color_by_intention("Critical");
  return hex_color_by_intention("Unknown");
};
