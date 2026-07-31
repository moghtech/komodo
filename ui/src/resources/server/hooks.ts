import { atomWithStorage, useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { useAtom } from "jotai";
import { useServer } from ".";

const statsGranularityAtom = atomWithStorage<Types.Timelength>(
  "stats-granularity-v0",
  Types.Timelength.FiveMinutes,
);

export function useStatsGranularity() {
  return useAtom<Types.Timelength>(statsGranularityAtom);
}

export type ServerAddress = {
  raw: string;
  protocol: "http:" | "https:";
  hostname: string;
};

export function useServerAddress(id: string | undefined): ServerAddress | null {
  const server = useServer(id);

  if (!server) return null;

  const base = server.info.external_address || server.info.address;

  if (!base) return null;

  const parsed = (() => {
    try {
      return new URL(base);
    } catch {
      return new URL("http://" + base);
    }
  })();

  return {
    raw: base,
    protocol: parsed.protocol === "https:" ? "https:" : "http:",
    hostname: parsed.hostname,
  };
}

export function useIsServerAvailable(id: string) {
  return useServer(id)?.info.state === Types.ServerState.Ok;
}

export function useServerStats(id: string) {
  const isServerAvailable = useIsServerAvailable(id);
  return useRead(
    "GetSystemStats",
    { server: id },
    {
      enabled: isServerAvailable,
      refetchInterval: 10_000,
    },
  ).data;
}

export function serverThresholds(server: Types.ServerListItem | undefined) {
  const thresholds = server?.info.alerting_thresholds;
  return {
    cpuWarning: thresholds?.cpu_warning ?? 75,
    cpuCritical: thresholds?.cpu_critical ?? 90,
    memWarning: thresholds?.mem_warning ?? 75,
    memCritical: thresholds?.mem_critical ?? 90,
    diskWarning: thresholds?.disk_warning ?? 75,
    diskCritical: thresholds?.disk_critical ?? 90,
  };
}

export function useServerThresholds(id: string) {
  const server = useServer(id);
  return serverThresholds(server);
}
