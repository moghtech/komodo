import { Section } from "@components/layouts";
import { useExecute, usePermissions, useRead } from "@lib/hooks";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Dispatch, ReactNode, SetStateAction, useMemo, useState } from "react";
import { PlusCircle, Search } from "lucide-react";
import { Input } from "@ui/input";
import { cn, filterBySplit } from "@lib/utils";
import { SwarmResourceLink } from "..";
import { useToast } from "@ui/use-toast";
import { useNavigate } from "react-router-dom";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Label } from "@ui/label";
import { Button } from "@ui/button";
import { language_from_path, MonacoEditor } from "@components/monaco";
import { Badge } from "@ui/badge";

export const SwarmConfigs = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const configs =
    useRead("ListSwarmConfigs", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    configs,
    search,
    (config) => config.Name ?? config.ID ?? "Unknown"
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          <CreateSwarmConfig id={id} />
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
        </div>
      }
    >
      <DataTable
        containerClassName="min-h-[60vh]"
        tableKey="swarm-configs"
        data={filtered}
        columns={[
          {
            accessorKey: "Name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Config"
                swarm_id={id}
                resource_id={row.original.Name}
                name={row.original.Name}
                extra={
                  !row.original.InUse && (
                    <Badge variant="destructive">Unused</Badge>
                  )
                }
              />
            ),
          },
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => row.original.ID ?? "Unknown",
          },
          {
            accessorKey: "UpdatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Updated" />
            ),
          },
          {
            accessorKey: "CreatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Created" />
            ),
          },
        ]}
      />
    </Section>
  );
};

const CreateSwarmConfig = ({ id }: { id: string }) => {
  const { canExecute } = usePermissions({ type: "Swarm", id });
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [data, setData] = useState("");
  const { toast } = useToast();
  const nav = useNavigate();
  const { mutate, isPending } = useExecute("CreateSwarmConfig", {
    onSuccess: () => {
      toast({ title: "Created swarm config." });
      setName("");
      setData("");
      setOpen(false);
      nav(`/swarms/${id}/swarm-config/${name}`);
    },
  });
  const language = useMemo(() => language_from_path(name), [name]);
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger>
        <Button
          className={cn("items-center gap-2", !canExecute && "hidden")}
          variant="secondary"
          disabled={isPending}
        >
          New Config <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[90vw] max-w-[800px]">
        <DialogHeader>
          <DialogTitle>New Swarm Config</DialogTitle>
          <DialogDescription>
            <p>Enter a unique name and data for the new Swarm Config.</p>
            <p>
              For better file syntax support, choose a name with appropriate
              file extension, like <code>config.yaml</code>
            </p>
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-6 py-8">
          <div className="flex flex-col gap-2">
            <Label htmlFor="config-name">Name</Label>
            <Input
              id="config-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full lg:w-[300px]"
              autoFocus
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="config-data">Data</Label>
            <MonacoEditor
              id="config-data"
              value={data}
              onValueChange={setData}
              language={language}
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="secondary"
            onClick={() => {
              mutate({ swarm: id, name, data });
            }}
            disabled={isPending}
          >
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
