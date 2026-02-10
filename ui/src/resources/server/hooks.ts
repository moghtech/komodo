import { atomWithStorage, useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { useAtom } from "jotai";

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

export function useServerAddress(
  serverId: string | undefined,
): ServerAddress | null {
  const server = useRead("ListServers", {}).data?.find(
    (s) => s.id === serverId,
  );

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
