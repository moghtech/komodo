import { ConfigItem } from "@components/config/util";
import { ResourceComponents } from "@components/resources";
import { ResourceLink } from "@components/resources/common";
import { useRead } from "@lib/hooks";
import { resource_name } from "@lib/utils";
import { Types } from "komodo_client";
import { UsableResource } from "@types";
import { Button } from "@ui/button";
import { DataTable, SortableHeader } from "@ui/data-table";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTrigger,
} from "@ui/dialog";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import { useState } from "react";

export const ResourcesConfig = ({
  resources,
  set,
  disabled,
  blacklist,
}: {
  resources: Types.ResourceTarget[];
  set: (resources: Types.ResourceTarget[]) => void;
  disabled: boolean;
  blacklist: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const servers = useRead("ListServers", {}).data ?? [];
  const stacks = useRead("ListStacks", {}).data ?? [];
  const deployments = useRead("ListDeployments", {}).data ?? [];
  const builds = useRead("ListBuilds", {}).data ?? [];
  const repos = useRead("ListRepos", {}).data ?? [];
  const syncs = useRead("ListResourceSyncs", {}).data ?? [];
  const all_resources = [
    ...servers.map((server) => {
      return {
        type: "Server",
        id: server.id,
        name: server.name.toLowerCase(),
        enabled: resources.find(
          (r) => r.type === "Server" && r.id === server.id
        )
          ? true
          : false,
      };
    }),
    ...stacks.map((stack) => {
      return {
        type: "Stack",
        id: stack.id,
        name: stack.name.toLowerCase(),
        enabled: resources.find((r) => r.type === "Stack" && r.id === stack.id)
          ? true
          : false,
      };
    }),
    ...deployments.map((deployment) => ({
      type: "Deployment",
      id: deployment.id,
      name: deployment.name.toLowerCase(),
      enabled: resources.find(
        (r) => r.type === "Deployment" && r.id === deployment.id
      )
        ? true
        : false,
    })),
    ...builds.map((build) => ({
      type: "Build",
      id: build.id,
      name: build.name.toLowerCase(),
      enabled: resources.find((r) => r.type === "Build" && r.id === build.id)
        ? true
        : false,
    })),
    ...repos.map((repo) => ({
      type: "Repo",
      id: repo.id,
      name: repo.name.toLowerCase(),
      enabled: resources.find((r) => r.type === "Repo" && r.id === repo.id)
        ? true
        : false,
    })),
    ...syncs.map((sync) => ({
      type: "ResourceSync",
      id: sync.id,
      name: sync.name.toLowerCase(),
      enabled: resources.find(
        (r) => r.type === "ResourceSync" && r.id === sync.id
      )
        ? true
        : false,
    })),
  ];
  // Filter out resources that no longer exist (e.g. deleted resources
  // that haven't been cleaned up from the alerter config yet)
  const all_resource_ids = new Set(all_resources.map((r) => `${r.type}:${r.id}`));
  const valid_resources = resources.filter(
    (r) => all_resource_ids.has(`${r.type}:${r.id}`)
  );
  const searchSplit = search.split(" ");
  const filtered_resources = searchSplit.length
    ? all_resources.filter((r) => {
        const name = r.name.toLowerCase();
        return searchSplit.every((term) => name.includes(term));
      })
    : all_resources;
  return (
    <ConfigItem label={`Resource ${blacklist ? "Blacklist" : "Whitelist"}`}>
      <div className="flex items-center gap-4">
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger>
            <Button variant="secondary">Edit Resources</Button>
          </DialogTrigger>
          <DialogContent className="min-w-[90vw] xl:min-w-[1200px]">
            <DialogHeader>Alerter Resources</DialogHeader>
            <div className="flex flex-col gap-4">
              <Input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search..."
                className="w-[200px] lg:w-[300px]"
              />
              <div className="max-h-[70vh] overflow-auto">
                <DataTable
                  tableKey="alerter-resources"
                  data={filtered_resources}
                  columns={[
                    {
                      accessorKey: "type",
                      header: ({ column }) => (
                        <SortableHeader column={column} title="Resource" />
                      ),
                      cell: ({ row }) => {
                        const Components =
                          ResourceComponents[
                            row.original.type as UsableResource
                          ];
                        return (
                          <div className="flex gap-2 items-center">
                            <Components.Icon />
                            {row.original.type}
                          </div>
                        );
                      },
                    },
                    {
                      accessorKey: "id",
                      sortingFn: (a, b) => {
                        const ra = resource_name(
                          a.original.type as UsableResource,
                          a.original.id
                        );
                        const rb = resource_name(
                          b.original.type as UsableResource,
                          b.original.id
                        );

                        if (!ra && !rb) return 0;
                        if (!ra) return -1;
                        if (!rb) return 1;

                        if (ra > rb) return 1;
                        else if (ra < rb) return -1;
                        else return 0;
                      },
                      header: ({ column }) => (
                        <SortableHeader column={column} title="Target" />
                      ),
                      cell: ({ row: { original: resource_target } }) => {
                        return (
                          <ResourceLink
                            type={resource_target.type as UsableResource}
                            id={resource_target.id}
                          />
                        );
                      },
                    },
                    {
                      accessorKey: "enabled",
                      header: ({ column }) => (
                        <SortableHeader
                          column={column}
                          title={blacklist ? "Blacklist" : "Whitelist"}
                        />
                      ),
                      cell: ({ row }) => {
                        return (
                          <Switch
                            disabled={disabled}
                            checked={row.original.enabled}
                            onCheckedChange={() => {
                              if (row.original.enabled) {
                                set(
                                  resources.filter(
                                    (r) =>
                                      r.type !== row.original.type ||
                                      r.id !== row.original.id
                                  )
                                );
                              } else {
                                set([
                                  ...resources,
                                  {
                                    type: row.original.type as UsableResource,
                                    id: row.original.id,
                                  },
                                ]);
                              }
                            }}
                          />
                        );
                      },
                    },
                  ]}
                />
              </div>
            </div>
            <DialogFooter>
              <Button onClick={() => setOpen(false)}>Confirm</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
        {valid_resources.length ? (
          <div className="text-muted-foreground">
            Alerts {blacklist ? "blacklisted" : "whitelisted"} by{" "}
            {valid_resources.length} resources
          </div>
        ) : undefined}
      </div>
    </ConfigItem>
  );
};
