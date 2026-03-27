import {
  ResourceMap,
  SettingsView,
  useAllResources,
  useRead,
  useSettingsView,
  useUser,
} from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { terminalLink, usableResourcePath } from "@/lib/utils";
import { RESOURCE_TARGETS, ResourceComponents } from "@/resources";
import {
  spotlight,
  SpotlightActionData,
  SpotlightActionGroupData,
} from "@mantine/spotlight";
import { useCallback, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { TemplateMarker } from "@/components/template-marker";
import { DOCKER_LINK_ICONS } from "@/components/docker/link";
import { Types } from "komodo_client";

const ITEM_LIMIT = 7;
let count = 0;

export function useOmniSearch(): {
  search: string;
  setSearch: (value: string) => void;
  actions: SpotlightActionGroupData[];
} {
  const navigate = useNavigate();
  const nav = useCallback(
    (to: string) => {
      navigate(to);
      spotlight.close();
    },
    [navigate],
  );

  const [search, setSearch] = useState("");
  const searchTerms = useMemo(
    () =>
      search
        .toLowerCase()
        .split(" ")
        .filter((term) => term),
    [search],
  );

  const _containers = useRead(
    "ListAllDockerContainers",
    {},
    { refetchInterval: 15_000 },
  ).data;
  const containers = useMemo(() => {
    return _containers?.filter((c) => {
      if (searchTerms.length === 0) return true;
      const lower = c.name.toLowerCase();
      return searchTerms.every(
        (term) => lower.includes(term) || "containers".includes(term),
      );
    });
  }, [_containers, searchTerms]);

  const _terminals = useRead(
    "ListTerminals",
    {},
    { refetchInterval: 15_000 },
  ).data;
  const terminals = useMemo(() => {
    return _terminals?.filter((c) => {
      if (searchTerms.length === 0) return true;
      const lower = c.name.toLowerCase();
      return searchTerms.every(
        (term) => lower.includes(term) || "terminals".includes(term),
      );
    });
  }, [_terminals, searchTerms]);

  const user = useUser().data;
  const resources = useAllResources(15_000);
  const [_, setSettingsView] = useSettingsView();

  const _actions = useMemo(() => {
    return [
      {
        group: "",
        actions: [
          {
            id: "Dashboard",
            label: "Dashboard",
            leftSection: <ICONS.Dashboard size="1.3rem" />,
            onClick: () => nav("/"),
          },

          ...RESOURCE_TARGETS.map((_type) => {
            const type = _type === "ResourceSync" ? "Sync" : _type;
            const Components = ResourceComponents[_type];
            return {
              id: type + "s",
              label: type + "s",
              leftSection: <Components.Icon size="1.3rem" />,
              onClick: () => {
                if (type === "Builder" || type === "Alerter") {
                  setSettingsView((type + "s") as SettingsView);
                  nav("/settings");
                } else {
                  nav(usableResourcePath(_type));
                }
              },
            };
          }),

          {
            id: "Containers",
            label: "Containers",
            leftSection: <ICONS.Container size="1.3rem" />,
            onClick: () => nav("/containers"),
          },
          {
            id: "Terminals",
            label: "Terminals",
            leftSection: <ICONS.Terminal size="1.3rem" />,
            onClick: () => nav("/terminals"),
          },
          {
            id: "Schedules",
            label: "Schedules",
            leftSection: <ICONS.Schedule size="1.3rem" />,
            onClick: () => nav("/schedules"),
          },
          {
            id: "Variables",
            label: "Variables",
            leftSection: <ICONS.Variable size="1.3rem" />,
            onClick: () => {
              setSettingsView("Variables");
              nav("/settings");
            },
          },
          user?.admin && {
            id: "Users",
            label: "Users",
            leftSection: <ICONS.User size="1.3rem" />,
            onClick: () => {
              setSettingsView("Users");
              nav("/settings");
            },
          },
        ].filter((item) => {
          if (!item) return;
          const label = item.label.toLowerCase();
          return (
            searchTerms.length === 0 ||
            searchTerms.every((term) => label.includes(term))
          );
        }) as SpotlightActionData[],
      },

      ...RESOURCE_TARGETS.map((_type) => {
        const type = _type === "ResourceSync" ? "Sync" : _type;
        const lowerType = type.toLowerCase();
        const Components = ResourceComponents[_type];
        return {
          group: type + "s",
          actions:
            resources[_type]
              ?.filter((resource) => {
                const lowerName = resource.name.toLowerCase();
                return (
                  searchTerms.length === 0 ||
                  searchTerms.every(
                    (term) =>
                      lowerName.includes(term) || lowerType.includes(term),
                  )
                );
              })
              .map((resource) => {
                const info = resource.info as {
                  swarm_id: string;
                  server_id: string;
                };
                return {
                  id: type + " " + resource.name,
                  label: resource.name,
                  onClick: () =>
                    nav(`/${usableResourcePath(_type)}/${resource.id}`),
                  leftSection: (
                    <Components.Icon id={resource.id} size="1.3rem" />
                  ),
                  rightSection: resource.template && (
                    <TemplateMarker type={_type} />
                  ),
                  description: info.swarm_id
                    ? "Swarm: " +
                      resources.Swarm?.find(
                        (swarm) => info.swarm_id === swarm.id,
                      )?.name
                    : info.server_id
                      ? "Server: " +
                        resources.Server?.find(
                          (server) => info.server_id === server.id,
                        )?.name
                      : undefined,
                };
              }) ?? [],
        };
      }),

      {
        group: "Containers",
        actions:
          containers?.map((container) => ({
            id: container.server_id ?? "" + " " + container.name,
            label: container.name,
            description:
              "Server: " +
              resources.Server?.find(
                (server) => container.server_id === server.id,
              )?.name,
            onClick: () =>
              nav(
                `/servers/${container.server_id}/container/${container.name}`,
              ),
            leftSection: (
              <DOCKER_LINK_ICONS.Container
                serverId={container.server_id!}
                name={container.name}
                size="1.3rem"
              />
            ),
          })) ?? [],
      },

      {
        group: "Terminals",
        actions:
          terminals?.map((terminal) => ({
            id: JSON.stringify(terminal.target) + " " + terminal.name,
            label: terminal.name,
            description: terminalTargetDescription(terminal.target, resources),
            onClick: () => nav(terminalLink(terminal)),
            leftSection: <ICONS.Terminal size="1.3rem" />,
          })) ?? [],
      },
    ];
  }, [resources]);

  // LIMIT the action count for performance.
  // Reset count on render before creating actual actions.
  count = 0;
  const actions: SpotlightActionGroupData[] = [];
  for (const group of _actions) {
    const groupActions = [];
    for (const action of group.actions) {
      groupActions.push(action);
      count += 1;
      if (count > ITEM_LIMIT) {
        break;
      }
    }
    if (groupActions.length) {
      actions.push({ group: group.group, actions: groupActions });
    }
    if (count > ITEM_LIMIT) {
      break;
    }
  }

  return {
    search,
    setSearch,
    actions,
  };
}

function terminalTargetDescription(
  target: Types.TerminalTarget,
  resources: ResourceMap,
) {
  switch (target.type) {
    case "Server":
      return (
        "Server: " +
        resources.Server?.find((server) => target.params.server === server.id)
          ?.name
      );
    case "Container":
      return (
        "Server: " +
        resources.Server?.find((server) => target.params.server === server.id)
          ?.name +
        ", Container: " +
        target.params.container
      );
    case "Stack":
      return (
        "Stack: " +
        resources.Stack?.find((stack) => target.params.stack === stack.id)
          ?.name +
        ", Service: " +
        target.params.service
      );
    case "Deployment":
      return (
        "Deployment: " +
        resources.Deployment?.find(
          (deployment) => target.params.deployment === deployment.id,
        )?.name
      );
  }
}
