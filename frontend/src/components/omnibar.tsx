import { useAllResources, useLocalStorage, useRead, useSettingsView, useUser } from "@lib/hooks";
import { Button } from "@ui/button";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandList,
  CommandSeparator,
  CommandItem,
} from "@ui/command";
import { Box, Home, Search, User } from "lucide-react";
import { Fragment, ReactNode, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { cn, RESOURCE_TARGETS, usableResourcePath } from "@lib/utils";
import { Badge } from "@ui/badge";
import { ResourceComponents } from "./resources";
import { Switch } from "@ui/switch";
import { DOCKER_LINK_ICONS, TemplateMarker } from "./util";
import { UsableResource } from "@types";

export const OmniSearch = ({
  className,
  setOpen,
}: {
  className?: string;
  setOpen: (open: boolean) => void;
}) => {
  return (
    <Button
      variant="outline"
      onClick={() => setOpen(true)}
      className={cn(
        "flex items-center gap-4 w-fit md:w-[200px] lg:w-[300px] xl:w-[400px] justify-between hover:bg-card/50",
        className
      )}
    >
      <div className="flex items-center gap-4">
        <Search className="w-4 h-4" />{" "}
        <span className="text-muted-foreground hidden md:flex">Search</span>
      </div>
      <Badge
        variant="outline"
        className="text-muted-foreground hidden md:inline-flex"
      >
        shift + s
      </Badge>
    </Button>
  );
};

type OmniItem = {
  key: string;
  type: UsableResource;
  label: string;
  icon: ReactNode;
  template: boolean;
  onSelect: () => void;
};

export const OmniDialog = ({
  open,
  setOpen,
}: {
  open: boolean;
  setOpen: (open: boolean) => void;
}) => {
  const [search, setSearch] = useState("");
  const navigate = useNavigate();
  const nav = (value: string) => {
    setOpen(false);
    navigate(value);
  };
  const items = useOmniItems(nav, search);
  const [showContainers, setShowContainers] = useLocalStorage(
    "omni-show-containers",
    false
  );
  return (
    <CommandDialog open={open} onOpenChange={setOpen} manualFilter>
      <CommandInput
        placeholder="Search for resources..."
        value={search}
        onValueChange={setSearch}
      />
      <div className="flex gap-2 text-xs items-center justify-end px-2 py-1">
        <div className="text-muted-foreground">Show containers</div>
        <Switch checked={showContainers} onCheckedChange={setShowContainers} />
      </div>
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>

        {Object.entries(items)
          .filter(([_, items]) => items.length > 0)
          .map(([key, items], i) => (
            <Fragment key={key}>
              {i !== 0 && <CommandSeparator />}
              <CommandGroup heading={key ? key : undefined}>
                {items.map(({ key, type, label, icon, onSelect, template }) => (
                  <CommandItem
                    key={key}
                    value={key}
                    className="flex items-center gap-2 cursor-pointer"
                    onSelect={onSelect}
                  >
                    {icon}
                    {label}
                    {template && <TemplateMarker type={type} />}
                  </CommandItem>
                ))}
              </CommandGroup>
            </Fragment>
          ))}

        {showContainers && (
          <OmniContainers search={search} closeSearch={() => setOpen(false)} />
        )}
      </CommandList>
    </CommandDialog>
  );
};

const useOmniItems = (
  nav: (path: string) => void,
  search: string
): Record<string, OmniItem[]> => {
  const user = useUser().data;
  const resources = useAllResources();
  const [_, setSettingsView] = useSettingsView();
  return useMemo(() => {
    const searchTerms = search
      .toLowerCase()
      .split(" ")
      .filter((term) => term);
    return {
      "": [
        {
          key: "Home",
          type: "Server" as UsableResource,
          label: "Home",
          icon: <Home className="w-4 h-4" />,
          onSelect: () => nav("/"),
          template: false,
        },
        ...RESOURCE_TARGETS.map((_type) => {
          const type = _type === "ResourceSync" ? "Sync" : _type;
          const Components = ResourceComponents[_type];
          return {
            key: type + "s",
            type: _type,
            label: type + "s",
            icon: <Components.Icon />,
            onSelect: () => nav(usableResourcePath(_type)),
            template: false,
          };
        }),
        {
          key: "Containers",
          type: "Server" as UsableResource,
          label: "Containers",
          icon: <Box className="w-4 h-4" />,
          onSelect: () => nav("/containers"),
          template: false,
        },
        (user?.admin && {
          key: "Users",
          type: "Server" as UsableResource,
          label: "Users",
          icon: <User className="w-4 h-4" />,
          onSelect: () => {
            setSettingsView("Users");
            nav("/settings");
          },
          template: false,
        }) as OmniItem,
      ]
        .filter((item) => item)
        .filter((item) => {
          const label = item.label.toLowerCase();
          return (
            searchTerms.length === 0 ||
            searchTerms.every((term) => label.includes(term))
          );
        }),
      ...Object.fromEntries(
        RESOURCE_TARGETS.map((_type) => {
          const type = _type === "ResourceSync" ? "Sync" : _type;
          const lower_type = type.toLowerCase();
          const Components = ResourceComponents[_type];
          return [
            type + "s",
            resources[_type]
              ?.filter((resource) => {
                const lower_name = resource.name.toLowerCase();
                return (
                  searchTerms.length === 0 ||
                  searchTerms.every(
                    (term) =>
                      lower_name.includes(term) || lower_type.includes(term)
                  )
                );
              })
              .map((resource) => ({
                key: type + "-" + resource.name,
                type: _type,
                label: resource.name,
                icon: <Components.Icon id={resource.id} />,
                onSelect: () =>
                  nav(`/${usableResourcePath(_type)}/${resource.id}`),
                template: resource.template,
              })) || [],
          ];
        })
      ),
    };
  }, [user, resources, search]);
};

const OmniContainers = ({
  search,
  closeSearch,
}: {
  search: string;
  closeSearch: () => void;
}) => {
  const _containers = useRead("ListAllDockerContainers", {}).data;
  const containers = useMemo(() => {
    return _containers?.filter((c) => {
      const searchTerms = search
        .toLowerCase()
        .split(" ")
        .filter((term) => term);
      if (searchTerms.length === 0) return true;
      const lower = c.name.toLowerCase();
      return searchTerms.every(
        (term) => lower.includes(term) || "containers".includes(term)
      );
    });
  }, [_containers, search]);
  const navigate = useNavigate();
  if ((containers?.length ?? 0) < 1) return null;
  return (
    <>
      <CommandSeparator />
      <CommandGroup heading="Containers">
        {containers?.map((container) => (
          <CommandItem
            key={container.id}
            value={container.name}
            className="flex items-center gap-2 cursor-pointer"
            onSelect={() => {
              closeSearch();
              navigate(
                `/servers/${container.server_id!}/container/${container.name}`
              );
            }}
          >
            <DOCKER_LINK_ICONS.container
              server_id={container.server_id!}
              name={container.name}
            />
            {container.name}
          </CommandItem>
        ))}
      </CommandGroup>
    </>
  );
};
