import {
  LOGIN_TOKENS,
  useRead,
  useResourceParamType,
  useShiftKeyListener,
  useUser,
  useUserInvalidate,
} from "@lib/hooks";
import { ResourceComponents } from "./resources";
import {
  AlertTriangle,
  Bell,
  Box,
  Boxes,
  CalendarDays,
  FileQuestion,
  FolderTree,
  Keyboard,
  LayoutDashboard,
  LogOut,
  Plus,
  Settings,
  User,
  Users,
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
import { RESOURCE_TARGETS, usableResourcePath } from "@lib/utils";
import { OmniSearch, OmniDialog } from "./omnibar";
import { WsStatusIndicator } from "@lib/socket";
import { TopbarUpdates } from "./updates/topbar";
import { ThemeToggle } from "@ui/theme";
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
import { TopbarAlerts } from "./alert/topbar";
import { ConfirmButton } from "./util";

export const Topbar = () => {
  const [omniOpen, setOmniOpen] = useState(false);
  useShiftKeyListener("S", () => setOmniOpen(true));

  return (
    <div className="fixed top-0 w-full bg-accent z-50 border-b shadow-sm">
      <div className="container h-16 flex items-center justify-between md:grid md:grid-cols-[auto_1fr] lg:grid-cols-3">
        {/* Logo */}
        <Link
          to="/"
          className="flex gap-3 items-center text-2xl tracking-widest md:mx-2"
        >
          <img src="/komodo-512x512.png" className="w-[32px]" />
          <div className="hidden lg:block">KOMODO</div>
        </Link>

        {/* Searchbar */}
        <div className="hidden lg:flex justify-center">
          <OmniSearch setOpen={setOmniOpen} />
        </div>

        {/* Shortcuts */}
        <div className="flex justify-end items-center gap-1">
          <MobileDropdown />
          <OmniSearch setOpen={setOmniOpen} className="lg:hidden" />
          <div className="flex gap-0">
            <Docs />
            <Version />
          </div>
          <WsStatusIndicator />
          <KeyboardShortcuts />
          <TopbarAlerts />
          <TopbarUpdates />
          <ThemeToggle />
          <UserDropdown />
        </div>
      </div>
      <OmniDialog open={omniOpen} setOpen={setOmniOpen} />
    </div>
  );
};

const Docs = () => (
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

const Version = () => {
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

const MobileDropdown = () => {
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

const KeyboardShortcuts = () => {
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

const UserDropdown = () => {
  const [_, setRerender] = useState(false);
  const rerender = () => setRerender((r) => !r);
  const [open, setOpen] = useState(false);
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
        {accounts.map((login) => (
          <div className="flex gap-2 items-center w-full">
            <Button
              variant={
                login.user_id === user?._id?.$oid ? "secondary" : "outline"
              }
              className="flex gap-2 items-center justify-center w-full"
              onClick={() => {
                if (login.user_id === user?._id?.$oid) {
                  // Noop
                  setOpen(false);
                  return;
                }
                LOGIN_TOKENS.change(login.user_id);
                location.reload();
              }}
            >
              <Username user_id={login.user_id} />
            </Button>
            <Button
              variant="destructive"
              size="icon"
              onClick={() => {
                LOGIN_TOKENS.remove(login.user_id);
                if (login.user_id === user?._id?.$oid) {
                  location.reload();
                } else {
                  rerender();
                }
              }}
            >
              <LogOut className="w-4" />
            </Button>
          </div>
        ))}

        <Link
          to={`/login?${new URLSearchParams({ backto: `${location.pathname}${location.search}` })}`}
          className="w-full"
        >
          <Button
            variant="outline"
            onClick={() => setOpen(false)}
            className="flex gap-1 items-center justify-center w-full"
          >
            Add another
            <Plus className="w-4" />
          </Button>
        </Link>

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
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const Username = ({ user_id }: { user_id: string }) => {
  const res = useRead("GetUsername", { user_id }).data;
  return <UsernameView username={res?.username} avatar={res?.avatar} />;
};

const UsernameView = ({
  username,
  avatar,
}: {
  username: string | undefined;
  avatar: string | undefined;
}) => {
  return (
    <>
      {avatar ? <img src={avatar} className="w-4" /> : <User className="w-4" />}
      <div className="hidden xl:flex max-w-[120px] overflow-hidden overflow-ellipsis">
        {username}
      </div>
    </>
  );
};
