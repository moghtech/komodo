import { useMemo } from "react";
import { useFullResourceSync, useResourceSync } from ".";
import { useResourceSyncTabsView } from "./hooks";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import ResourceSyncConfig from "./config";
import { colorByIntention, resourceSyncStateIntention } from "@/lib/color";
import ResourceSyncInfo from "./info";
import ResourceSyncPending from "./pending";

export default function ResourceSyncTabs({ id }: { id: string }) {
  const info = useResourceSync(id)?.info;
  const sync = useFullResourceSync(id);
  const { view, setView, hideInfo, showPending } =
    useResourceSyncTabsView(sync);

  const tabsNoContent = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
      },
      {
        value: "Info",
        hidden: hideInfo,
      },
      {
        value: "Execute",
        disabled: !showPending,
      },
      {
        value: "Commit",
        hidden: !sync?.config?.managed,
        disabled: !showPending,
      },
    ],
    [hideInfo, sync?.config?.managed, showPending],
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
      View = <ResourceSyncConfig id={id} titleOther={Selector} />;
      break;
    case "Info":
      View = <ResourceSyncInfo id={id} titleOther={Selector} />;
      break;
    case "Execute":
      View = <ResourceSyncPending id={id} titleOther={Selector} />;
      break;
    case "Commit":
      View = <ResourceSyncPending id={id} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs
      color={colorByIntention(resourceSyncStateIntention(info?.state))}
      value={view}
    >
      {View}
    </Tabs>
  );
}
