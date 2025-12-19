import { useLocalStorage, usePermissions, useRead, useUser } from "@lib/hooks";
import { ReactNode, useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@ui/mobile-friendly-tabs";
import { SwarmConfig } from "./config";
import { SwarmInfo } from "./info";
import { Section } from "@components/layouts";
import { ResourceComponents } from "..";
import { StackTable } from "../stack/table";
import { DeploymentTable } from "../deployment/table";
import { Types } from "komodo_client";
import { SwarmInspect } from "./info/inspect";

type SwarmTabsView = "Config" | "Info" | "Resources" | "Inspect";

export const SwarmTabs = ({ id }: { id: string }) => {
  const [view, setView] = useLocalStorage<SwarmTabsView>(
    `swarm-${id}-tab-v1`,
    "Config"
  );

  const { specificInspect } = usePermissions({ type: "Swarm", id });

  const stacks =
    useRead("ListStacks", {}).data?.filter(
      (stack) => stack.info.swarm_id === id
    ) ?? [];
  const noStacks = stacks.length === 0;
  const deployments =
    useRead("ListDeployments", {}).data?.filter(
      (deployment) => deployment.info.swarm_id === id
    ) ?? [];
  const noDeployments = deployments.length === 0;
  const noResources = noDeployments && noStacks;

  const tabs = useMemo<TabNoContent<SwarmTabsView>[]>(
    () => [
      {
        value: "Config",
      },
      {
        value: "Info",
      },
      {
        value: "Inspect",
        disabled: !specificInspect,
      },
      {
        value: "Resources",
        disabled: noResources,
      },
    ],
    [specificInspect, noResources]
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
      tabsTriggerClassname="w-[110px]"
    />
  );

  switch (view) {
    case "Config":
      return <SwarmConfig id={id} titleOther={Selector} />;
    case "Info":
      return <SwarmInfo id={id} titleOther={Selector} />;
    case "Inspect":
      return <SwarmInspect id={id} titleOther={Selector} />;
    case "Resources":
      return (
        <SwarmTabsResources
          id={id}
          stacks={stacks}
          deployments={deployments}
          Selector={Selector}
        />
      );
  }
};

const SwarmTabsResources = ({
  Selector,
  id,
  stacks,
  deployments,
}: {
  Selector: ReactNode;
  id: string;
  stacks: Types.StackListItem[];
  deployments: Types.DeploymentListItem[];
}) => {
  const is_admin = useUser().data?.admin ?? false;
  const disable_non_admin_create =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;

  return (
    <Section titleOther={Selector}>
      <Section
        title="Stacks"
        actions={
          (is_admin || !disable_non_admin_create) && (
            <ResourceComponents.Stack.New swarm_id={id} />
          )
        }
      >
        <StackTable stacks={stacks} />
      </Section>
      <Section
        title="Deployments"
        actions={
          (is_admin || !disable_non_admin_create) && (
            <ResourceComponents.Deployment.New swarm_id={id} />
          )
        }
      >
        <DeploymentTable deployments={deployments} />
      </Section>
    </Section>
  );
};
