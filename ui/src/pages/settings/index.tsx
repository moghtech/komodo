import { lazy } from "react";
import { useSettingsView, useUser } from "@/lib/hooks";
import { Stack } from "@mantine/core";
import SettingsCoreInfo from "./core-info";
import MobileFriendlyTabs from "@/ui/mobile-friendly-tabs";
import ExportToml from "@/components/export-toml";
import SettingsVariables from "./variables";
import SettingsTags from "./tags";
import SettingsProviders from "./providers";
import { ICONS } from "@/theme/icons";
import SettingsOnboardingKeys from "./onboarding";
import SettingsUsers from "./users";

const Resources = lazy(() => import("@/pages/resources"));

export default function Settings() {
  const user = useUser().data;
  const [view, setView] = useSettingsView();
  const currentView =
    (view === "Users" || view === "Providers") && !user?.admin
      ? "Variables"
      : view;
  return (
    <Stack gap="xl" mb="50vh">
      <SettingsCoreInfo />
      <MobileFriendlyTabs
        value={view}
        onValueChange={setView as any}
        actions={currentView === "Variables" && <ExportToml includeVariables />}
        tabsProps={{ color: "green" }}
        tabs={[
          {
            value: "Variables",
            content: <SettingsVariables />,
            icon: ICONS.Variable,
          },
          {
            value: "Tags",
            content: <SettingsTags />,
            icon: ICONS.Tag,
          },
          {
            value: "Builders",
            content: <Resources _type="Builder" />,
            icon: ICONS.Builder,
          },
          {
            value: "Alerters",
            content: <Resources _type="Alerter" />,
            icon: ICONS.Alerter,
          },
          {
            value: "Providers",
            content: <SettingsProviders />,
            icon: ICONS.Provider,
          },
          {
            value: "Users",
            content: <SettingsUsers />,
            icon: ICONS.User,
            hidden: !user?.admin,
          },
          {
            value: "Onboarding",
            content: <SettingsOnboardingKeys />,
            icon: ICONS.OnboardingKey,
          },
        ]}
      />
    </Stack>
  );
}
