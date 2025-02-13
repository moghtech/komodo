import { atomWithStorage } from "@lib/hooks";
import { Types } from "komodo_client";
import { useAtom } from "jotai";

const statsGranularityAtom = atomWithStorage<Types.Timelength>(
  "stats-granularity-v0",
  Types.Timelength.FiveMinutes
);

export const useStatsGranularity = () => useAtom(statsGranularityAtom);
