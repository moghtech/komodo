import { buildStateIntention, colorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { useMemo } from "react";
import { useBuild } from ".";
import BuildConfig from "./config";
import Section from "@/ui/section";
import NewResource from "../new";
import DeploymentTable from "../deployment/table";
import BuildInfo from "./info";
import { ICONS } from "@/theme/icons";

type BuildTabsView = "Config" | "Info" | "Deployments";

export default function BuildTabs({ id }: { id: string }) {
  const [view, setView] = useLocalStorage<BuildTabsView>({
    key: `build-${id}-tab-v2`,
    defaultValue: "Config",
  });

  const info = useBuild(id)?.info;

  const deployments = useRead("ListDeployments", {}).data?.filter(
    (deployment) => deployment.info.build_id === id,
  );
  const deploymentsDisabled = (deployments?.length || 0) === 0;

  const tabsNoContent = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Config,
      },
      {
        value: "Info",
        icon: ICONS.Info,
      },
      {
        value: "Deployments",
        disabled: deploymentsDisabled,
        icon: ICONS.Deployment,
      },
    ],
    [deploymentsDisabled],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabsNoContent}
      value={view}
      onValueChange={setView as any}
      tabProps={{ w: 140 }}
    />
  );

  let View = Selector;
  switch (view) {
    case "Config":
      View = <BuildConfig id={id} titleOther={Selector} />;
      break;
    case "Info":
      View = <BuildInfo id={id} titleOther={Selector} />;
      break;
    case "Deployments":
      View = (
        <Section
          titleOther={Selector}
          actions={<NewResource type="Deployment" buildId={id} />}
        >
          <DeploymentTable resources={deployments ?? []} />
        </Section>
      );
      break;
  }

  return (
    <Tabs
      color={colorByIntention(buildStateIntention(info?.state))}
      value={view}
    >
      {View}
    </Tabs>
  );
}
