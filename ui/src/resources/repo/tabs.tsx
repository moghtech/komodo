import { useLocalStorage } from "@mantine/hooks";
import { useRepo } from ".";
import { useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { ICONS } from "@/theme/icons";
import { colorByIntention, repoStateIntention } from "@/lib/color";
import { Tabs } from "@mantine/core";
import RepoConfig from "./config";
import { useRead } from "@/lib/hooks";
import RepoLinkedResourcesSection from "./resources";

type RepoTabsView = "Config" | "Resources";

export default function RepoTabs({ id }: { id: string }) {
  const [view, setView] = useLocalStorage<RepoTabsView>({
    key: "repo-tabs-v1",
    defaultValue: "Config",
  });
  const info = useRepo(id)?.info;
  const stacks =
    useRead("ListStacks", {}).data?.filter(
      (stack) => stack.info.linked_repo === id,
    ) ?? [];
  const noStacks = stacks.length === 0;
  const builds =
    useRead("ListBuilds", {}).data?.filter(
      (build) => build.info.linked_repo === id,
    ) ?? [];
  const noBuilds = builds.length === 0;
  const syncs =
    useRead("ListResourceSyncs", {}).data?.filter(
      (sync) => sync.info.linked_repo === id,
    ) ?? [];
  const noSyncs = syncs.length === 0;

  const noResources = noStacks && noBuilds && noSyncs;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Config,
      },
      {
        value: "Resources",
        icon: ICONS.Resources,
        disabled: noResources,
      },
    ],
    [noResources],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
    />
  );

  let View = Selector;
  switch (view) {
    case "Config":
      View = <RepoConfig id={id} titleOther={Selector} />;
      break;
    case "Resources":
      View = (
        <RepoLinkedResourcesSection
          stacks={stacks}
          builds={builds}
          syncs={syncs}
          titleOther={Selector}
        />
      );
      break;
  }

  return (
    <Tabs
      color={colorByIntention(repoStateIntention(info?.state))}
      value={view}
    >
      {View}
    </Tabs>
  );
}
