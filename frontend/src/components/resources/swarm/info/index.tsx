import { Section } from "@components/layouts";
import { ReactNode, useMemo, useState } from "react";
import { useSwarm } from "..";
import { Types } from "komodo_client";
import { useLocalStorage } from "@lib/hooks";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@ui/mobile-friendly-tabs";
import { SwarmNodes } from "./nodes";
import { SwarmSecrets } from "./secrets";
import { SwarmServices } from "./services";
import { SwarmTasks } from "./tasks";
import { SwarmInspect } from "./inspect";
import { SwarmConfigs } from "./configs";
import { SwarmStacks } from "./stacks";

type SwarmInfoView =
  | "Nodes"
  | "Stacks"
  | "Services"
  | "Tasks"
  | "Configs"
  | "Secrets"
  | "Inspect";

export const SwarmInfo = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const _search = useState("");
  const state = useSwarm(id)?.info.state ?? Types.SwarmState.Unknown;
  const [view, setView] = useLocalStorage<SwarmInfoView>(
    "swarm-info-view-v1",
    "Inspect"
  );

  if (state === Types.SwarmState.Unknown) {
    return (
      <Section titleOther={titleOther}>
        <h2 className="text-muted-foreground">
          Swarm unreachable, info is not available
        </h2>
      </Section>
    );
  }

  const tabsNoContent = useMemo<TabNoContent<SwarmInfoView>[]>(
    () => [
      {
        value: "Nodes",
      },
      {
        value: "Stacks",
      },
      {
        value: "Services",
      },
      {
        value: "Tasks",
      },
      {
        value: "Configs",
      },
      {
        value: "Secrets",
      },
      {
        value: "Inspect",
      },
    ],
    []
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabsNoContent}
      value={view}
      onValueChange={setView as any}
      tabsTriggerClassname="w-[110px]"
    />
  );

  const Component = () => {
    switch (view) {
      case "Nodes":
        return <SwarmNodes id={id} titleOther={Selector} _search={_search} />;
      case "Stacks":
        return <SwarmStacks id={id} titleOther={Selector} _search={_search} />;
      case "Services":
        return (
          <SwarmServices id={id} titleOther={Selector} _search={_search} />
        );
      case "Tasks":
        return <SwarmTasks id={id} titleOther={Selector} _search={_search} />;
      case "Configs":
        return <SwarmConfigs id={id} titleOther={Selector} _search={_search} />;
      case "Secrets":
        return <SwarmSecrets id={id} titleOther={Selector} _search={_search} />;
      case "Inspect":
        return <SwarmInspect id={id} titleOther={Selector} />;
    }
  };

  return (
    <Section titleOther={titleOther}>
      <Component />
    </Section>
  );
};
