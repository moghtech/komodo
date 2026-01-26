import { Page } from "@/ui/page";
import { useDashboardPreferences, useSetTitle } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import RecentsDashboard from "./recents";

export default function Dashboard() {
  const { preferences } = useDashboardPreferences();
  useSetTitle(undefined);
  return (
    <>
      <Page title="Dashboard" icon={ICONS.Dashboard}>
        <RecentsDashboard />
        {/* {preferences.showTables ? <></> : <RecentsDashboard />} */}
      </Page>
    </>
  );
}
