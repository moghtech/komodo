import Page from "@/ui/page";
import { useDashboardPreferences, useSetTitle } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Group } from "@mantine/core";
import DashboardRecents from "./recents";
import ExportToml from "@/components/export-toml";
import ServerShowStats from "@/resources/server/show-stats";
import ShowTables from "./show-tables";
import DashboardTables from "./tables";
import DashboardActiveResources from "./active";

export default function Dashboard() {
  const { preferences } = useDashboardPreferences();
  useSetTitle(undefined);
  return (
    <Page
      aboveTitle={<DashboardActiveResources />}
      title="Dashboard"
      icon={ICONS.Dashboard}
      oppositeTitle={
        <Group>
          <ShowTables />
          <ServerShowStats />
          <ExportToml />
        </Group>
      }
    >
      {preferences.showTables ? <DashboardTables /> : <DashboardRecents />}
    </Page>
  );
}
