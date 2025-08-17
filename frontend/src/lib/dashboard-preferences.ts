import { atomWithStorage } from "@lib/hooks";
import { useAtom } from "jotai";

interface DashboardPreferences {
  showServerStats: boolean;
}

const DEFAULT_PREFERENCES: DashboardPreferences = {
  showServerStats: false,
};

export const dashboardPreferencesAtom = atomWithStorage<DashboardPreferences>(
  "komodo-dashboard-preferences",
  DEFAULT_PREFERENCES
);

export const useDashboardPreferences = () => {
  const [preferences, setPreferences] = useAtom(dashboardPreferencesAtom);

  const updatePreference = <K extends keyof DashboardPreferences>(
    key: K,
    value: DashboardPreferences[K]
  ) => {
    setPreferences({ ...preferences, [key]: value });
  };

  return {
    preferences,
    updatePreference,
  };
};
