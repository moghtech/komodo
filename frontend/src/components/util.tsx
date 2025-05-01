import {
  FocusEventHandler,
  Fragment,
  MouseEventHandler,
  ReactNode,
  forwardRef,
  useEffect,
  useState,
} from "react";
import { Button } from "../ui/button";
import {
  Box,
  Check,
  CheckCircle,
  ChevronDown,
  ChevronLeft,
  ChevronUp,
  Copy,
  Database,
  Edit2,
  HardDrive,
  Loader2,
  LogOut,
  Network,
  SearchX,
  Settings,
  Tags,
  User,
  X,
} from "lucide-react";
import { Input } from "../ui/input";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogContent,
  DialogFooter,
} from "@ui/dialog";
import { toast, useToast } from "@ui/use-toast";
import { cn, usableResourcePath } from "@lib/utils";
import { Link, useNavigate } from "react-router-dom";
import { AUTH_TOKEN_STORAGE_KEY } from "@main";
import { Textarea } from "@ui/textarea";
import { Card } from "@ui/card";
import { snake_case_to_upper_space_case } from "@lib/formatting";
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
import { useInvalidate, useRead, useUser, useWrite } from "@lib/hooks";
import { Prune } from "./resources/server/actions";
import { MonacoEditor, MonacoLanguage } from "./monaco";
import { UsableResource } from "@types";
import { ResourceComponents } from "./resources";
import { useEditPermissions } from "@pages/resource";

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
    },
    ref
  ) => (
    <Button
      size={size}
      variant={variant || "secondary"}
      className={cn("flex items-center justify-between w-[190px]", className)}
      onClick={onClick}
      onBlur={onBlur}
      disabled={disabled || loading}
      ref={ref}
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
  const disable_confirm_dialog =
    useRead("GetCoreInfo", {}).data?.disable_confirm_dialog ?? false;
  const [open, setOpen] = useState(false);
  const [input, setInput] = useState("");

  if (disable_confirm_dialog) {
    return (
      <ConfirmButton
        variant={variant}
        title={title}
        icon={icon}
        disabled={disabled}
        loading={loading}
        className={targetClassName}
        onClick={onClick}
      />
    );
  }

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

export const ConfirmButton = ({
  variant,
  size,
  title,
  icon,
  disabled,
  loading,
  onClick,
  className,
}: {
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
}) => {
  const [confirmed, set] = useState(false);

  return (
    <ActionButton
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
    />
  );
};

export const Logout = () => {
  const user = useUser().data;
  return (
    user && (
      <Button
        variant="ghost"
        onClick={() => {
          localStorage.removeItem(AUTH_TOKEN_STORAGE_KEY);
          location.reload();
        }}
        className="px-2 flex flex-row gap-2 items-center"
      >
        <div className="hidden xl:flex max-w-[120px] overflow-hidden overflow-ellipsis">
          {user.username}
        </div>
        <LogOut className="w-4 h-4" />
      </Button>
    )
  );
};

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
}: {
  content: string | undefined;
  className?: string;
}) => {
  const { toast } = useToast();
  const [copied, set] = useState(false);

  useEffect(() => {
    if (copied) {
      toast({ title: "Copied selection" });
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
      {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
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
}: {
  server_id: string;
  containers: Types.ListDockerContainersResponse;
  show?: boolean;
  setShow?: (show: boolean) => void;
  pruneButton?: boolean;
  titleOther?: ReactNode;
}) => {
  const allRunning = useRead("ListDockerContainers", {
    server: server_id,
  }).data?.every(
    (container) => container.state === Types.ContainerStateStatusEnum.Running
  );
  return (
    <div className={cn(setShow && show && "mb-8")}>
      <Section
        titleOther={titleOther}
        title={!titleOther ? "Containers" : undefined}
        icon={!titleOther ? <Box className="w-4 h-4" /> : undefined}
        actions={
          <div className="flex items-center gap-2">
            {pruneButton && !allRunning && (
              <Prune server_id={server_id} type="Containers" />
            )}
            {setShow && <ShowHideButton show={show} setShow={setShow} />}
          </div>
        }
      >
        {show && (
          <DataTable
            tableKey="server-containers"
            data={containers}
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
                size: 300,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Networks" />
                ),
                cell: ({ row }) =>
                  row.original.networks.length > 0 ? (
                    <div className="flex items-center gap-x-2 flex-wrap">
                      {row.original.networks.map((network, i) => (
                        <Fragment key={network}>
                          <DockerResourceLink
                            type="network"
                            server_id={server_id}
                            name={network}
                          />
                          {i !== row.original.networks.length - 1 && (
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
            ]}
          />
        )}
      </Section>
    </div>
  );
};

export const ResourcePageHeader = ({
  intent,
  icon,
  type,
  id,
  name,
  state,
  status,
}: {
  intent: ColorIntention;
  icon: ReactNode;
  name: string | undefined;
  state: string | undefined;
  status: string | undefined;
  // Required for rename
  type: UsableResource | undefined;
  id: string | undefined;
}) => {
  const color = text_color_class_by_intention(intent);
  const background = hex_color_by_intention(intent) + "15";
  return (
    <div
      className="flex items-center gap-8 pl-8 pr-16 py-4 rounded-t-md w-full"
      style={{ background }}
    >
      {icon}
      <div>
        {type && id && name ? (
          <ResourceName type={type} id={id} name={name} />
        ) : (
          <p />
        )}
        {!type && <p className="text-3xl font-semibold">{name}</p>}
        <div className="flex items-center gap-2 text-sm uppercase">
          <p className={cn(color, "font-semibold")}>{state}</p>
          <p className="text-muted-foreground">{status}</p>
        </div>
      </div>
    </div>
  );
};

const ResourceName = ({
  type,
  id,
  name,
}: {
  type: UsableResource;
  id: string;
  name: string;
}) => {
  const invalidate = useInvalidate();
  const { toast } = useToast();
  const { canWrite } = useEditPermissions({ type, id });
  const [newName, setName] = useState(name);
  const [editing, setEditing] = useState(false);
  const { mutate, isPending } = useWrite(`Rename${type}`, {
    onSuccess: () => {
      invalidate([`List${type}s`]);
      toast({ title: `${type} Renamed` });
      setEditing(false);
    },
    onError: () => {
      // If fails, set name back to no name
      setName(name);
    },
  });

  if (editing) {
    return (
      <div className="flex items-center gap-2">
        <Input
          className="text-3xl font-semibold px-1 w-[200px] lg:w-[300px]"
          placeholder="name"
          value={newName}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              if (newName && name !== newName) {
                mutate({ id, name: newName });
              }
            } else if (e.key === "Escape") {
              setEditing(false);
            }
          }}
          autoFocus
        />
        {name !== newName && (
          <Button
            onClick={() => mutate({ id, name: newName })}
            disabled={!newName || isPending}
          >
            {isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : "Save"}
          </Button>
        )}
        {name === newName && (
          <Button variant="ghost" onClick={() => setEditing(false)}>
            <X className="w-4 h-4" />
          </Button>
        )}
      </div>
    );
  } else {
    return (
      <div
        className={cn(
          "flex items-center gap-2 w-full",
          canWrite && "cursor-pointer"
        )}
        onClick={() => {
          if (canWrite) {
            setEditing(true);
          }
        }}
      >
        <p className="text-3xl font-semibold">{name}</p>
        {canWrite && (
          <Button variant="ghost" className="p-2 h-fit">
            <Edit2 className="w-4 h-4" />
          </Button>
        )}
      </div>
    );
  }
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
