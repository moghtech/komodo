import {
  SettingsView,
  useAllResources,
  useRead,
  useSettingsView,
  useUser,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import {
  terminalLink,
  termMatchesTypeKeyword,
  usableResourcePath,
} from "@/lib/utils";
import { RESOURCE_TARGETS, ResourceComponents } from "@/resources";
import {
  createSpotlight,
  SpotlightActionData,
  SpotlightActionGroupData,
} from "@mantine/spotlight";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { TemplateMarker } from "@/components/template-marker";
import { DOCKER_LINK_ICONS } from "@/components/docker/link";
import { Types } from "komodo_client";
import { hexColorByIntention, useDebounce } from "mogh_ui";
import { containerStateIntention, swarmStateIntention } from "@/lib/color";
import { keepPreviousData } from "@tanstack/react-query";

const ITEM_LIMIT = 7;

// Use a dedicated store to allow selection control below.
const [spotlightStore, spotlight] = createSpotlight();

export function useOmniSearch(opened: boolean) {
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
        .map((term) => term.trim())
        .filter((term) => term),
    [search],
  );

  const debouncedTerms = useDebounce(searchTerms, 500);

  const containersQuery: Types.ListAllContainers = useMemo(
    () => ({
      terms: debouncedTerms
        // Allows search like 'cont my name' to return containers matching 'my name'
        .filter((term) => !termMatchesTypeKeyword("containers", term)),
      limit: 10,
      page: 0,
    }),
    [debouncedTerms],
  );

  const containers = useRead("ListAllContainers", containersQuery, {
    refetchInterval: 15_000,
    // Only fetch when open and there is query typed
    enabled: opened && !!debouncedTerms.length,
    // Keep the previous results visible while fetching after a search
    // change to prevent resource flashing.
    placeholderData: keepPreviousData,
  }).data;

  const servicesQuery: Types.ListAllStackServices = useMemo(
    () => ({
      terms: debouncedTerms
        // Allows search like 'serv my name' to return services matching 'my name'
        .filter((term) => !termMatchesTypeKeyword("services", term)),
      limit: 10,
      page: 0,
    }),
    [debouncedTerms],
  );

  const services = useRead("ListAllStackServices", servicesQuery, {
    refetchInterval: 15_000,
    enabled: opened && !!debouncedTerms.length,
    placeholderData: keepPreviousData,
  }).data;

  const terminalsQuery: Types.ListTerminals = useMemo(
    () => ({
      terms: debouncedTerms
        // Allows search like 'term my name' to return terminals matching 'my name'
        .filter((term) => !termMatchesTypeKeyword("terminals", term)),
      limit: 10,
      page: 0,
    }),
    [debouncedTerms],
  );

  const terminals = useRead("ListTerminals", terminalsQuery, {
    refetchInterval: 15_000,
    enabled: opened,
    placeholderData: keepPreviousData,
  }).data;

  const user = useUser().data;
  const resources = useAllResources(debouncedTerms, 10, 15_000, opened);
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
            id: "Stats",
            label: "Stats",
            leftSection: <ICONS.Stats size="1.3rem" />,
            onClick: () => nav("/stats"),
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
        // Matches the server side term stripping in useAllResources
        const typeKeyword = type.toLowerCase() + "s";
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
                      lowerName.includes(term) ||
                      termMatchesTypeKeyword(typeKeyword, term),
                  )
                );
              })
              .map((resource) => {
                const info = resource.info as {
                  swarm_id?: string;
                  swarm_name?: string;
                  server_id?: string;
                  server_name?: string;
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
                    ? "Swarm: " + info.swarm_name
                    : info.server_id
                      ? "Server: " + info.server_name
                      : undefined,
                };
              }) ?? [],
        };
      }),

      {
        group: "Services",
        actions:
          services?.map((service) => {
            const intention = service?.swarm_service?.State
              ? swarmStateIntention(service?.swarm_service?.State)
              : containerStateIntention(service?.container?.state);
            const color = hexColorByIntention(intention);
            return {
              id: service.stack_id + " " + service.service,
              label: service.service,
              description: "Stack: " + service.stack_name,
              onClick: () =>
                nav(`/stacks/${service.stack_id}/service/${service.service}`),
              leftSection: <ICONS.Service size="1.3rem" color={color} />,
            };
          }) ?? [],
      },

      {
        group: "Containers",
        actions:
          containers?.map((container) => ({
            id: (container.server_id ?? "") + " " + container.name,
            label: container.name,
            description: "Server: " + container.server_name,
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
            description: terminalTargetDescription(
              terminal.target,
              terminal.target_name,
            ),
            onClick: () => nav(terminalLink(terminal)),
            leftSection: <ICONS.Terminal size="1.3rem" />,
          })) ?? [],
      },
    ];
  }, [
    resources,
    containers,
    services,
    terminals,
    searchTerms,
    user,
    nav,
    setSettingsView,
  ]);

  useEffect(() => {
    // Mantine only auto selects the first action when the query changes,
    // but the omni search actions settle later (debounce + fetch),
    // which can leave nothing selected. Select the first action
    // whenever newly settled actions render.
    const { listId } = spotlightStore.getState();
    const list = listId ? document.getElementById(listId) : null;
    if (!list) return;
    list.querySelector("[data-selected]")?.removeAttribute("data-selected");
    const first = list.querySelector("[data-action]");
    if (first) {
      first.setAttribute("data-selected", "true");
      first.scrollIntoView({ block: "nearest" });
    }
    spotlightStore.updateState((state) => ({
      ...state,
      selected: first ? 0 : -1,
    }));
  }, [_actions]);

  // LIMIT the action count for performance.
  let count = 0;
  const actions: SpotlightActionGroupData[] = [];
  for (const group of _actions) {
    const groupActions = [];
    for (const action of group.actions) {
      if (count >= ITEM_LIMIT) {
        break;
      }
      groupActions.push(action);
      count += 1;
    }
    if (groupActions.length) {
      actions.push({ group: group.group, actions: groupActions });
    }
    if (count >= ITEM_LIMIT) {
      break;
    }
  }

  return {
    search,
    setSearch,
    actions,
    spotlight,
    spotlightStore,
  };
}

function terminalTargetDescription(
  target: Types.TerminalTarget,
  target_name: string | undefined,
) {
  switch (target.type) {
    case "Server":
      return "Server: " + target_name;
    case "Container":
      return (
        "Server: " + target_name + ", Container: " + target.params.container
      );
    case "Stack":
      return "Stack: " + target_name + ", Service: " + target.params.service;
    case "Deployment":
      return "Deployment: " + target_name;
  }
}
