import {
  ActionWithDialog,
  ConfirmButton,
  CopyButton,
  RepoLink,
  TextUpdateMenuSimple,
} from "@components/util";
import {
  useInvalidate,
  useRead,
  useWrite,
  WebhookIntegration,
} from "@lib/hooks";
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
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Check,
  ChevronsUpDown,
  Copy,
  Loader2,
  NotepadText,
  SearchX,
  Server,
  Trash,
} from "lucide-react";
import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ResourceComponents } from ".";
import { Input } from "@ui/input";
import { useToast } from "@ui/use-toast";
import { NewLayout } from "@components/layouts";
import { Types } from "komodo_client";
import { filterBySplit, usableResourcePath } from "@lib/utils";

export const ResourceDescription = ({
  type,
  id,
  disabled,
}: {
  type: UsableResource;
  id: string;
  disabled: boolean;
}) => {
  const { toast } = useToast();
  const inv = useInvalidate();

  const key = type === "ResourceSync" ? "sync" : type.toLowerCase();

  const resource = useRead(`Get${type}`, {
    [key]: id,
  } as any).data;

  const { mutate: update_description } = useWrite("UpdateDescription", {
    onSuccess: () => {
      inv([`Get${type}`]);
      toast({ title: `Updated description on ${type} ${resource?.name}` });
    },
  });

  return (
    <TextUpdateMenuSimple
      title="Update Description"
      placeholder="Set Description"
      value={resource?.description}
      onUpdate={(description) =>
        update_description({
          target: { type, id },
          description,
        })
      }
      triggerClassName="text-muted-foreground"
      disabled={disabled}
    />
  );
};

export const ResourceSelector = ({
  type,
  selected,
  onSelect,
  disabled,
  align,
}: {
  type: UsableResource;
  selected: string | undefined;
  onSelect?: (id: string) => void;
  disabled?: boolean;
  align?: "start" | "center" | "end";
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");

  const resources = useRead(`List${type}s`, {}).data;
  const name = resources?.find((r) => r.id === selected)?.name;

  if (!resources) return null;

  const filtered = filterBySplit(
    resources as Types.ResourceListItem<unknown>[],
    search,
    (item) => item.name
  ).sort((a, b) => {
    if (a.name > b.name) {
      return 1;
    } else if (a.name < b.name) {
      return -1;
    } else {
      return 0;
    }
  });

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="secondary"
          className="flex justify-start gap-2 w-fit max-w-[350px]"
          disabled={disabled}
        >
          {name || `Select ${type}`}
          {!disabled && <ChevronsUpDown className="w-3 h-3" />}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[300px] max-h-[300px] p-0" align={align}>
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={`Search ${type}s`}
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center pt-3 pb-2">
              {`No ${type}s Found`}
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              {!search && (
                <CommandItem
                  onSelect={() => {
                    onSelect && onSelect("");
                    setOpen(false);
                  }}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">None</div>
                </CommandItem>
              )}
              {filtered.map((resource) => (
                <CommandItem
                  key={resource.id}
                  onSelect={() => {
                    onSelect && onSelect(resource.id);
                    setOpen(false);
                  }}
                  className="flex items-center justify-between cursor-pointer"
                >
                  <div className="p-1">{resource.name}</div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

export const ResourceLink = ({
  type,
  id,
  onClick,
}: {
  type: UsableResource;
  id: string;
  onClick?: () => void;
}) => {
  const Components = ResourceComponents[type];
  return (
    <Link
      to={`/${usableResourcePath(type)}/${id}`}
      onClick={(e) => {
        e.stopPropagation();
        onClick?.();
      }}
      className="flex items-center gap-2 text-sm hover:underline"
    >
      <Components.Icon id={id} />
      <ResourceName type={type} id={id} />
    </Link>
  );
};

export const ResourceName = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const Components = ResourceComponents[type];
  const name = Components.list_item(id)?.name ?? "unknown";
  return <>{name}</>;
};

export const CopyResource = ({
  id,
  disabled,
  type,
}: {
  id: string;
  disabled?: boolean;
  type: Exclude<UsableResource, "Server">;
}) => {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");

  const nav = useNavigate();
  const inv = useInvalidate();
  const { mutate } = useWrite(`Copy${type}`, {
    onSuccess: (res) => {
      inv([`List${type}s`]);
      nav(`/${usableResourcePath(type)}/${res._id?.$oid}`);
    },
  });

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="secondary"
          className="flex gap-2 items-center"
          onClick={() => setOpen(true)}
          disabled={disabled}
        >
          <Copy className="w-4 h-4" />
          Copy
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Copy {type}</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 my-4">
          <p>Provide a name for the newly created {type}.</p>
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <DialogFooter>
          <ConfirmButton
            title="Copy"
            icon={<Check className="w-4 h-4" />}
            disabled={!name}
            onClick={() => {
              mutate({ id, name });
              setOpen(false);
            }}
          />
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export const NewResource = ({
  type,
  readable_type,
  server_id,
  builder_id,
  build_id,
  name: _name = "",
}: {
  type: UsableResource;
  readable_type?: string;
  server_id?: string;
  builder_id?: string;
  build_id?: string;
  name?: string;
}) => {
  const nav = useNavigate();
  const { toast } = useToast();
  const { mutateAsync } = useWrite(`Create${type}`);
  const [name, setName] = useState(_name);
  const type_display =
    type === "ResourceSync" ? "resource-sync" : type.toLowerCase();
  const config: Types._PartialDeploymentConfig | Types._PartialRepoConfig =
    type === "Deployment"
      ? {
          server_id,
          image: build_id
            ? { type: "Build", params: { build_id } }
            : { type: "Image", params: { image: "" } },
        }
      : type === "Stack"
        ? { server_id }
        : type === "Repo"
          ? { server_id, builder_id }
          : type === "Build"
            ? { builder_id }
            : {};
  const onConfirm = async () => {
    if (!name) toast({ title: "Name cannot be empty" });
    const id = (await mutateAsync({ name, config }))._id?.$oid!;
    nav(`/${usableResourcePath(type)}/${id}`);
  };
  return (
    <NewLayout
      entityType={readable_type ?? type}
      onConfirm={onConfirm}
      enabled={!!name}
      onOpenChange={() => setName(_name)}
    >
      <div className="grid md:grid-cols-2 items-center">
        {readable_type ?? type} Name
        <Input
          placeholder={`${type_display}-name`}
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (!name) return;
            if (e.key === "Enter") {
              onConfirm();
            }
          }}
        />
      </div>
    </NewLayout>
  );
};

export const DeleteResource = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const nav = useNavigate();
  const key = type === "ResourceSync" ? "sync" : type.toLowerCase();
  const resource = useRead(`Get${type}`, {
    [key]: id,
  } as any).data;
  const { mutateAsync, isPending } = useWrite(`Delete${type}`);

  if (!resource) return null;

  return (
    <div className="flex items-center justify-end">
      <ActionWithDialog
        name={resource.name}
        title="Delete"
        variant="destructive"
        icon={<Trash className="h-4 w-4" />}
        onClick={async () => {
          await mutateAsync({ id });
          nav(`/${usableResourcePath(type)}`);
        }}
        disabled={isPending}
        loading={isPending}
        forceConfirmDialog
      />
    </div>
  );
};

export const CopyWebhook = ({
  integration,
  path,
}: {
  integration: WebhookIntegration;
  path: string;
}) => {
  const base_url = useRead("GetCoreInfo", {}).data?.webhook_base_url;
  const url = base_url + "/listener/" + integration.toLowerCase() + path;
  return (
    <div className="flex gap-2 items-center">
      <Input className="w-[400px] max-w-[70vw]" value={url} readOnly />
      <CopyButton content={url} />
    </div>
  );
};

export const StandardSource = ({
  info,
}: {
  info:
    | {
        files_on_host: boolean;
        repo: string;
        repo_link: string;
      }
    | undefined;
}) => {
  if (!info) {
    return <Loader2 className="w-4 h-4 animate-spin" />;
  }
  if (info.files_on_host) {
    return (
      <div className="flex items-center gap-2">
        <Server className="w-4 h-4" />
        Files on Server
      </div>
    );
  }
  if (info.repo) {
    return <RepoLink repo={info.repo} link={info.repo_link} />;
  }
  return (
    <div className="flex items-center gap-2">
      <NotepadText className="w-4 h-4" />
      UI Defined
    </div>
  );
};
