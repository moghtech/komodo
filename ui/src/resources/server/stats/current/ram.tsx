import { ICONS } from "@/lib/icons";
import { StatBar } from "mogh_ui";
import { Types } from "komodo_client";
import { useFullServer } from "@/resources/server";

export function ServerRamUsage({
  id,
  stats,
}: {
  id: string;
  stats: Types.SystemStats | undefined;
}) {
  const server = useFullServer(id);
  const usedRam = stats?.mem_used_gb;
  const totalRam = stats?.mem_total_gb;
  return (
    <StatBar
      title="RAM Usage"
      icon={<ICONS.Memory size="1.3rem" />}
      description={
        usedRam &&
        totalRam && (
          <>
            <b>{usedRam?.toFixed(1)} GB</b> of <b>{totalRam?.toFixed(1)} GB</b>{" "}
            in use
          </>
        )
      }
      percentage={((usedRam ?? 0) / (totalRam ?? 0)) * 100}
      warning={server?.config?.mem_warning}
      critical={server?.config?.mem_critical}
      flex="1"
    />
  );
}
