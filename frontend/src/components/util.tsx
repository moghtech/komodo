import {
  Dispatch,
  FocusEventHandler,
  Fragment,
  MouseEventHandler,
  ReactNode,
  SetStateAction,
  forwardRef,
  useEffect,
  useRef,
  useState,
} from "react";
import { Button } from "../ui/button";
import {
  Box,
  Check,
  CheckCircle,
  ChevronDown,
  ChevronLeft,
  ChevronsUpDown,
  ChevronUp,
  Copy,
  Database,
  EthernetPort,
  FolderGit,
  HardDrive,
  LinkIcon,
  Loader2,
  Network,
  Search,
  SearchX,
  Settings,
  Tags,
  User,
} from "lucide-react";
import { Input } from "../ui/input";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogContent,
  DialogFooter,
  DialogDescription,
} from "@ui/dialog";
import { toast, useToast } from "@ui/use-toast";
import { cn, filterBySplit, usableResourcePath } from "@lib/utils";
import { Link, useNavigate } from "react-router-dom";
import { Textarea } from "@ui/textarea";
import { Card } from "@ui/card";
import {
  fmt_port_mount,
  fmt_resource_type,
  fmt_utc_offset,
  snake_case_to_upper_space_case,
} from "@lib/formatting";
import {
  ColorIntention,
  container_state_intention,
  hex_color_by_intention,
  stroke_color_class_by_intention,
  text_color_class_by_intention,
} from "@lib/color";
import { Types } from "komodo_client";
import { Badge } from "@ui/badge";
import { Section } from "./layouts";
import { DataTable, SortableHeader } from "@ui/data-table";
import {
  useContainerPortsMap,
  useRead,
  useTemplatesQueryBehavior,
} from "@lib/hooks";
import { Prune } from "./resources/server/actions";
import { MonacoEditor, MonacoLanguage } from "./monaco";
import { UsableResource } from "@types";
import { ResourceComponents } from "./resources";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { useServer } from "./resources/server";

export const WithLoading = ({
  children,
  isLoading,
  loading,
  isError,
  error,
}: {
  children: ReactNode;
  isLoading: boolean;
  loading?: ReactNode;
  isError: boolean;
  error?: ReactNode;
}) => {
  if (isLoading) return <>{loading ?? "loading"}</>;
  if (isError) return <>{error ?? null}</>;
  return <>{children}</>;
};

export const ActionButton = forwardRef<
  HTMLButtonElement,
  {
    variant?:
      | "link"
      | "default"
      | "destructive"
      | "outline"
      | "secondary"
      | "ghost"
      | null
      | undefined;
    size?: "default" | "sm" | "lg" | "icon" | null | undefined;
    title: string;
    icon: ReactNode;
    disabled?: boolean;
    className?: string;
    onClick?: MouseEventHandler<HTMLButtonElement>;
    onBlur?: FocusEventHandler<HTMLButtonElement>;
    loading?: boolean;
    "data-confirm-button"?: boolean;
  }
>(
  (
    {
      variant,
      size,
      title,
      icon,
      disabled,
      className,
      loading,
      onClick,
      onBlur,
      "data-confirm-button": dataConfirmButton,
    },
    ref
  ) => (
    <Button
      size={size}
      variant={variant || "secondary"}
      className={cn(
        "flex flex-1 shrink-0 gap-4 items-center justify-between max-w-[190px]",
        className
      )}
      onClick={onClick}
      onBlur={onBlur}
      disabled={disabled || loading}
      ref={ref}
      data-confirm-button={dataConfirmButton}
    >
      {title} {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : icon}
    </Button>
  )
);

export const ActionWithDialog = ({
  name,
  title,
  icon,
  disabled,
  loading,
  onClick,
  additional,
  targetClassName,
  variant,
}: {
  name: string;
  title: string;
  icon: ReactNode;
  disabled?: boolean;
  loading?: boolean;
  onClick?: () => void;
  additional?: ReactNode;
  targetClassName?: string;
  variant?:
    | "link"
    | "default"
    | "destructive"
    | "outline"
    | "secondary"
    | "ghost"
    | null
    | undefined;
}) => {
  const [open, setOpen] = useState(false);
  const [input, setInput] = useState("");
  const confirmButtonRef = useRef<HTMLButtonElement>(null);

  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        setOpen(open);
        setInput("");
      }}
    >
      <DialogTrigger asChild>
        <ActionButton
          className={targetClassName}
          title={title}
          icon={icon}
          disabled={disabled}
          onClick={() => setOpen(true)}
          loading={loading}
          variant={variant}
        />
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm {title}</DialogTitle>
          <DialogDescription>
            Please enter the name below to confirm this action.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4 my-4">
          <p
            onClick={() => {
              navigator.clipboard.writeText(name);
              toast({ title: `Copied "${name}" to clipboard!` });
            }}
            className="cursor-pointer"
          >
            Please enter <b>{name}</b> below to confirm this action.
            <br />
            <span className="text-xs text-muted-foreground">
              You may click the name in bold to copy it
            </span>
          </p>
          <Input value={input} onChange={(e) => setInput(e.target.value)} />
          {additional}
        </div>
        <DialogFooter>
          <ConfirmButton
            ref={confirmButtonRef}
            title={title}
            icon={icon}
            disabled={disabled || name !== input}
            onClick={() => {
              onClick && onClick();
              setOpen(false);
            }}
          />
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export const ConfirmButton = forwardRef<
  HTMLButtonElement,
  {
    variant?:
      | "link"
      | "default"
      | "destructive"
      | "outline"
      | "secondary"
      | "ghost"
      | null
      | undefined;
    size?: "default" | "sm" | "lg" | "icon" | null | undefined;
    title: string;
    icon: ReactNode;
    onClick?: MouseEventHandler<HTMLButtonElement>;
    loading?: boolean;
    disabled?: boolean;
    className?: string;
  }
>(({
  variant,
  size,
  title,
  icon,
  disabled,
  loading,
  onClick,
  className,
}, ref) => {
  const [confirmed, set] = useState(false);

  return (
    <ActionButton
      ref={ref}
      variant={variant}
      size={size}
      title={confirmed ? "Confirm" : title}
      icon={confirmed ? <Check className="w-4 h-4" /> : icon}
      disabled={disabled}
      onClick={
        confirmed
          ? (e) => {
              e.stopPropagation();
              onClick && onClick(e);
              set(false);
            }
          : (e) => {
              e.stopPropagation();
              set(true);
            }
      }
      onBlur={() => set(false)}
      loading={loading}
      className={className}
      data-confirm-button={true}
    />
  );
});

export const UserSettings = () => (
  <Link to="/settings">
    <Button variant="ghost" size="icon">
      <Settings className="w-4 h-4" />
    </Button>
  </Link>
);

export const CopyButton = ({
  content,
  className,
  icon = <Copy className="w-4 h-4" />,
  label = "selection",
}: {
  content: string | undefined;
  className?: string;
  icon?: ReactNode;
  label?: string;
}) => {
  const { toast } = useToast();
  const [copied, set] = useState(false);

  useEffect(() => {
    if (copied) {
      toast({ title: "Copied " + label });
      const timeout = setTimeout(() => set(false), 3000);
      return () => {
        clearTimeout(timeout);
      };
    }
  }, [content, copied, toast]);

  return (
    <Button
      className={cn("shrink-0", className)}
      size="icon"
      variant="outline"
      onClick={() => {
        if (!content) return;
        navigator.clipboard.writeText(content);
        set(true);
      }}
      disabled={!content}
    >
      {copied ? <Check className="w-4 h-4" /> : icon}
    </Button>
  );
};

export const TextUpdateMenuMonaco = ({
  title,
  titleRight,
  value = "",
  triggerClassName,
  onUpdate,
  placeholder,
  confirmButton,
  disabled,
  fullWidth,
  open,
  setOpen,
  triggerHidden,
  language,
  triggerChild,
}: {
  title: string;
  titleRight?: ReactNode;
  value: string | undefined;
  onUpdate: (value: string) => void;
  triggerClassName?: string;
  placeholder?: string;
  confirmButton?: boolean;
  disabled?: boolean;
  fullWidth?: boolean;
  open?: boolean;
  setOpen?: (open: boolean) => void;
  triggerHidden?: boolean;
  language?: MonacoLanguage;
  triggerChild?: ReactNode;
}) => {
  const [_open, _setOpen] = useState(false);
  const [__open, __setOpen] = [open ?? _open, setOpen ?? _setOpen];
  const [_value, setValue] = useState(value);
  useEffect(() => setValue(value), [value]);
  const onClick = () => {
    onUpdate(_value);
    __setOpen(false);
  };

  return (
    <Dialog open={__open} onOpenChange={__setOpen}>
      <DialogTrigger asChild>
        {triggerChild ?? (
          <Card
            className={cn(
              "px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer",
              fullWidth ? "w-full" : "w-fit",
              triggerHidden && "hidden"
            )}
          >
            <div
              className={cn(
                "text-sm text-nowrap overflow-hidden overflow-ellipsis",
                (!value || !!disabled) && "text-muted-foreground",
                triggerClassName
              )}
            >
              {value.split("\n")[0] || placeholder}
            </div>
          </Card>
        )}
      </DialogTrigger>
      <DialogContent className="min-w-[50vw]">
        {titleRight && (
          <div className="flex items-center gap-4">
            <DialogHeader>
              <DialogTitle>{title}</DialogTitle>
            </DialogHeader>
            {titleRight}
          </div>
        )}
        {!titleRight && (
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
          </DialogHeader>
        )}

        <MonacoEditor
          value={_value}
          language={language}
          onValueChange={setValue}
          readOnly={disabled}
        />

        {!disabled && (
          <DialogFooter>
            {confirmButton ? (
              <ConfirmButton
                title="Update"
                icon={<CheckCircle className="w-4 h-4" />}
                onClick={onClick}
              />
            ) : (
              <Button
                variant="secondary"
                onClick={onClick}
                className="flex items-center gap-2"
              >
                <CheckCircle className="w-4 h-4" />
                Update
              </Button>
            )}
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
};

export const UserAvatar = ({
  avatar,
  size = 4,
}: {
  avatar: string | undefined;
  size?: number;
}) =>
  avatar ? (
    <img src={avatar} alt="Avatar" className={`w-${size} h-${size}`} />
  ) : (
    <User className={`w-${size} h-${size}`} />
  );

export const StatusBadge = ({
  text,
  intent,
}: {
  text: string | undefined;
  intent: ColorIntention;
}) => {
  if (!text) return null;

  const color = text_color_class_by_intention(intent);
  const background = hex_color_by_intention(intent) + "25";

  const _text = text === Types.ServerState.NotOk ? "Not Ok" : text;

  return (
    <p
      className={cn(
        "px-2 py-1 w-fit text-xs text-white rounded-md font-medium tracking-wide",
        color
      )}
      style={{ background }}
    >
      {snake_case_to_upper_space_case(_text).toUpperCase()}
    </p>
  );
};

export const DockerOptions = ({
  options,
}: {
  options: Record<string, string> | undefined;
}) => {
  if (!options) return null;
  const entries = Object.entries(options);
  if (entries.length === 0) return null;
  return (
    <div className="flex gap-2 flex-wrap">
      {entries.map(([key, value]) => (
        <Badge key={key} variant="secondary">
          {key} = {value}
        </Badge>
      ))}
    </div>
  );
};

export const DockerLabelsSection = ({
  labels,
}: {
  labels: Record<string, string> | undefined;
}) => {
  if (!labels) return null;
  const entries = Object.entries(labels);
  if (entries.length === 0) return null;
  return (
    <Section title="Labels" icon={<Tags className="w-4 h-4" />}>
      <div className="flex gap-2 flex-wrap">
        {entries.map(([key, value]) => (
          <Badge key={key} variant="secondary" className="flex gap-1">
            <span className="text-muted-foreground">{key}</span>
            <span className="text-muted-foreground">=</span>
            <span
              title={value}
              className="font-extrabold text-nowrap max-w-[200px] overflow-hidden text-ellipsis"
            >
              {value}
            </span>
          </Badge>
        ))}
      </div>
    </Section>
  );
};

export const ShowHideButton = ({
  show,
  setShow,
}: {
  show: boolean;
  setShow: (show: boolean) => void;
}) => {
  return (
    <Button
      size="sm"
      variant="outline"
      className="gap-4"
      onClick={() => setShow(!show)}
    >
      {show ? "Hide" : "Show"}
      {show ? <ChevronUp className="w-4" /> : <ChevronDown className="w-4" />}
    </Button>
  );
};

type DockerResourceType = "container" | "network" | "image" | "volume";

export const DOCKER_LINK_ICONS: {
  [type in DockerResourceType]: React.FC<{
    server_id: string;
    name: string | undefined;
    size?: number;
  }>;
} = {
  container: ({ server_id, name, size = 4 }) => {
    const state =
      useRead("ListDockerContainers", { server: server_id }).data?.find(
        (container) => container.name === name
      )?.state ?? Types.ContainerStateStatusEnum.Empty;
    return (
      <Box
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(container_state_intention(state))
        )}
      />
    );
  },
  network: ({ server_id, name, size = 4 }) => {
    const containers =
      useRead("ListDockerContainers", { server: server_id }).data ?? [];
    const no_containers = !name
      ? false
      : containers.every((container) => !container.networks?.includes(name));
    return (
      <Network
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(
            !name
              ? "Warning"
              : no_containers
                ? ["none", "host", "bridge"].includes(name)
                  ? "None"
                  : "Critical"
                : "Good"
          )
        )}
      />
    );
  },
  image: ({ server_id, name, size = 4 }) => {
    const containers =
      useRead("ListDockerContainers", { server: server_id }).data ?? [];
    const no_containers = !name
      ? false
      : containers.every((container) => container.image_id !== name);
    return (
      <HardDrive
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(
            !name ? "Warning" : no_containers ? "Critical" : "Good"
          )
        )}
      />
    );
  },
  volume: ({ server_id, name, size = 4 }) => {
    const containers =
      useRead("ListDockerContainers", { server: server_id }).data ?? [];
    const no_containers = !name
      ? false
      : containers.every((container) => !container.volumes?.includes(name));
    return (
      <Database
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(
            !name ? "Warning" : no_containers ? "Critical" : "Good"
          )
        )}
      />
    );
  },
};

export const DockerResourceLink = ({
  server_id,
  name,
  id,
  type,
  extra,
  muted,
}: {
  server_id: string;
  name: string | undefined;
  id?: string;
  type: "container" | "network" | "image" | "volume";
  extra?: ReactNode;
  muted?: boolean;
}) => {
  if (!name) return "Unknown";

  const Icon = DOCKER_LINK_ICONS[type];

  return (
    <Link
      to={`/servers/${server_id}/${type}/${encodeURIComponent(name)}`}
      className={cn(
        "flex items-center gap-2 text-sm hover:underline py-1",
        muted && "text-muted-foreground"
      )}
    >
      <Icon server_id={server_id} name={type === "image" ? id : name} />
      <div
        title={name}
        className="max-w-[250px] lg:max-w-[300px] overflow-hidden overflow-ellipsis break-words"
      >
        {name}
      </div>
      {extra && <div className="no-underline">{extra}</div>}
    </Link>
  );
};

export const DockerResourcePageName = ({ name: _name }: { name?: string }) => {
  const name = _name ?? "Unknown";
  return (
    <h1
      title={name}
      className="text-3xl max-w-[300px] md:max-w-[500px] xl:max-w-[700px] overflow-hidden overflow-ellipsis"
    >
      {name}
    </h1>
  );
};

export const DockerContainersSection = ({
  server_id,
  containers,
  show = true,
  setShow,
  pruneButton,
  titleOther,
  forceTall,
  _search,
}: {
  server_id: string;
  containers: Types.ListDockerContainersResponse;
  show?: boolean;
  setShow?: (show: boolean) => void;
  pruneButton?: boolean;
  titleOther?: ReactNode;
  forceTall?: boolean;
  _search?: [string, Dispatch<SetStateAction<string>>];
}) => {
  const allRunning = useRead("ListDockerContainers", {
    server: server_id,
  }).data?.every(
    (container) => container.state === Types.ContainerStateStatusEnum.Running
  );
  const filtered = _search
    ? filterBySplit(containers, _search[0], (container) => container.name)
    : containers;
  return (
    <div className={cn(setShow && show && "mb-8")}>
      <Section
        titleOther={titleOther}
        title={!titleOther ? "Containers" : undefined}
        icon={!titleOther ? <Box className="w-4 h-4" /> : undefined}
        actions={
          <div className="flex items-center gap-4">
            {pruneButton && !allRunning && (
              <Prune server_id={server_id} type="Containers" />
            )}
            {_search && (
              <div className="relative">
                <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
                <Input
                  value={_search[0]}
                  onChange={(e) => _search[1](e.target.value)}
                  placeholder="search..."
                  className="pl-8 w-[200px] lg:w-[300px]"
                />
              </div>
            )}
            {setShow && <ShowHideButton show={show} setShow={setShow} />}
          </div>
        }
      >
        {show && (
          <DataTable
            containerClassName={forceTall ? "min-h-[60vh]" : undefined}
            tableKey="server-containers"
            data={filtered}
            columns={[
              {
                accessorKey: "name",
                size: 260,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Name" />
                ),
                cell: ({ row }) => (
                  <DockerResourceLink
                    type="container"
                    server_id={server_id}
                    name={row.original.name}
                  />
                ),
              },
              {
                accessorKey: "state",
                size: 160,
                header: ({ column }) => (
                  <SortableHeader column={column} title="State" />
                ),
                cell: ({ row }) => {
                  const state = row.original?.state;
                  return (
                    <StatusBadge
                      text={state}
                      intent={container_state_intention(state)}
                    />
                  );
                },
              },
              {
                accessorKey: "image",
                size: 300,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Image" />
                ),
                cell: ({ row }) => (
                  <DockerResourceLink
                    type="image"
                    server_id={server_id}
                    name={row.original.image}
                    id={row.original.image_id}
                  />
                ),
              },
              {
                accessorKey: "networks.0",
                size: 200,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Networks" />
                ),
                cell: ({ row }) =>
                  (row.original.networks?.length ?? 0) > 0 ? (
                    <div className="flex items-center gap-x-2 flex-wrap">
                      {row.original.networks?.map((network, i) => (
                        <Fragment key={network}>
                          <DockerResourceLink
                            type="network"
                            server_id={server_id}
                            name={network}
                          />
                          {i !== row.original.networks!.length - 1 && (
                            <div className="text-muted-foreground">|</div>
                          )}
                        </Fragment>
                      ))}
                    </div>
                  ) : (
                    row.original.network_mode && (
                      <DockerResourceLink
                        type="network"
                        server_id={server_id}
                        name={row.original.network_mode}
                      />
                    )
                  ),
              },
              {
                accessorKey: "ports.0",
                size: 200,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Ports" />
                ),
                cell: ({ row }) => (
                  <ContainerPortsTableView
                    ports={row.original.ports ?? []}
                    server_id={row.original.server_id}
                  />
                ),
              },
            ]}
          />
        )}
      </Section>
    </div>
  );
};

export const TextUpdateMenuSimple = ({
  title,
  titleRight,
  value = "",
  triggerClassName,
  onUpdate,
  placeholder,
  confirmButton,
  disabled,
  open,
  setOpen,
}: {
  title: string;
  titleRight?: ReactNode;
  value: string | undefined;
  onUpdate: (value: string) => void;
  triggerClassName?: string;
  placeholder?: string;
  confirmButton?: boolean;
  disabled?: boolean;
  open?: boolean;
  setOpen?: (open: boolean) => void;
}) => {
  const [_open, _setOpen] = useState(false);
  const [__open, __setOpen] = [open ?? _open, setOpen ?? _setOpen];
  const [_value, setValue] = useState(value);
  useEffect(() => setValue(value), [value]);
  const onClick = () => {
    onUpdate(_value);
    __setOpen(false);
  };

  return (
    <Dialog open={__open} onOpenChange={__setOpen}>
      <DialogTrigger asChild>
        <div
          className={cn(
            "text-sm text-nowrap overflow-hidden overflow-ellipsis p-2 border rounded-md flex-1 cursor-pointer hover:bg-accent/25",
            (!value || !!disabled) && "text-muted-foreground",
            triggerClassName
          )}
        >
          {value.split("\n")[0] || placeholder}
        </div>
      </DialogTrigger>
      <DialogContent className="min-w-[50vw]">
        {titleRight && (
          <div className="flex items-center gap-4">
            <DialogHeader>
              <DialogTitle>{title}</DialogTitle>
            </DialogHeader>
            {titleRight}
          </div>
        )}
        {!titleRight && (
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
          </DialogHeader>
        )}

        <Textarea
          value={_value}
          onChange={(e) => setValue(e.target.value)}
          placeholder={placeholder}
          className="min-h-[200px]"
          disabled={disabled}
        />
        {!disabled && (
          <DialogFooter>
            {confirmButton ? (
              <ConfirmButton
                title="Update"
                icon={<CheckCircle className="w-4 h-4" />}
                onClick={onClick}
              />
            ) : (
              <Button
                variant="secondary"
                onClick={onClick}
                className="flex items-center gap-2"
              >
                <CheckCircle className="w-4 h-4" />
                Update
              </Button>
            )}
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
};

export const NotFound = ({ type }: { type: UsableResource | undefined }) => {
  const nav = useNavigate();
  const Components = type && ResourceComponents[type];
  return (
    <div className="flex flex-col gap-4">
      {type && (
        <div className="flex items-center justify-between mb-4">
          <Button
            className="gap-2"
            variant="secondary"
            onClick={() => nav("/" + usableResourcePath(type))}
          >
            <ChevronLeft className="w-4" /> Back
          </Button>
        </div>
      )}
      <div className="grid lg:grid-cols-2 gap-4">
        <div className="flex items-center gap-4">
          <div className="mt-1">
            {Components ? (
              <Components.BigIcon />
            ) : (
              <SearchX className="w-8 h-8" />
            )}
          </div>
          <h1 className="text-3xl font-mono">
            {type} {type && " - "} 404 Not Found
          </h1>
        </div>
      </div>
    </div>
  );
};

export const RepoLink = ({ repo, link }: { repo: string; link: string }) => {
  return (
    <a
      target="_blank"
      href={link}
      className="text-sm cursor-pointer hover:underline"
    >
      <div className="flex items-center gap-2">
        <FolderGit className="w-4 h-4" />
        {repo}
      </div>
    </a>
  );
};

const TIMEZONES: ("Default" | Types.IanaTimezone)[] = [
  "Default",
  ...Object.values(Types.IanaTimezone),
];

export const TimezoneSelector = ({
  timezone,
  onChange,
  disabled,
  triggerClassName,
}: {
  timezone: string;
  onChange: (timezone: "" | Types.IanaTimezone) => void;
  disabled?: boolean;
  triggerClassName?: string;
}) => {
  const core_tz = useRead("GetCoreInfo", {}).data?.timezone || "Core TZ";
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const filtered = filterBySplit(TIMEZONES, search, (t) => t);

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <PopoverTrigger asChild>
        <Button
          variant="secondary"
          className={cn(
            "flex justify-between gap-2 w-[300px]",
            triggerClassName
          )}
          disabled={disabled}
        >
          {timezone
            ? `${timezone} (${fmt_utc_offset(timezone as Types.IanaTimezone)})`
            : `Default (${core_tz})`}
          {!disabled && <ChevronsUpDown className="w-3 h-3" />}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[300px] max-h-[300px] p-0 z-[100]">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={"Search Timezones"}
            className="h-9"
            value={search}
            onValueChange={setSearch}
          />
          <CommandList>
            <CommandEmpty className="flex justify-evenly items-center pt-3 pb-2">
              No Timezones Found
              <SearchX className="w-3 h-3" />
            </CommandEmpty>

            <CommandGroup>
              {filtered.map((timezone) =>
                timezone !== "Default" ? (
                  <CommandItem
                    key={timezone}
                    onSelect={() => {
                      onChange(timezone);
                      setOpen(false);
                    }}
                    className="flex items-center justify-between cursor-pointer"
                  >
                    <div className="p-1">
                      {timezone} ({fmt_utc_offset(timezone)})
                    </div>
                  </CommandItem>
                ) : (
                  <CommandItem
                    key={timezone}
                    onSelect={() => {
                      onChange("");
                      setOpen(false);
                    }}
                    className="flex items-center justify-between cursor-pointer"
                  >
                    <div className="p-1">Default ({core_tz})</div>
                  </CommandItem>
                )
              )}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

export const TemplateMarker = ({ type }: { type: UsableResource }) => {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge className="px-1 py-0">T</Badge>
      </TooltipTrigger>
      <TooltipContent>
        <div>This {fmt_resource_type(type).toLowerCase()} is a template.</div>
      </TooltipContent>
    </Tooltip>
  );
};

export const TemplateQueryBehaviorSelector = () => {
  const [value, set] = useTemplatesQueryBehavior();
  return (
    <Select
      value={value + " Templates"}
      onValueChange={(value) =>
        set(value.replace(" Templates", "") as Types.TemplatesQueryBehavior)
      }
    >
      <SelectTrigger className="w-[180px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {[
          Types.TemplatesQueryBehavior.Exclude,
          Types.TemplatesQueryBehavior.Include,
          Types.TemplatesQueryBehavior.Only,
        ].map((behavior) => (
          <SelectItem key={behavior} value={behavior + " Templates"}>
            {behavior} Templates
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

export const ContainerPortLink = ({
  host_port,
  ports,
  server_id,
}: {
  host_port: string;
  ports: Types.Port[];
  server_id: string | undefined;
}) => {
  const server = useServer(server_id);
  // Get the server address with periphery port removed
  const server_address = server?.info.external_address
    ? server.info.external_address
    : server?.info.address
        .split(":")
        // take just protocol and dns (indexes 0 and 1)
        .filter((_, i) => i < 2)
        .join(":");
  const link =
    host_port === "443"
      ? server_address
      : server_address?.replace("https", "http") + ":" + host_port;
  return (
    <Tooltip>
      <TooltipTrigger>
        <a
          target="_blank"
          href={link}
          className="text-sm cursor-pointer hover:underline px-1 py-2 flex items-center gap-2"
        >
          <EthernetPort
            className={cn("w-4 h-4", stroke_color_class_by_intention("Good"))}
          />
          {host_port}
        </a>
      </TooltipTrigger>
      <TooltipContent className="flex flex-col gap-2 w-fit">
        <a
          target="_blank"
          href={link}
          className="text-sm cursor-pointer hover:underline flex items-center gap-2"
        >
          <LinkIcon className="w-3 h-3" />
          {link}
        </a>
        {ports.map((port, i) => (
          <div key={i} className="flex gap-2 text-sm text-muted-foreground">
            <div>-</div>
            <div>{fmt_port_mount(port)}</div>
          </div>
        ))}
      </TooltipContent>
    </Tooltip>
  );
};

export const ContainerPortsTableView = ({
  ports,
  server_id,
}: {
  ports: Types.Port[];
  server_id: string | undefined;
}) => {
  const map = useContainerPortsMap(ports);
  const host_ports = Object.keys(map);
  return (
    <div className="flex items-center gap-x-1 flex-wrap">
      {host_ports.map((host_port, i) => {
        return (
          <Fragment key={host_port}>
            <ContainerPortLink
              host_port={host_port}
              ports={map[host_port]}
              server_id={server_id}
            />
            {i !== host_ports.length - 1 && (
              <div className="text-muted-foreground">|</div>
            )}
          </Fragment>
        );
      })}
    </div>
  );
};
