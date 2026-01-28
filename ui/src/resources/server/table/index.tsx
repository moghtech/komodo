import { useDashboardPreferences } from "@/lib/hooks";
import { Types } from "komodo_client";
import StandardServerTable from "./standard";
import { BoxProps } from "@mantine/core";

export default function ServerTable({
  resources,
  ...boxProps
}: {
  resources: Types.ServerListItem[];
} & BoxProps) {
  const { preferences } = useDashboardPreferences();
  if (preferences.showServerStats) {
    return;
  } else {
    return <StandardServerTable resources={resources} {...boxProps} />;
  }
}
