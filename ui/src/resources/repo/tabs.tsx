import { useLocalStorage } from "@mantine/hooks";
import { useRepo } from ".";
import { useMemo } from "react";
import { MobileFriendlyTabsSelector, TabNoContent } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import { repoStateIntention } from "@/lib/color";
import { Tabs } from "@mantine/core";
import RepoConfig from "./config";
import { useRead } from "@/lib/hooks";
import RepoLinkedResourcesSection from "./resources";

type RepoTabsView = "Config" | "Resources";

export default function RepoTabs({ id }: { id: string }) {
  const [view, setView] = useLocalStorage<RepoTabsView>({
    key: `repo-${id}-tab-v2`,
    defaultValue: "Config",
  });
  const info = useRepo(id)?.info;
  const stacks =
    useRead("ListStacks", {
      query: { specific: { linked_repos: [id] } },
      limit: 1,
    }).data ?? [];
  const noStacks = stacks.length === 0;
  const builds =
    useRead("ListBuilds", {
      query: { specific: { linked_repos: [id] } },
      limit: 1,
    }).data ?? [];
  const noBuilds = builds.length === 0;
  const syncs =
    useRead("ListResourceSyncs", {
      query: { specific: { linked_repos: [id] } },
      limit: 1,
    }).data ?? [];
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
      View = <RepoLinkedResourcesSection repoId={id} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs color={repoStateIntention(info?.state)} value={view}>
      {View}
    </Tabs>
  );
}
