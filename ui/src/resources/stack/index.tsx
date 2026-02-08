import { stackStateIntention, hexColorByIntention } from "@/lib/color";
import { useInvalidate, usePermissions, useRead, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import { RequiredResourceComponents } from "@/resources";
import NewResource from "@/resources/new";
import StackTable from "./table";
import StackTabs from "./tabs";
import {
  DeployStack,
  DestroyStack,
  PauseUnpauseStack,
  PullStack,
  RestartStack,
  StartStopStack,
} from "./executions";
import { SwarmComponents } from "@/resources/swarm";
import { ServerComponents } from "@/resources/server";
import {
  ActionIcon,
  Button,
  Group,
  HoverCard,
  List,
  Text,
} from "@mantine/core";
import FileSource from "@/components/file-source";
import { ArrowUp } from "lucide-react";
import { notifications } from "@mantine/notifications";
import StackServices from "./services";
import ResourceLink from "@/resources/link";

export const StackComponents: RequiredResourceComponents<
  Types.StackConfig,
  Types.StackInfo,
  Types.StackListItemInfo
> = {
  useList: () => useRead("ListStacks", {}).data,
  useListItem: (id) => StackComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetStack", { stack: id }).data,

  useResourceLinks: (stack) => stack?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetStacksSummary", {}).data;
    const all = [
      summary?.running ?? 0,
      summary?.stopped ?? 0,
      summary?.unhealthy ?? 0,
      summary?.unknown ?? 0,
    ];
    const [running, stopped, unhealthy, unknown] = all;
    return [
      all.every((item) => item === 0) && {
        title: "Down",
        intention: "Neutral",
        value: summary?.down ?? 0,
      },
      { intention: "Good", value: running, title: "Running" },
      {
        intention: "Warning",
        value: stopped,
        title: "Stopped",
      },
      {
        intention: "Critical",
        value: unhealthy,
        title: "Unhealthy",
      },
      {
        intention: "Unknown",
        value: unknown,
        title: "Unknown",
      },
    ];
  },

  Description: () => <>Deploy docker compose files.</>,

  New: () => <NewResource type="Stack" />,

  GroupExecutions: () => <></>,

  Table: StackTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListStacks", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = state && hexColorByIntention(stackStateIntention(state));
    return <ICONS.Stack size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const stack = StackComponents.useListItem(id);
    return (
      <EntityHeader
        intent={stackStateIntention(stack?.info.state)}
        icon={ICONS.Stack}
        name={stack?.name}
        state={stack?.info.state}
        status={
          stack?.info.state === Types.StackState.Unhealthy
            ? stack?.info.status
            : undefined
        }
      />
    );
  },

  State: ({ id }) => {
    let state = StackComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={stackStateIntention(state)} />;
  },
  Info: {
    DeployTarget: ({ id }) => {
      const info = StackComponents.useListItem(id)?.info;
      const swarm = SwarmComponents.useListItem(info?.swarm_id);
      const server = ServerComponents.useListItem(info?.server_id);
      return swarm?.id ? (
        <ResourceLink type="Swarm" id={swarm?.id} />
      ) : server?.id ? (
        <ResourceLink type="Server" id={server?.id} />
      ) : (
        <Group gap="xs">
          <ICONS.Server size="1rem" />
          <Text>Unknown</Text>
        </Group>
      );
    },
    Source: ({ id }) => {
      const info = StackComponents.useListItem(id)?.info;
      return <FileSource info={info} />;
    },
    Services: ({ id }) => {
      const info = StackComponents.useListItem(id)?.info;
      return (
        <HoverCard position="bottom-start">
          <HoverCard.Target>
            <Text>
              <b>{info?.services.length}</b> Service
              {(info?.services.length ?? 0 > 1) ? "s" : ""}
            </Text>
          </HoverCard.Target>
          <HoverCard.Dropdown>
            <List>
              {info?.services.map((service) => (
                <List.Item key={service.service}>
                  <Group gap="xs">
                    <Text>{service.service}</Text>
                    <Text c="dimmed">-</Text>
                    <Text c="dimmed">{service.image}</Text>
                    {service.update_available && (
                      <ActionIcon variant="light" color="cyan" size="xs">
                        <ArrowUp size="0.9rem" />
                      </ActionIcon>
                    )}
                  </Group>
                </List.Item>
              ))}
            </List>
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    NoConfig: ({ id }) => {
      const config = StackComponents.useFull(id)?.config;
      if (
        !config ||
        config?.files_on_host ||
        config?.file_contents ||
        config?.linked_repo ||
        config?.repo
      ) {
        return null;
      }
      return (
        <HoverCard width={300} position="bottom-start">
          <HoverCard.Target>
            <Button color="red">Config Missing</Button>
          </HoverCard.Target>
          <HoverCard.Dropdown>
            <Text>
              No configuration provided for stack. Cannot get stack state.
              Either paste the compose file contents into the UI, or configure a
              git repo containing your files.
            </Text>
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    ProjectMissing: ({ id }) => {
      const info = StackComponents.useListItem(id)?.info;
      const state = info?.state ?? Types.StackState.Unknown;
      if (
        !info ||
        !info?.project_missing ||
        [
          Types.StackState.Deploying,
          Types.StackState.Down,
          Types.StackState.Unknown,
        ].includes(state)
      ) {
        return null;
      }
      return (
        <HoverCard width={300} position="bottom-start">
          <HoverCard.Target>
            <Button color="red">Project Missing</Button>
          </HoverCard.Target>
          <HoverCard.Dropdown>
            <Text>
              The compose project is not on the host. If the compose stack is
              running, the 'Project Name' needs to be set. This can be found
              with 'docker compose ls'.
            </Text>
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },
    RemoteErrors: ({ id }) => {
      const info = StackComponents.useFull(id)?.info;
      const errors = info?.remote_errors;
      if (!info || !errors || errors.length === 0) {
        return null;
      }
      return (
        <HoverCard width={300} position="bottom-start">
          <HoverCard.Target>
            <Button color="red">Remote Error</Button>
          </HoverCard.Target>
          <HoverCard.Dropdown>
            <Text>
              There are errors reading the remote file contents. See <b>Info</b>{" "}
              tab for details.
            </Text>
          </HoverCard.Dropdown>
        </HoverCard>
      );
    },

    Refresh: ({ id }) => {
      const inv = useInvalidate();
      const info = StackComponents.useListItem(id)?.info;
      const { canExecute } = usePermissions({ type: "Stack", id });
      const { mutate, isPending } = useWrite("RefreshStackCache", {
        onSuccess: () => {
          inv(["ListStacks"], ["GetStack", { stack: id }]);
          notifications.show({ message: "Refreshed source file contents" });
        },
      });
      if (
        !canExecute ||
        // Don't show for UI defined, doesn't do anything
        (!info?.files_on_host && !info?.linked_repo && !info?.repo)
      )
        return null;
      return (
        <ActionIcon
          variant="light"
          onClick={() => mutate({ stack: id })}
          loading={isPending}
        >
          <ICONS.Refresh size="1rem" />
        </ActionIcon>
      );
    },
  },

  Executions: {
    DeployStack,
    PullStack,
    RestartStack,
    PauseUnpauseStack,
    StartStopStack,
    DestroyStack,
  },

  Config: StackTabs,
  DangerZone: ({ id }) => <></>,

  Page: {
    StackServices
  },
};

export const DEFAULT_STACK_FILE_CONTENTS = `## Add your compose file here
services:
  hello_world:
    image: hello-world
    # networks:
    #   - default
    # ports:
    #   - 3000:3000
    # volumes:
    #   - data:/data

# networks:
#   default: {}

# volumes:
#   data:
`;
