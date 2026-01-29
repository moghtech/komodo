import { useAllResources, useSettingsView, useUser } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { usableResourcePath } from "@/lib/utils";
import { RESOURCE_TARGETS, ResourceComponents } from "@/resources";
import { useLocalStorage } from "@mantine/hooks";
import {
  spotlight,
  SpotlightActionData,
  SpotlightActionGroupData,
} from "@mantine/spotlight";
import { useCallback, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { TemplateMarker } from "../template-marker";

const ITEM_LIMIT = 10;
let count = 0;

export function useOmniSearch(): {
  search: string;
  setSearch: (value: string) => void;
  actions: SpotlightActionGroupData[];
} {
  const [search, setSearch] = useState("");
  const navigate = useNavigate();
  const nav = useCallback(
    (to: string) => {
      navigate(to);
      spotlight.close();
    },
    [navigate],
  );
  const [showContainers, setShowContainers] = useLocalStorage({
    key: "omni-show-containers",
    defaultValue: false,
  });
  const user = useUser().data;
  const resources = useAllResources();
  const [_, setSettingsView] = useSettingsView();
  const _actions = useMemo(() => {
    const searchTerms = search
      .toLowerCase()
      .split(" ")
      .filter((term) => term);
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
              onClick: () => nav(usableResourcePath(_type)),
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
              .map((resource) => ({
                id: type + " " + resource.name,
                label: resource.name,
                onClick: () =>
                  nav(`/${usableResourcePath(_type)}/${resource.id}`),
                leftSection: <Components.Icon id={resource.id} />,
                rightSection: resource.template && (
                  <TemplateMarker type={_type} />
                ),
              })) ?? [],
        };
      }),
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
