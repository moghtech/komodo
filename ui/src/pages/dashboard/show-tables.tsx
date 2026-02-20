import { useDashboardPreferences } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Button } from "@mantine/core";

export default function ShowTables() {
  const { preferences, togglePreference } = useDashboardPreferences();
  return (
    <Button
      variant="outline"
      onClick={() => togglePreference("showTables")}
      leftSection={
        preferences.showTables ? (
          <ICONS.Dashboard size="1rem" />
        ) : (
          <ICONS.Table size="1rem" />
        )
      }
    >
      {preferences.showTables ? "Show Dashboard" : "Show Tables"}
    </Button>
  );
}
