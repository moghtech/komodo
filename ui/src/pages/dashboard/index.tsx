import Page from "@/ui/page";
import { useDashboardPreferences, useSetTitle } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import DashboardRecents from "./recents";
import ExportToml from "@/components/export-toml";
import { Group } from "@mantine/core";
import ServerShowStats from "@/resources/server/show-stats";
import ShowTables from "./show-tables";
import DashboardTables from "./tables";

export default function Dashboard() {
  const { preferences } = useDashboardPreferences();
  useSetTitle(undefined);
  return (
    <>
      <Page
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
    </>
  );
}
