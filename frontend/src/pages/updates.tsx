import { Page } from "@components/layouts";
import { ResourceComponents } from "@components/resources";
import { UpdatesTable } from "@components/updates/table";
import { useRead, useResourceParamType, useSetTitle } from "@lib/hooks";
import { filterBySplit, RESOURCE_TARGETS } from "@lib/utils";
import { Types } from "komodo_client";
import { CaretSortIcon } from "@radix-ui/react-icons";
import { UsableResource } from "@types";
import { Button } from "@ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Bell,
  Box,
  ChevronLeft,
  ChevronRight,
  MinusCircle,
  SearchX,
} from "lucide-react";
import { useState } from "react";
import { useParams, useSearchParams } from "react-router-dom";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { ResourceSelector } from "@components/resources/common";

export const UpdatesPage = () => {
  const [page, setPage] = useState(0);
  const [params, setParams] = useSearchParams();

  const { type, id, operation } = {
    type: (params.get("type") as UsableResource) ?? undefined,
    id: params.get("id") ?? undefined,
    operation: (params.get("operation") as Types.Operation) ?? undefined,
  };

  const { data: updates } = useRead("ListUpdates", {
    query: { "target.type": type, "target.id": id, operation },
    page,
  });

  return (
    <Page
      title="Updates"
      icon={<Bell className="w-8" />}
      actions={
        <>
          <div className="flex items-center md:justify-end gap-4 flex-wrap">
            {/* resource type */}
            <Select
              value={type ?? "all"}
              onValueChange={(type) => {
                const p = new URLSearchParams(params.toString());
                type === "all" ? p.delete("type") : p.set("type", type);
                p.delete("id");
                p.delete("operation");
                setParams(p);
              }}
            >
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">
                  <div className="flex items-center gap-2">
                    <Box className="w-4 text-muted-foreground" />
                    All Resources
                  </div>
                </SelectItem>
                <SelectSeparator />
                {RESOURCE_TARGETS.map((type) => {
                  const Icon = ResourceComponents[type].Icon;
                  return (
                    <SelectItem key={type} value={type}>
                      <div className="flex items-center gap-2">
                        <span className="text-muted-foreground">
                          <Icon />
                        </span>
                        {type}
                      </div>
                    </SelectItem>
                  );
                })}
              </SelectContent>
            </Select>

            {/* resource id */}
            {type && (
              <ResourceSelector
                type={type}
                selected={id}
                onSelect={(id) => {
                  const p = new URLSearchParams(params.toString());
                  id === "all" ? p.delete("id") : p.set("id", id);
                  setParams(p);
                }}
              />
            )}

            {/* operation */}
            <OperationSelector
              selected={operation}
              options={type && OPERATIONS_BY_RESOURCE[type]}
              onSelect={(op) => {
                const p = new URLSearchParams(params.toString());
                op ? p.set("operation", op) : p.delete("operation");
                setParams(p);
              }}
            />

            {/* reset */}
            <Button
              size="icon"
              onClick={() => setParams({})}
              variant="secondary"
            >
              <MinusCircle className="w-4" />
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-2">
        <UpdatesTable
          updates={updates?.updates ?? []}
          showTarget={!params.get("id")}
        />
        <div className="flex gap-4 items-center">
          <Button
            variant="outline"
            onClick={() => setPage(page - 1)}
            disabled={page === 0}
            size="icon"
          >
            <ChevronLeft className="w-4" />
          </Button>
          {Array.from(new Array(page + 1)).map((_, i) => (
            <Button
              key={i}
              onClick={() => setPage(i)}
              variant={page === i ? "secondary" : "outline"}
            >
              {i + 1}
            </Button>
          ))}
          {/* Page: {page + 1} */}
          <Button
            variant="outline"
            onClick={() => updates?.next_page && setPage(updates.next_page)}
            disabled={!updates?.next_page}
            size="icon"
          >
            <ChevronRight className="w-4" />
          </Button>
        </div>
      </div>
    </Page>
  );
};

export const Updates = () => {
  const type = useResourceParamType()!;
  const id = useParams().id as string;
  if (type && id) {
    return <ResourceUpdates type={type} id={id} />;
  } else {
    return <AllUpdates />;
  }
};

const AllUpdates = () => {
  useSetTitle("Updates");
  const [operation, setOperation] = useState<Types.Operation | undefined>();
  const [page, setPage] = useState(0);
  const updates = useRead("ListUpdates", { query: { operation }, page }).data;
  return (
    <Page
      title="Updates"
      icon={<Bell className="w-8 h-8" />}
      actions={
        <OperationSelector selected={operation} onSelect={setOperation} />
      }
    >
      <div className="flex flex-col gap-4">
        <UpdatesTable updates={updates?.updates ?? []} showTarget />
        <div className="flex gap-4 justify-center items-center text-muted-foreground">
          <Button
            variant="outline"
            onClick={() => setPage(page - 1)}
            disabled={page === 0}
          >
            Prev Page
          </Button>
          Page: {page + 1}
          <Button
            variant="outline"
            onClick={() => updates?.next_page && setPage(updates.next_page)}
            disabled={!updates?.next_page}
          >
            Next Page
          </Button>
        </div>
      </div>
    </Page>
  );
};

const ResourceUpdates = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const name = useRead(`List${type}s`, {}).data?.find((r) => r.id === id)?.name;
  useSetTitle(name && `${name} | Updates`);
  const [operation, setOperation] = useState<Types.Operation | undefined>();
  const [page, setPage] = useState(0);
  const updates = useRead("ListUpdates", {
    query: {
      "target.type": type,
      "target.id": id,
      operation,
    },
    page,
  }).data;
  const Components = ResourceComponents[type];
  return (
    <Page
      title={name}
      titleRight={<h2 className="text-muted-foreground">Updates</h2>}
      icon={<Components.BigIcon id={id} />}
      actions={
        <OperationSelector
          selected={operation}
          onSelect={setOperation}
          options={OPERATIONS_BY_RESOURCE[type]}
        />
      }
    >
      <div className="flex flex-col gap-4">
        <UpdatesTable updates={updates?.updates ?? []} />
        <div className="flex gap-4 justify-center items-center text-muted-foreground">
          <Button
            variant="outline"
            onClick={() => setPage(page - 1)}
            disabled={page === 0}
          >
            Prev Page
          </Button>
          Page: {page + 1}
          <Button
            variant="outline"
            onClick={() => updates?.next_page && setPage(updates.next_page)}
            disabled={!updates?.next_page}
          >
            Next Page
          </Button>
        </div>
      </div>
    </Page>
  );
};

const OPERATIONS_BY_RESOURCE: { [key: string]: Types.Operation[] } = {
  Server: [
    Types.Operation.CreateServer,
    Types.Operation.UpdateServer,
    Types.Operation.DeleteServer,
    Types.Operation.RenameServer,
    Types.Operation.StartContainer,
    Types.Operation.RestartContainer,
    Types.Operation.PauseContainer,
    Types.Operation.UnpauseContainer,
    Types.Operation.StopContainer,
    Types.Operation.DestroyContainer,
    Types.Operation.StartAllContainers,
    Types.Operation.RestartAllContainers,
    Types.Operation.PauseAllContainers,
    Types.Operation.UnpauseAllContainers,
    Types.Operation.StopAllContainers,
    Types.Operation.PruneContainers,
    Types.Operation.CreateNetwork,
    Types.Operation.DeleteNetwork,
    Types.Operation.PruneNetworks,
    Types.Operation.DeleteImage,
    Types.Operation.PruneImages,
    Types.Operation.DeleteVolume,
    Types.Operation.PruneVolumes,
    Types.Operation.PruneDockerBuilders,
    Types.Operation.PruneBuildx,
    Types.Operation.PruneSystem,
  ],
  Stack: [
    Types.Operation.CreateStack,
    Types.Operation.UpdateStack,
    Types.Operation.RenameStack,
    Types.Operation.DeleteStack,
    Types.Operation.WriteStackContents,
    Types.Operation.RefreshStackCache,
    Types.Operation.DeployStack,
    Types.Operation.StartStack,
    Types.Operation.RestartStack,
    Types.Operation.PauseStack,
    Types.Operation.UnpauseStack,
    Types.Operation.StopStack,
    Types.Operation.DestroyStack,
    Types.Operation.StartStackService,
    Types.Operation.RestartStackService,
    Types.Operation.PauseStackService,
    Types.Operation.UnpauseStackService,
    Types.Operation.StopStackService,
  ],
  Deployment: [
    Types.Operation.CreateDeployment,
    Types.Operation.UpdateDeployment,
    Types.Operation.DeleteDeployment,
    Types.Operation.Deploy,
    Types.Operation.StartDeployment,
    Types.Operation.RestartDeployment,
    Types.Operation.PauseDeployment,
    Types.Operation.UnpauseDeployment,
    Types.Operation.StopDeployment,
    Types.Operation.DestroyDeployment,
    Types.Operation.RenameDeployment,
  ],
  Build: [
    Types.Operation.CreateBuild,
    Types.Operation.UpdateBuild,
    Types.Operation.DeleteBuild,
    Types.Operation.RunBuild,
    Types.Operation.CancelBuild,
  ],
  Repo: [
    Types.Operation.CreateRepo,
    Types.Operation.UpdateRepo,
    Types.Operation.DeleteRepo,
    Types.Operation.CloneRepo,
    Types.Operation.PullRepo,
    Types.Operation.BuildRepo,
    Types.Operation.CancelRepoBuild,
  ],
  Procedure: [
    Types.Operation.CreateProcedure,
    Types.Operation.UpdateProcedure,
    Types.Operation.DeleteProcedure,
    Types.Operation.RunProcedure,
  ],
  Builder: [
    Types.Operation.CreateBuilder,
    Types.Operation.UpdateBuilder,
    Types.Operation.DeleteBuilder,
  ],
  Alerter: [
    Types.Operation.CreateAlerter,
    Types.Operation.UpdateAlerter,
    Types.Operation.DeleteAlerter,
  ],
  ResourceSync: [
    Types.Operation.CreateResourceSync,
    Types.Operation.UpdateResourceSync,
    Types.Operation.DeleteResourceSync,
    Types.Operation.CommitSync,
    Types.Operation.RunSync,
  ],
};

const OperationSelector = ({
  selected,
  onSelect,
  options = Object.values(Types.Operation).filter(
    (o) => o !== Types.Operation.None
  ),
}: {
  selected: Types.Operation | undefined;
  onSelect: (operation: Types.Operation | undefined) => void;
  options?: Types.Operation[];
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const filtered = filterBySplit(options, search, (item) => item);
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <div className="h-full w-[200px] cursor-pointer flex items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1">
          {selected ?? "Select Operation"}
          <CaretSortIcon className="h-4 w-4 opacity-50" />
        </div>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[200px] max-h-[200px] p-0">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search Operations"
            value={search}
            onValueChange={setSearch}
            className="h-9"
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center">
              No Operations Found
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              <CommandItem
                className="cursor-pointer"
                onSelect={() => {
                  onSelect(undefined);
                  setOpen(false);
                }}
              >
                <div>All</div>
              </CommandItem>

              {filtered.map((operation) => (
                <CommandItem
                  key={operation}
                  className="cursor-pointer"
                  onSelect={() => {
                    onSelect(operation);
                    setOpen(false);
                  }}
                >
                  {operation}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};
