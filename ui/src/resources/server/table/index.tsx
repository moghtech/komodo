import { BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import { useDashboardPreferences } from "@/lib/hooks";
import StandardServerTable from "./standard";
import StatsServerTable from "./stats";

export default function ServerTable({
  resources,
  ...boxProps
}: {
  resources: Types.ServerListItem[];
} & BoxProps) {
  const { preferences } = useDashboardPreferences();
  if (preferences.showServerStats) {
    return <StatsServerTable resources={resources} {...boxProps} />;
  } else {
    return <StandardServerTable resources={resources} {...boxProps} />;
  }
}
