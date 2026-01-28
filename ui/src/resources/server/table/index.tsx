import { useDashboardPreferences } from "@/lib/hooks";
import { Types } from "komodo_client";
import StandardServerTable from "./standard";

export default function ServerTable({
  resources,
}: {
  resources: Types.ServerListItem[];
}) {
  const { preferences } = useDashboardPreferences();
  if (preferences.showServerStats) {
    return;
  } else {
    return <StandardServerTable resources={resources} />;
  }
}
