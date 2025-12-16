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
import { Button } from "@ui/button";
import { Label } from "@ui/label";
import { language_from_path, MonacoEditor } from "@components/monaco";

export const SwarmSecrets = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const secrets =
    useRead("ListSwarmSecrets", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    secrets,
    search,
    (secret) => secret.Name ?? secret.ID ?? "Unknown"
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          <CreateSwarmSecret id={id} />
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
        tableKey="swarm-secrets"
        data={filtered}
        columns={[
          {
            accessorKey: "Name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Secret"
                swarm_id={id}
                resource_id={row.original.Name}
                name={row.original.Name}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => row.original.ID ?? "Unknown",
            size: 200,
          },
          {
            accessorKey: "Driver",
            header: ({ column }) => (
              <SortableHeader column={column} title="Driver" />
            ),
            cell: ({ row }) =>
              row.original.Driver ?? (
                <div className="text-muted-foreground">None</div>
              ),
          },
          {
            accessorKey: "Templating",
            header: ({ column }) => (
              <SortableHeader column={column} title="Templating" />
            ),
            cell: ({ row }) =>
              row.original.Templating ?? (
                <div className="text-muted-foreground">None</div>
              ),
          },
          {
            accessorKey: "UpdatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Updated" />
            ),
            cell: ({ row }) =>
              row.original.UpdatedAt
                ? new Date(row.original.UpdatedAt).toLocaleString()
                : "Unknown",
            size: 200,
          },
          {
            accessorKey: "CreatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Created" />
            ),
            cell: ({ row }) =>
              row.original.CreatedAt
                ? new Date(row.original.CreatedAt).toLocaleString()
                : "Unknown",
            size: 200,
          },
        ]}
      />
    </Section>
  );
};

const CreateSwarmSecret = ({ id }: { id: string }) => {
  const { canExecute } = usePermissions({ type: "Swarm", id });
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [data, setData] = useState("");
  const { toast } = useToast();
  const nav = useNavigate();
  const { mutate, isPending } = useExecute("CreateSwarmSecret", {
    onSuccess: () => {
      toast({ title: "Created swarm secret." });
      setName("");
      setData("");
      setOpen(false);
      nav(`/swarms/${id}/swarm-secret/${name}`);
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
          New Secret <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[90vw] max-w-[800px]">
        <DialogHeader>
          <DialogTitle>New Swarm Secret</DialogTitle>
          <DialogDescription>
            <p>Enter a unique name and data for the new Swarm Secret.</p>
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-6 py-8">
          <div className="flex flex-col gap-2">
            <Label htmlFor="secret-name">Name</Label>
            <Input
              id="secret-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full lg:w-[300px]"
              autoFocus
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="secret-data">Data</Label>
            <MonacoEditor
              id="secret-data"
              value={data}
              onValueChange={setData}
              language={language}
            />
          </div>
          <h2 className="font-extrabold">
            Note: the data cannot be viewed again after creation.
          </h2>
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
