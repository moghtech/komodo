import { useDashboardPreferences } from "@/lib/hooks";
import { Types } from "komodo_client";
import StandardServerTable from "./standard";
import { TableProps } from "@mantine/core";

export default function ServerTable({
  resources,
  ...tableProps
}: {
  resources: Types.ServerListItem[];
} & TableProps) {
  const { preferences } = useDashboardPreferences();
  if (preferences.showServerStats) {
    return;
  } else {
    return <StandardServerTable resources={resources} {...tableProps} />;
  }
}
