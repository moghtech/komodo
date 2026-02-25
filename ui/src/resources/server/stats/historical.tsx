import Section from "@/ui/section";
import { useStatsGranularity } from "../hooks";
import { useMemo, useState } from "react";
import { Types } from "komodo_client";
import { hexColorByIntention } from "@/lib/color";
import { convertTsMsToLocalUnixTsInMs } from "@/lib/utils";
import { useRead } from "@/lib/hooks";
import { ChartLine } from "lucide-react";
import ShowHideButton from "@/ui/show-hide-button";
import { Group, Select } from "@mantine/core";

type StatType =
  | "Cpu"
  | "Memory"
  | "Disk"
  | "Network Ingress"
  | "Network Egress"
  | "Load Average";

type StatDatapoint = { date: number; value: number };

export default function ServerHistoricalStats({ id }: { id: string }) {
  const [interval, setInterval] = useStatsGranularity();
  const [show, setShow] = useState(false);
  return (
    <Section
      title="Historical"
      icon={<ChartLine size="1.3rem" />}
      titleRight={
        <Group ml={{ sm: "xl" }}>
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
    >
      {/* CHARTS */}
    </Section>
  );
}

function StatChart({ serverId, type }: { serverId: string; type: StatType }) {
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
        date: convertTsMsToLocalUnixTsInMs(s.ts),
        value: s.load_average?.one ?? 0,
      }));
      const five = records.map((s) => ({
        date: convertTsMsToLocalUnixTsInMs(s.ts),
        value: s.load_average?.five ?? 0,
      }));
      const fifteen = records.map((s) => ({
        date: convertTsMsToLocalUnixTsInMs(s.ts),
        value: s.load_average?.fifteen ?? 0,
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

  const stats = seriesData.flatMap((s) => s.data);

  const min = stats?.[0]?.date ?? 0;
  const max = stats?.[stats.length - 1]?.date ?? 0;
  const diff = max - min;
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
  if (type === "Cpu") return hexColorByIntention("Good");
  if (type === "Memory") return hexColorByIntention("Warning");
  if (type === "Disk") return hexColorByIntention("Neutral");
  if (type === "Network Ingress") return hexColorByIntention("Good");
  if (type === "Network Egress") return hexColorByIntention("Critical");
  return hexColorByIntention("Unknown");
}
