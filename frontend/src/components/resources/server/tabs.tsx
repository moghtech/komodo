import { useLocalStorage, usePermissions, useRead, useUser } from "@lib/hooks";
import { useServer } from ".";
import { ReactNode, useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@ui/mobile-friendly-tabs";
import { ServerStats } from "./stats";
import { ServerInfo } from "./info";
import { ServerConfig } from "./config";
import { Section } from "@components/layouts";
import { DeploymentTable } from "../deployment/table";
import { ResourceComponents } from "..";
import { StackTable } from "../stack/table";
import { RepoTable } from "../repo/table";
import { ServerTerminals } from "@components/terminal/server";
import { Card, CardHeader, CardTitle } from "@ui/card";
import { Types } from "komodo_client";

type ServerTabsView = "Config" | "Stats" | "Docker" | "Resources" | "Terminals";

export const ServerTabs = ({ id }: { id: string }) => {
  const [view, setView] = useLocalStorage<ServerTabsView>(
    `server-${id}-tab`,
    "Config"
  );

  const { specificTerminal } = usePermissions({ type: "Server", id });
  const server_info = useServer(id)?.info;
  const terminalDisabled =
    !specificTerminal || (server_info?.terminals_disabled ?? true);

  const stacks =
    useRead("ListStacks", {}).data?.filter(
      (stack) => stack.info.server_id === id
    ) ?? [];
  const noStacks = stacks.length === 0;
  const deployments =
    useRead("ListDeployments", {}).data?.filter(
      (deployment) => deployment.info.server_id === id
    ) ?? [];
  const noDeployments = deployments.length === 0;
  const repos =
    useRead("ListRepos", {}).data?.filter(
      (repo) => repo.info.server_id === id
    ) ?? [];
  const noRepos = repos.length === 0;

  const noResources = noDeployments && noRepos && noStacks;

  const tabs = useMemo<TabNoContent<ServerTabsView>[]>(
    () => [
      {
        value: "Config",
      },
      {
        value: "Stats",
      },
      {
        value: "Docker",
      },
      {
        value: "Resources",
        disabled: noResources,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
      },
    ],
    [noResources, terminalDisabled]
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
      return <ServerConfig id={id} titleOther={Selector} />;
    case "Stats":
      return <ServerStats id={id} titleOther={Selector} />;
    case "Docker":
      return <ServerInfo id={id} titleOther={Selector} />;
    case "Resources":
      return (
        <ServerTabsResources
          id={id}
          stacks={stacks}
          deployments={deployments}
          repos={repos}
          Selector={Selector}
        />
      );
    case "Terminals":
      return <ServerTabsTerminals id={id} Selector={Selector} />;
  }
};

const ServerTabsResources = ({
  Selector,
  id,
  stacks,
  deployments,
  repos,
}: {
  Selector: ReactNode;
  id: string;
  stacks: Types.StackListItem[];
  deployments: Types.DeploymentListItem[];
  repos: Types.RepoListItem[];
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
            <ResourceComponents.Stack.New server_id={id} />
          )
        }
      >
        <StackTable stacks={stacks} />
      </Section>
      <Section
        title="Deployments"
        actions={
          (is_admin || !disable_non_admin_create) && (
            <ResourceComponents.Deployment.New server_id={id} />
          )
        }
      >
        <DeploymentTable deployments={deployments} />
      </Section>
      <Section
        title="Repos"
        actions={
          (is_admin || !disable_non_admin_create) && (
            <ResourceComponents.Repo.New server_id={id} />
          )
        }
      >
        <RepoTable repos={repos} />
      </Section>
    </Section>
  );
};

const ServerTabsTerminals = ({
  Selector,
  id,
}: {
  Selector: ReactNode;
  id: string;
}) => {
  const { specificTerminal } = usePermissions({ type: "Server", id });
  const server_info = useServer(id)?.info;
  const state = server_info?.state ?? Types.ServerState.NotOk;
  const terminals_disabled = server_info?.terminals_disabled ?? true;
  const container_terminals_disabled =
    server_info?.container_terminals_disabled ?? true;

  if (!specificTerminal) {
    return (
      <Section titleOther={Selector}>
        <Card>
          <CardHeader>
            <CardTitle>
              User does not have permission to use Terminals on this Server.
            </CardTitle>
          </CardHeader>
        </Card>
      </Section>
    );
  }

  if (state !== Types.ServerState.Ok) {
    return (
      <Section titleOther={Selector}>
        <Card>
          <CardHeader>
            <CardTitle>Server is not connected</CardTitle>
          </CardHeader>
        </Card>
      </Section>
    );
  }

  if (terminals_disabled && container_terminals_disabled) {
    return (
      <Section titleOther={Selector}>
        <Card>
          <CardHeader>
            <CardTitle>Terminals are disabled on this Server.</CardTitle>
          </CardHeader>
        </Card>
      </Section>
    );
  }

  return <ServerTerminals id={id} titleOther={Selector} />;
};
