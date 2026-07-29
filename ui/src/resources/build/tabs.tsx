import { buildStateIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { MobileFriendlyTabsSelector, TabNoContent } from "mogh_ui";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { useMemo } from "react";
import { useBuild } from ".";
import BuildConfig from "./config";
import { Section } from "mogh_ui";
import BuildInfo from "./info";
import { ICONS } from "@/lib/icons";
import ResourceTable from "../table";

type BuildTabsView = "Config" | "Info" | "Resources";

export default function BuildTabs({ id }: { id: string }) {
  const [view, setView] = useLocalStorage<BuildTabsView>({
    key: `build-${id}-tab-v2`,
    defaultValue: "Config",
  });

  const info = useBuild(id)?.info;

  const deploymentsDisabled =
    (useRead("ListDeployments", {
      query: { specific: { build_ids: [id] } },
      limit: 1,
    }).data?.length || 0) === 0;

  const hasBeenBuilt = !!useRead("ListUpdates", {
    query: { "target.type": "Build", "target.id": id, operation: "RunBuild" },
  }).data?.updates[0];

  const showInfo =
    hasBeenBuilt || info?.files_on_host || !!info?.repo || !!info?.linked_repo;

  const tabsNoContent = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Config,
      },
      {
        value: "Info",
        icon: ICONS.Info,
        hidden: !showInfo,
      },
      {
        value: "Resources",
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
    case "Resources":
      View = (
        <Section titleOther={Selector}>
          <ResourceTable
            type="Deployment"
            specific={{ build_ids: [id] }}
            newProps={{ buildId: id }}
            mt="xs"
          />
        </Section>
      );
      break;
  }

  return (
    <Tabs color={buildStateIntention(info?.state)} value={view}>
      {View}
    </Tabs>
  );
}
