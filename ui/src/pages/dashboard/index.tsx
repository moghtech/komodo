import Page from "@/ui/page";
import { useDashboardPreferences, useSetTitle } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import RecentsDashboard from "./recents";
import ExportToml from "@/components/export-toml";

export default function Dashboard() {
  const { preferences } = useDashboardPreferences();
  useSetTitle(undefined);
  return (
    <>
      <Page
        title="Dashboard"
        icon={ICONS.Dashboard}
        oppositeTitle={<ExportToml />}
      >
        <RecentsDashboard />
        {/* {preferences.showTables ? <></> : <RecentsDashboard />} */}
      </Page>
    </>
  );
}
