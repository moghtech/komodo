import {
  LOGIN_TOKENS,
  useManageUser,
  useRead,
  useResourceParamType,
  useUser,
  useUserInvalidate,
} from "@lib/hooks";
import { ResourceComponents } from "../resources";
import {
  AlertTriangle,
  ArrowLeftRight,
  Bell,
  Box,
  Boxes,
  Calendar,
  CalendarDays,
  Check,
  Circle,
  FileQuestion,
  FolderTree,
  Keyboard,
  LayoutDashboard,
  Loader2,
  LogOut,
  Plus,
  Settings,
  User,
  Users,
  X,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { Button } from "@ui/button";
import { Link } from "react-router-dom";
import {
  cn,
  RESOURCE_TARGETS,
  usableResourcePath,
  version_is_none,
} from "@lib/utils";
import { useAtom } from "jotai";
import { ReactNode, useState } from "react";
import { HomeView, homeViewAtom } from "@main";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Badge } from "@ui/badge";
import { ConfirmButton } from "../util";
import { Types } from "komodo_client";
import { UpdateDetails, UpdateUser } from "@components/updates/details";
import { fmt_date, fmt_operation, fmt_version } from "@lib/formatting";
import { ResourceLink, ResourceNameSimple } from "@components/resources/common";
import { UsableResource } from "@types";
import { AlertLevel } from "@components/alert";
import { AlertDetailsDialogContent } from "@components/alert/details";
import { Separator } from "@ui/separator";

export const MobileDropdown = () => {
  const type = useResourceParamType();
  const Components = type && ResourceComponents[type];
  const [view, setView] = useAtom<HomeView>(homeViewAtom);

  const [icon, title] = Components
    ? [<Components.Icon />, (type === "ResourceSync" ? "Sync" : type) + "s"]
    : location.pathname === "/" && view === "Dashboard"
      ? [<LayoutDashboard className="w-4 h-4" />, "Dashboard"]
      : location.pathname === "/" && view === "Resources"
        ? [<Boxes className="w-4 h-4" />, "Resources"]
        : location.pathname === "/" && view === "Tree"
          ? [<FolderTree className="w-4 h-4" />, "Tree"]
          : location.pathname === "/containers"
            ? [<Box className="w-4 h-4" />, "Containers"]
            : location.pathname === "/settings"
              ? [<Settings className="w-4 h-4" />, "Settings"]
              : location.pathname === "/schedules"
                ? [<CalendarDays className="w-4 h-4" />, "Schedules"]
                : location.pathname === "/alerts"
                  ? [<AlertTriangle className="w-4 h-4" />, "Alerts"]
                  : location.pathname === "/updates"
                    ? [<Bell className="w-4 h-4" />, "Updates"]
                    : location.pathname.split("/")[1] === "user-groups"
                      ? [<Users className="w-4 h-4" />, "User Groups"]
                      : location.pathname.split("/")[1] === "users"
                        ? [<User className="w-4 h-4" />, "Users"]
                        : [<FileQuestion className="w-4 h-4" />, "Unknown"];

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild className="lg:hidden justify-self-end">
        <Button
          variant="ghost"
          className="flex justify-start items-center gap-2 w-36 px-3"
        >
          {icon}
          {title}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-36" side="bottom" align="start">
        <DropdownMenuGroup>
          <DropdownLinkItem
            label="Dashboard"
            icon={<LayoutDashboard className="w-4 h-4" />}
            to="/"
            onClick={() => setView("Dashboard")}
          />
          <DropdownLinkItem
            label="Resources"
            icon={<Boxes className="w-4 h-4" />}
            to="/"
            onClick={() => setView("Resources")}
          />
          <DropdownLinkItem
            label="Containers"
            icon={<Box className="w-4 h-4" />}
            to="/containers"
          />

          <DropdownMenuSeparator />

          {RESOURCE_TARGETS.map((type) => {
            const RTIcon = ResourceComponents[type].Icon;
            const name = type === "ResourceSync" ? "Sync" : type;
            return (
              <DropdownLinkItem
                key={type}
                label={`${name}s`}
                icon={<RTIcon />}
                to={`/${usableResourcePath(type)}`}
              />
            );
          })}

          <DropdownMenuSeparator />

          <DropdownLinkItem
            label="Alerts"
            icon={<AlertTriangle className="w-4 h-4" />}
            to="/alerts"
          />

          <DropdownLinkItem
            label="Updates"
            icon={<Bell className="w-4 h-4" />}
            to="/updates"
          />

          <DropdownMenuSeparator />

          <DropdownLinkItem
            label="Schedules"
            icon={<CalendarDays className="w-4 h-4" />}
            to="/schedules"
          />

          <DropdownLinkItem
            label="Settings"
            icon={<Settings className="w-4 h-4" />}
            to="/settings"
          />
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const DropdownLinkItem = ({
  label,
  icon,
  to,
  onClick,
}: {
  label: string;
  icon: ReactNode;
  to: string;
  onClick?: () => void;
}) => {
  return (
    <Link to={to} onClick={onClick}>
      <DropdownMenuItem className="flex items-center gap-2 cursor-pointer">
        {icon}
        {label}
      </DropdownMenuItem>
    </Link>
  );
};

export const UserDropdown = () => {
  const [_, setRerender] = useState(false);
  const rerender = () => setRerender((r) => !r);
  const [viewLogout, setViewLogout] = useState(false);
  const [open, _setOpen] = useState(false);
  const setOpen = (open: boolean) => {
    _setOpen(open);
    if (open) {
      setViewLogout(false);
    }
  };
  const user = useUser().data;
  const userInvalidate = useUserInvalidate();
  const accounts = LOGIN_TOKENS.accounts();
  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="flex items-center gap-2 px-2">
          <UsernameView
            username={user?.username}
            avatar={(user?.config.data as any).avatar}
          />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="w-[260px] flex flex-col gap-2 items-end p-2"
        side="bottom"
        align="end"
        sideOffset={16}
      >
        <div className="flex items-center justify-between gap-2 w-full">
          <div className="flex gap-2 items-center text-muted-foreground pl-4 text-sm">
            <ArrowLeftRight className="w-4" />
            Switch accounts
          </div>
          <Button
            className="px-2 py-0"
            variant={viewLogout ? "secondary" : "outline"}
            onClick={() => setViewLogout((l) => !l)}
          >
            <Settings className="w-4" />
          </Button>
        </div>

        {accounts.map((login) => (
          <Account
            login={login}
            current_id={user?._id?.$oid}
            setOpen={setOpen}
            rerender={rerender}
            viewLogout={viewLogout}
          />
        ))}

        <Separator />

        <Link
          to={`/login?${new URLSearchParams({ backto: `${location.pathname}${location.search}` })}`}
          className="w-full"
        >
          <Button
            variant="ghost"
            onClick={() => setOpen(false)}
            className="flex gap-1 items-center justify-center w-full"
          >
            Add account
            <Plus className="w-4" />
          </Button>
        </Link>

        {viewLogout && (
          <ConfirmButton
            title="Log Out All"
            icon={<LogOut className="w-4 h-4" />}
            variant="destructive"
            className="flex gap-2 items-center justify-center w-full max-w-full"
            onClick={() => {
              LOGIN_TOKENS.remove_all();
              userInvalidate();
            }}
          />
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const Account = ({
  login,
  current_id,
  setOpen,
  rerender,
  viewLogout,
}: {
  login: Types.JwtResponse;
  current_id?: string;
  setOpen: (open: boolean) => void;
  rerender: () => void;
  viewLogout: boolean;
}) => {
  const res = useRead("GetUsername", { user_id: login.user_id });
  if (!res.data) return;
  const selected = login.user_id === current_id;
  return (
    <div className="flex gap-2 items-center w-full">
      <Button
        variant={selected ? "secondary" : "ghost"}
        className="flex gap-2 items-center justify-between w-full"
        onClick={() => {
          if (selected) {
            // Noop
            setOpen(false);
            return;
          }
          LOGIN_TOKENS.change(login.user_id);
          location.reload();
        }}
      >
        <div className="flex items-center gap-2">
          <UsernameView
            username={res.data?.username}
            avatar={res.data?.avatar}
          />
        </div>
        {selected && (
          <Circle className="w-3 h-3 stroke-none transition-colors fill-green-500" />
        )}
      </Button>

      {viewLogout && (
        <Button
          variant="destructive"
          className="px-2 py-0"
          onClick={() => {
            LOGIN_TOKENS.remove(login.user_id);
            if (selected) {
              location.reload();
            } else {
              rerender();
            }
          }}
        >
          <LogOut className="w-4" />
        </Button>
      )}
    </div>
  );
};

const UsernameView = ({
  username,
  avatar,
  full,
}: {
  username: string | undefined;
  avatar: string | undefined;
  full?: boolean;
}) => {
  return (
    <>
      {avatar ? <img src={avatar} className="w-4" /> : <User className="w-4" />}
      <div
        className={cn(
          "overflow-hidden overflow-ellipsis",
          full ? "max-w-[200px]" : "hidden xl:flex max-w-[140px]"
        )}
      >
        {username}
      </div>
    </>
  );
};

export const TopbarUpdates = () => {
  const updates = useRead("ListUpdates", {}).data;

  const last_opened = useUser().data?.last_update_view;
  const unseen_update = updates?.updates.some(
    (u) => u.start_ts > (last_opened ?? Number.MAX_SAFE_INTEGER)
  );

  const userInvalidate = useUserInvalidate();
  const { mutate } = useManageUser("SetLastSeenUpdate", {
    onSuccess: userInvalidate,
  });

  return (
    <DropdownMenu onOpenChange={(o) => o && mutate({})}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="w-4 h-4" />
          <Circle
            className={cn(
              "absolute top-2 right-2 w-2 h-2 stroke-blue-500 fill-blue-500 transition-opacity",
              unseen_update ? "opacity-1" : "opacity-0"
            )}
          />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="w-[100vw] md:w-[500px] h-[500px] overflow-auto"
        sideOffset={20}
      >
        <DropdownMenuGroup>
          {updates?.updates.map((update) => (
            <SingleUpdate update={update} key={update.id} />
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const SingleUpdate = ({ update }: { update: Types.UpdateListItem }) => {
  const Components =
    update.target.type !== "System"
      ? ResourceComponents[update.target.type]
      : null;

  const Icon = () => {
    if (update.status === Types.UpdateStatus.Complete) {
      if (update.success) return <Check className="w-4 h-4 stroke-green-500" />;
      else return <X className="w-4 h-4 stroke-red-500" />;
    } else return <Loader2 className="w-4 h-4 animate-spin" />;
  };

  return (
    <UpdateDetails id={update.id}>
      <div className="px-2 py-4 hover:bg-muted transition-colors border-b last:border-none cursor-pointer">
        <div className="flex items-center justify-between">
          <div className="text-sm w-full">
            <div className="flex items-center gap-2">
              <Icon />
              {fmt_operation(update.operation)}
              <div className="text-xs text-muted-foreground">
                {!version_is_none(update.version) &&
                  fmt_version(update.version)}
              </div>
            </div>
            <div className="flex items-center gap-2 text-muted-foreground">
              {Components && (
                <>
                  <Components.Icon />
                  <ResourceNameSimple
                    type={update.target.type as UsableResource}
                    id={update.target.id}
                  />
                </>
              )}
              {!Components && (
                <>
                  <Settings className="w-4 h-4" />
                  System
                </>
              )}
            </div>
          </div>
          <div className="text-xs text-muted-foreground w-48">
            <div className="flex items-center gap-2 h-[20px]">
              <Calendar className="w-4 h-4" />
              <div>
                {update.status === Types.UpdateStatus.InProgress
                  ? "ongoing"
                  : fmt_date(new Date(update.start_ts))}
              </div>
            </div>
            <UpdateUser user_id={update.operator} iconSize={4} defaultAvatar />
          </div>
        </div>
      </div>
    </UpdateDetails>
  );
};

export const TopbarAlerts = () => {
  const { data } = useRead(
    "ListAlerts",
    { query: { resolved: false } },
    { refetchInterval: 3_000 }
  );
  const [open, setOpen] = useState(false);

  // If this is set, details will open.
  const [alert, setAlert] = useState<Types.Alert>();

  if (!data || data.alerts.length === 0) {
    return null;
  }

  return (
    <>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger asChild disabled={!data?.alerts.length}>
          <Button variant="ghost" size="icon" className="relative">
            <AlertTriangle className="w-4 h-4" />
            {!!data?.alerts.length && (
              <div className="absolute top-0 right-0 w-4 h-4 bg-red-500 flex items-center justify-center text-[10px] text-white rounded-full">
                {data.alerts.length}
              </div>
            )}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent sideOffset={20}>
          {data?.alerts.map((alert) => (
            <DropdownMenuItem
              key={alert._id?.$oid}
              className="flex items-center gap-8 border-b last:border-none cursor-pointer"
              onClick={() => setAlert(alert)}
            >
              <div className="w-24">
                <AlertLevel level={alert.level} />
              </div>
              <div className="w-64">
                <div className="w-fit">
                  <ResourceLink
                    type={alert.target.type as UsableResource}
                    id={alert.target.id}
                    onClick={() => setOpen(false)}
                  />
                </div>
              </div>
              <p className="w-64">{alert.data.type}</p>
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
      <AlertDetails alert={alert} onClose={() => setAlert(undefined)} />
    </>
  );
};

const AlertDetails = ({
  alert,
  onClose,
}: {
  alert: Types.Alert | undefined;
  onClose: () => void;
}) => (
  <>
    {alert && (
      <Dialog open={!!alert} onOpenChange={(o) => !o && onClose()}>
        <AlertDetailsDialogContent alert={alert} onClose={onClose} />
      </Dialog>
    )}
  </>
);

export const Docs = () => (
  <a
    href="https://komo.do/docs/intro"
    target="_blank"
    className="hidden lg:block"
  >
    <Button variant="link" size="sm" className="px-2">
      <div>Docs</div>
    </Button>
  </a>
);

export const Version = () => {
  const version = useRead("GetVersion", {}, { refetchInterval: 30_000 }).data
    ?.version;

  if (!version) return null;
  return (
    <a
      href="https://github.com/moghtech/komodo/releases"
      target="_blank"
      className="hidden lg:block"
    >
      <Button variant="link" size="sm" className="px-2">
        <div>v{version}</div>
      </Button>
    </a>
  );
};

export const KeyboardShortcuts = () => {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost" size="icon" className="hidden md:flex">
          <Keyboard className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Keyboard Shortcuts</DialogTitle>
        </DialogHeader>
        <div className="grid gap-3 grid-cols-2 pt-8">
          <KeyboardShortcut label="Save" keys={["Ctrl / Cmd", "Enter"]} />
          <KeyboardShortcut label="Go Home" keys={["Shift", "H"]} />

          <KeyboardShortcut label="Go to Servers" keys={["Shift", "G"]} />
          <KeyboardShortcut label="Go to Stacks" keys={["Shift", "Z"]} />
          <KeyboardShortcut label="Go to Deployments" keys={["Shift", "D"]} />
          <KeyboardShortcut label="Go to Builds" keys={["Shift", "B"]} />
          <KeyboardShortcut label="Go to Repos" keys={["Shift", "R"]} />
          <KeyboardShortcut label="Go to Procedures" keys={["Shift", "P"]} />

          <KeyboardShortcut label="Search" keys={["Shift", "S"]} />
          <KeyboardShortcut label="Add Filter Tag" keys={["Shift", "T"]} />
          <KeyboardShortcut
            label="Clear Filter Tags"
            keys={["Shift", "C"]}
            divider={false}
          />
        </div>
      </DialogContent>
    </Dialog>
  );
};

const KeyboardShortcut = ({
  label,
  keys,
  divider = true,
}: {
  label: string;
  keys: string[];
  divider?: boolean;
}) => {
  return (
    <>
      <div>{label}</div>
      <div className="flex items-center gap-2">
        {keys.map((key) => (
          <Badge variant="secondary" key={key}>
            {key}
          </Badge>
        ))}
      </div>

      {divider && (
        <div className="col-span-full bg-gray-600 h-[1px] opacity-40" />
      )}
    </>
  );
};
